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
            expander::{
                structs::{ExpanderCommitment, ExpanderProof, ExpanderVerifierSetup},
                verify_impl::verify_pcs_opening_and_aggregation_no_mpi,
            },
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
    let (ok, ch, v0, v1) = gkr_verify(
        pc, &ec, &[],
        &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO,
        &mut t, &mut cur,
    );
    if !ok { return false; }

    verify_pcs_opening_and_aggregation_no_mpi::<C, ECCConfig>(
        &mut cur, kernel, vs, &ch, v0, v1, comms, is_broadcast, 0, pc, &mut t)
}
