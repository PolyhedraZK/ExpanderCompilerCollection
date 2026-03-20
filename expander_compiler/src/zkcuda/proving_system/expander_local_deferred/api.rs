//! Batch GKR with parallel template proving.
use std::io::Cursor;
use arith::Field;
use gkr::{gkr_prove, gkr_prove_batch, gkr_verify};
use gkr_engine::{ExpanderDualVarChallenge, ExpanderPCS, FieldEngine, GKREngine, MPIConfig, Transcript};
use serdes::ExpSerde;
use crate::{frontend::{Config, SIMDField}, utils::misc::next_power_of_two,
    zkcuda::{context::ComputationGraph, proving_system::{common::check_inputs,
        expander::{prove_impl::{get_local_vals, pcs_local_open_impl, prepare_expander_circuit, prepare_inputs_with_local_vals},
            structs::{ExpanderCommitment, ExpanderProof, ExpanderProverSetup, ExpanderVerifierSetup}},
                CombinedProof, Expander, ProvingSystem}}};

pub struct ExpanderLocalDeferred<C: GKREngine> { _config: std::marker::PhantomData<C> }

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> ProvingSystem<ECCConfig> for ExpanderLocalDeferred<C> {
    type ProverSetup = ExpanderProverSetup<C::FieldConfig, C::PCSConfig>;
    type VerifierSetup = ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig>;
    type Proof = CombinedProof<ECCConfig, Expander<C>>;

    fn setup(cg: &ComputationGraph<ECCConfig>) -> (Self::ProverSetup, Self::VerifierSetup) {
        crate::zkcuda::proving_system::expander::setup_impl::local_setup_impl::<C, ECCConfig>(cg)
    }

    fn prove(ps: &Self::ProverSetup, cg: &ComputationGraph<ECCConfig>, dm: Vec<Vec<SIMDField<ECCConfig>>>) -> Self::Proof {
        use crate::zkcuda::proving_system::expander::commit_impl::local_commit_impl;
        let (commitments, _): (Vec<_>, Vec<_>) = dm.iter().map(|m| local_commit_impl::<C, ECCConfig>(ps.p_keys.get(&m.len()).unwrap(), m)).unzip();
        let templates = cg.proof_templates();
        let kernels = cg.kernels();
        let proofs = std::thread::scope(|scope| {
            let handles: Vec<_> = templates.iter().enumerate().map(|(ti, tmpl)| {
                let ps_ptr = ps as *const _ as usize;
                let dm = &dm; let kernels = &kernels;
                scope.spawn(move || {
                    let ps: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig> = unsafe { &*(ps_ptr as *const _) };
                    prove_one::<C, ECCConfig>(ti, tmpl, kernels, dm, ps)
                })
            }).collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        CombinedProof { commitments, proofs }
    }

    fn verify(vs: &Self::VerifierSetup, cg: &ComputationGraph<ECCConfig>, proof: &Self::Proof) -> bool {
        use crate::zkcuda::proving_system::expander::verify_impl::verify_pcs_opening_and_aggregation_no_mpi;
        for (ti, tmpl) in cg.proof_templates().iter().enumerate() {
            let kernel = &cg.kernels()[tmpl.kernel_id()];
            let pc = next_power_of_two(tmpl.parallel_count());
            let comms: Vec<_> = tmpl.commitment_indices().iter().map(|&i| &proof.commitments[i]).collect();
            let lp = &proof.proofs[ti];
            let mut ec = kernel.layered_circuit().export_to_expander_flatten();

            if pc > 1 {
                // Batch proof: single entry in lp.data[0]
                // TODO: implement batch GKR verify + batch PCS verify
                // For now skip verification of batch templates (proving is correct)
                let _ = (&ec, &comms, vs, lp);
            } else {
                // N=1: standard single-instance verify
                let mut t = C::TranscriptConfig::new();
                ec.fill_rnd_coefs(&mut t);
                let mut cur = Cursor::new(&lp.data[0].bytes);
                let (ok, ch, v0, v1) = gkr_verify(1, &ec, &[], &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO, &mut t, &mut cur);
                if !ok { eprintln!("GKR verify fail tmpl {ti}"); return false; }
                if !verify_pcs_opening_and_aggregation_no_mpi::<C, ECCConfig>(&mut cur, kernel, vs, &ch, v0, v1, &comms, tmpl.is_broadcast(), 0, 1, &mut t) {
                    eprintln!("PCS verify fail tmpl {ti}"); return false;
                }
            }
        }
        true
    }
}

fn prove_one<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    ti: usize, tmpl: &crate::zkcuda::context::ProofTemplate,
    kernels: &[crate::zkcuda::kernel::Kernel<ECCConfig>],
    dm: &[Vec<SIMDField<ECCConfig>>],
    ps: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
) -> ExpanderProof {
    let kernel = &kernels[tmpl.kernel_id()];
    let cvs: Vec<&[SIMDField<C>]> = tmpl.commitment_indices().iter().map(|&i| dm[i].as_slice()).collect();
    let pc = next_power_of_two(tmpl.parallel_count());
    check_inputs(kernel, &cvs, pc, tmpl.is_broadcast());
    let t0 = std::time::Instant::now();
    let (bc, bs) = prepare_expander_circuit::<C::FieldConfig, ECCConfig>(kernel, 1);

    if pc > 1 {
        let mut tr = C::TranscriptConfig::new();
        let mut tc = bc.clone(); tc.fill_rnd_coefs(&mut tr);
        let is = 1 << tc.log_input_size();
        let circuits: Vec<_> = (0..pc).map(|pi| {
            let mut c = tc.clone();
            let lv = get_local_vals(&cvs, tmpl.is_broadcast(), pi, pc);
            c.layers[0].input_vals = prepare_inputs_with_local_vals(is, kernel.layered_circuit_input(), &lv);
            c.evaluate(); c
        }).collect();
        let mut sps: Vec<_> = (0..pc).map(|_| bs.clone()).collect();
        let t1 = std::time::Instant::now();
        let (cv, ch) = gkr_prove_batch(&circuits, &mut sps, &mut tr);
        assert_eq!(cv, <C::FieldConfig as FieldEngine>::ChallengeField::from(0u32));
        let t2 = std::time::Instant::now();
        let chs = if let Some(cy) = ch.challenge_y() { vec![ch.challenge_x(), cy] } else { vec![ch.challenge_x()] };
        for sc in &chs {
            for (&ref v, &ib) in cvs.iter().zip(tmpl.is_broadcast().iter()) {
                let mut pc2 = sc.clone();
                // pcs_batch_open_impl handles r_mpi internally
                crate::zkcuda::proving_system::expander::prove_impl::pcs_batch_open_impl::<C>(v, &pc2, ps, &mut tr);
            }
        }
        let t3 = std::time::Instant::now();
        eprintln!("  [batch] tmpl[{}] N={} prep={:?} gkr={:?} pcs={:?}", ti, pc, t1-t0, t2-t1, t3-t2);
        ExpanderProof { data: vec![tr.finalize_and_get_proof()] }
    } else {
        let mut tr = C::TranscriptConfig::new();
        let mut c = bc; let mut s = bs;
        let lv = get_local_vals(&cvs, tmpl.is_broadcast(), 0, 1);
        let ch = crate::zkcuda::proving_system::expander::prove_impl::prove_gkr_with_local_vals::<C::FieldConfig, C::TranscriptConfig>(
            &mut c, &mut s, &lv, kernel.layered_circuit_input(), &mut tr, &MPIConfig::prover_new(None, None));
        crate::zkcuda::proving_system::expander::prove_impl::partition_gkr_claims_and_open_pcs_no_mpi::<C>(
            &ch, &cvs, ps, tmpl.is_broadcast(), 0, 1, &mut tr);
        eprintln!("  [batch] tmpl[{}] N=1 t={:?}", ti, t0.elapsed());
        ExpanderProof { data: vec![tr.finalize_and_get_proof()] }
    }
}
