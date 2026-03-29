use std::io::Cursor;

use arith::Field;
use gkr::gkr_verify;
use gkr_engine::{FieldEngine, GKREngine, Transcript};

use crate::{
    frontend::{Config, SIMDField},
    utils::misc::next_power_of_two,
    zkcuda::{
        context::ComputationGraph,
        proving_system::{
            common::bind_commitments_to_transcript,
            expander::structs::{ExpanderCommitment, ExpanderProof, ExpanderVerifierSetup},
            CombinedProof,
        },
    },
};

/// Verify all template proofs: GKR + PCS opening + commitment binding.
pub fn verify_all<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    vs: &ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig>,
    cg: &ComputationGraph<ECCConfig>,
    proof: &CombinedProof<ECCConfig, crate::zkcuda::proving_system::Expander<C>>,
) -> bool {
    use rayon::prelude::*;
    let vs_ptr = vs as *const _ as usize;
    let results: Vec<bool> = cg.proof_templates().par_iter().enumerate().map(|(ti, tmpl)| {
        let vs: &ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig> = unsafe { &*(vs_ptr as *const _) };
        let kernel = &cg.kernels()[tmpl.kernel_id()];
        let pc = next_power_of_two(tmpl.parallel_count());
        let comms: Vec<_> = tmpl.commitment_indices().iter().map(|&i| &proof.commitments[i]).collect();
        let lp = &proof.proofs[ti];

        let ok = verify_template::<C, ECCConfig>(vs, kernel, lp, &comms, tmpl.is_broadcast(), pc, &proof.commitments);
        if !ok { eprintln!("  [verify-fail] template {} (pc={}) FAILED", ti, pc); }
        ok
    }).collect();
    results.iter().all(|&x| x)
}

/// Verify a single template proof (GKR + PCS).
fn verify_template<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    vs: &ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig>,
    kernel: &crate::zkcuda::kernel::Kernel<ECCConfig>,
    lp: &ExpanderProof,
    comms: &[&ExpanderCommitment<C::FieldConfig, C::PCSConfig>],
    is_broadcast: &[bool],
    pc: usize,
    all_commitments: &[ExpanderCommitment<C::FieldConfig, C::PCSConfig>],
) -> bool {
    let mut ec = kernel.layered_circuit().export_to_expander_flatten();
    let mut t = C::TranscriptConfig::new();
    bind_commitments_to_transcript::<C>(&mut t, all_commitments);
    ec.fill_rnd_coefs(&mut t);

    let mut cur = Cursor::new(&lp.data[0].bytes);
    let (ok, ch, v0, v1) = gkr_verify(
        pc, &ec, &[],
        &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO,
        &mut t, &mut cur,
    );
    if !ok { return false; }

    use crate::zkcuda::proving_system::expander::verify_impl::{
        verify_pcs, verify_pcs_opening_and_aggregation_no_mpi,
    };
    use serdes::ExpSerde;

    if pc <= 1 {
        // N=1: prover used pcs_local_open_with_scratch (partitioned challenges)
        verify_pcs_opening_and_aggregation_no_mpi::<C, ECCConfig>(
            &mut cur, kernel, vs, &ch, v0, v1, comms, is_broadcast, 0, 1, &mut t)
    } else {
        // N>1: prover used pcs_batch_open_with_scratch — same merged challenge for ALL commitments.
        // Do NOT use verify_pcs_opening_and_aggregation (it partitions challenges per commitment).
        let chs = if let Some(cy) = ch.challenge_y() { vec![ch.challenge_x(), cy] } else { vec![ch.challenge_x()] };
        for sc in &chs {
            // Merge r_mpi into rz (matching prover's pcs_batch_open_with_scratch line 337-339)
            let mut merged = sc.clone();
            merged.rz.extend_from_slice(&merged.r_mpi);
            merged.r_mpi = vec![];
            // Pad rz if needed (matching prover line 342-345), do NOT truncate
            // (prover also doesn't truncate — PCS::open/verify handle via lift_poly)

            for (&ref comm, &_ib) in comms.iter().zip(is_broadcast.iter()) {
                let claim = <C::FieldConfig as FieldEngine>::ChallengeField::deserialize_from(&mut cur).unwrap();
                t.append_field_element(&claim);

                // Use the same merged challenge for ALL commitments (matching prover)
                let verified = verify_pcs::<C, ECCConfig>(&mut cur, comm, &merged, &claim, vs, &mut t);
                if !verified { return false; }
            }
        }
        true
    }
}
