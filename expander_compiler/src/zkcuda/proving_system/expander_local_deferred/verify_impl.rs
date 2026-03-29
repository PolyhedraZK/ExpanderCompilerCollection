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

        verify_template::<C, ECCConfig>(vs, kernel, lp, &comms, tmpl.is_broadcast(), pc, &proof.commitments)
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
    let (ok, ch, _v0, _v1) = gkr_verify(
        pc, &ec, &[],
        &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO,
        &mut t, &mut cur,
    );
    if !ok { return false; }

    // Verify PCS openings matching prover's pcs_batch_open_with_scratch format:
    // For each challenge direction (x, optionally y), for each commitment:
    //   1. Read eval claim from proof
    //   2. Verify PCS opening at merged challenge (rz ++ r_mpi)
    use gkr_engine::ExpanderPCS;
    use serdes::ExpSerde;
    let chs = if let Some(cy) = ch.challenge_y() { vec![ch.challenge_x(), cy] } else { vec![ch.challenge_x()] };
    for sc in &chs {
        for (&ref comm, &_ib) in comms.iter().zip(is_broadcast.iter()) {
            let commit_len = comm.vals_len;

            // Read eval claim (prover wrote transcript.append_field_element(&v))
            let claim = <C::FieldConfig as FieldEngine>::ChallengeField::deserialize_from(&mut cur).unwrap();
            t.append_field_element(&claim);

            // Build PCS challenge: merge r_mpi into rz (matching prover)
            let mut pcs_ch = sc.clone();
            pcs_ch.rz.extend_from_slice(&pcs_ch.r_mpi);
            pcs_ch.r_mpi = vec![];
            let target_rz = commit_len.ilog2() as usize;
            while pcs_ch.rz.len() < target_rz {
                pcs_ch.rz.push(<C::FieldConfig as FieldEngine>::ChallengeField::ZERO);
            }

            // Verify PCS opening
            let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::gen_params(commit_len.ilog2() as usize, 1);
            let v_key = vs.v_keys.get(&commit_len)
                .unwrap_or_else(|| vs.v_keys.get(vs.v_keys.keys().max().unwrap()).unwrap());
            let opening = <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::Opening::deserialize_from(&mut cur).unwrap();
            t.lock_proof();
            let verified = <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::verify(
                &params, v_key, &comm.commitment, &pcs_ch, claim, &mut t, &opening);
            t.unlock_proof();
            let mut buf = vec![];
            opening.serialize_into(&mut buf).unwrap();
            t.append_u8_slice(&buf);

            if !verified { return false; }
        }
    }
    true
}
