use std::io::Cursor;

use arith::Field;
use gkr::gkr_verify;
use gkr_engine::{FieldEngine, GKREngine, Transcript};
use polynomials::EqPolynomial;

use crate::{
    frontend::{Config, SIMDField},
    utils::misc::next_power_of_two,
    zkcuda::{
        context::ComputationGraph,
        proving_system::{
            common::bind_commitments_to_transcript,
            expander::{
                structs::{ExpanderCommitment, ExpanderProof, ExpanderVerifierSetup},
                verify_impl::{verify_pcs, verify_pcs_opening_and_aggregation_no_mpi},
            },
            CombinedProof, Commitment,
        },
    },
};

/// Verify all template proofs: GKR + PCS opening + commitment binding.
pub fn verify_all<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    vs: &ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig>,
    cg: &ComputationGraph<ECCConfig>,
    proof: &CombinedProof<ECCConfig, crate::zkcuda::proving_system::Expander<C>>,
) -> bool {
    let results: Vec<bool> = cg.proof_templates().iter().enumerate().map(|(ti, tmpl)| {
        let kernel = &cg.kernels()[tmpl.kernel_id()];
        let pc = next_power_of_two(tmpl.parallel_count());
        let comms: Vec<_> = tmpl.commitment_indices().iter().map(|&i| &proof.commitments[i]).collect();
        let lp = &proof.proofs[ti];

        let ok = verify_template::<C, ECCConfig>(vs, kernel, lp, &comms, tmpl.is_broadcast(), pc, &proof.commitments);
        if !ok {
            let bc: Vec<_> = tmpl.is_broadcast().iter().map(|&b| if b {"B"} else {"_"}).collect();
            let vls: Vec<_> = tmpl.commitment_indices().iter().map(|&i| proof.commitments[i].vals_len).collect();
            eprintln!("  [verify-fail] tmpl {} pc={} bc={:?} vls={:?}", ti, pc, bc, vls);
        }
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
    if !ok { eprintln!("      [gkr-verify-fail] pc={}", pc); return false; }

    use serdes::ExpSerde;

    if pc <= 1 {
        // N=1: standard PCS verify with partition_challenge
        verify_pcs_opening_and_aggregation_no_mpi::<C, ECCConfig>(
            &mut cur, kernel, vs, &ch, v0, v1, comms, is_broadcast, 0, 1, &mut t)
    } else {
        // N>1 batch: PCS challenge = rz[0..partition_vars] ++ r_mpi
        // Aggregation: y = Σ eq(component_vars, i) × claim_i
        // component_vars = rz[partition_vars..output_var_num] (WITHOUT r_mpi)
        let chs = if let Some(cy) = ch.challenge_y() { vec![ch.challenge_x(), cy] } else { vec![ch.challenge_x()] };
        let claims = if let Some(v1) = v1 { vec![v0, v1] } else { vec![v0] };

        for (sc, y) in chs.iter().zip(claims.iter()) {
            let mut target_y = <C::FieldConfig as FieldEngine>::ChallengeField::ZERO;
            for ((input, &ref comm), &ib) in kernel.layered_circuit_input().iter()
                .zip(comms.iter()).zip(is_broadcast.iter())
            {
                let val_len = <ExpanderCommitment<C::FieldConfig, C::PCSConfig> as Commitment<ECCConfig>>::vals_len(comm);
                let partition_size = if ib { val_len } else { val_len / pc };
                let n_pv = if partition_size > 0 { partition_size.ilog2() as usize } else { 0 };

                let claim = <C::FieldConfig as FieldEngine>::ChallengeField::deserialize_from(&mut cur).unwrap();
                t.append_field_element(&claim);

                let pcs_ch = gkr_engine::ExpanderSingleVarChallenge::new(
                    { let mut rz = sc.rz[..n_pv].to_vec(); rz.extend_from_slice(&sc.r_mpi); rz },
                    sc.r_simd.clone(), vec![],
                );
                if !verify_pcs::<C, ECCConfig>(&mut cur, comm, &pcs_ch, &claim, vs, &mut t) {
                    eprintln!("      [pcs-fail] vl={} n_pv={} pcs_rz={} pcs_simd={}", val_len, n_pv, pcs_ch.rz.len(), pcs_ch.r_simd.len());
                    return false;
                }

                let component_idx_vars = &sc.rz[n_pv..];
                let component_index = input.offset / input.len;
                let v_index = EqPolynomial::ith_eq_vec_elem(component_idx_vars, component_index);
                target_y += v_index * claim;
            }
            if *y != target_y {
                eprintln!("      [agg-fail] y={:?} target={:?} rz_len={} r_mpi_len={}",
                    y, target_y, sc.rz.len(), sc.r_mpi.len());
                return false;
            }
        }
        true
    }
}
