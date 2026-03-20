//! Batch GKR with parallel template proving via Rayon work-stealing.
use std::io::Cursor;
use std::sync::Mutex;
use arith::Field;
use gkr::{gkr_prove_batch, gkr_verify};
use gkr_engine::{ExpanderPCS, FieldEngine, GKREngine, MPIConfig, Transcript};
use crate::{frontend::{Config, SIMDField}, utils::misc::next_power_of_two,
    zkcuda::{context::ComputationGraph, proving_system::{common::check_inputs,
        expander::{prove_impl::{get_local_vals, prepare_expander_circuit, prepare_inputs_with_local_vals},
            structs::{ExpanderProof, ExpanderProverSetup, ExpanderVerifierSetup}},
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
        let t_commit = std::time::Instant::now();
        let (commitments, commit_states): (Vec<_>, Vec<_>) = dm.iter().map(|m| local_commit_impl::<C, ECCConfig>(ps.p_keys.get(&m.len()).unwrap(), m)).unzip();
        eprintln!("  [commit] {:?}", t_commit.elapsed());
        let templates = cg.proof_templates();
        let kernels = cg.kernels();
        let n = templates.len();
        let results: Vec<Mutex<Option<ExpanderProof>>> = (0..n).map(|_| Mutex::new(None)).collect();
        let commit_states = &commit_states;
        rayon::scope(|scope| {
            for (ti, tmpl) in templates.iter().enumerate() {
                let ps_ptr = ps as *const _ as usize;
                let dm = &dm;
                let kernels = &kernels;
                let slot = &results[ti];
                scope.spawn(move |_| {
                    let ps: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig> = unsafe { &*(ps_ptr as *const _) };
                    let proof = prove_one::<C, ECCConfig>(ti, tmpl, kernels, dm, ps, commit_states);
                    *slot.lock().unwrap() = Some(proof);
                });
            }
        });
        let proofs = results.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect();
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
                let mut t = C::TranscriptConfig::new();
                ec.fill_rnd_coefs(&mut t);
                let mut cur = Cursor::new(&lp.data[0].bytes);
                let (ok, ch, _v0, _v1) = gkr_verify(pc, &ec, &[], &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO, &mut t, &mut cur);
                if !ok { eprintln!("Batch GKR verify fail tmpl {ti}"); return false; }
                let chs = if let Some(cy) = ch.challenge_y() { vec![ch.challenge_x(), cy] } else { vec![ch.challenge_x()] };
                for sc in &chs {
                    for (&ref comm, &_ib) in comms.iter().zip(tmpl.is_broadcast().iter()) {
                        let commitment_len = comm.vals_len;
                        let local_size = commitment_len >> sc.r_mpi.len();
                        let n_local = if local_size > 0 { local_size.ilog2() as usize } else { 0 };
                        let mut eval_ch = sc.clone();
                        eval_ch.rz.truncate(n_local);
                        let _v: <C::FieldConfig as FieldEngine>::ChallengeField = t.generate_field_element();
                        let max_len = *vs.v_keys.keys().max().unwrap();
                        let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::gen_params(max_len.ilog2() as usize, 1);
                        let v_key = vs.v_keys.get(&max_len).unwrap();
                        let mut pcs_ch = sc.clone();
                        pcs_ch.rz.extend_from_slice(&pcs_ch.r_mpi);
                        pcs_ch.r_mpi = vec![];
                        let target_rz = max_len.ilog2() as usize;
                        while pcs_ch.rz.len() < target_rz { pcs_ch.rz.push(<C::FieldConfig as FieldEngine>::ChallengeField::ZERO); }
                        // TODO: actually verify PCS opening against commitment
                        let _ = (v_key, &params, &pcs_ch, comm, &mut cur);
                    }
                }
            } else {
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
    commit_states: &[crate::zkcuda::proving_system::expander::structs::ExpanderCommitmentState<C::FieldConfig, C::PCSConfig>],
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
        // Use clone_for_batch to share gate arrays (avoids O(N × gates) clone overhead)
        let mut circuits: Vec<_> = (0..pc).map(|pi| {
            let mut c = unsafe { tc.clone_for_batch() };
            let lv = get_local_vals(&cvs, tmpl.is_broadcast(), pi, pc);
            c.layers[0].input_vals = prepare_inputs_with_local_vals(is, kernel.layered_circuit_input(), &lv);
            c.evaluate(); c
        }).collect();
        let mut sps: Vec<_> = (0..pc).map(|_| bs.clone()).collect();
        let t1 = std::time::Instant::now();
        let (cv, ch) = gkr_prove_batch(&circuits, &mut sps, &mut tr);
        // Drop batch clones without freeing shared gate memory
        for c in circuits { unsafe { c.drop_batch_clone(); } }
        assert_eq!(cv, <C::FieldConfig as FieldEngine>::ChallengeField::from(0u32));
        let t2 = std::time::Instant::now();
        let chs = if let Some(cy) = ch.challenge_y() { vec![ch.challenge_x(), cy] } else { vec![ch.challenge_x()] };
        for sc in &chs {
            for (ci, (&ref v, &_ib)) in cvs.iter().zip(tmpl.is_broadcast().iter()).enumerate() {
                let pc2 = sc.clone();
                let comm_idx = tmpl.commitment_indices()[ci];
                let scratch = &commit_states[comm_idx].scratch;
                crate::zkcuda::proving_system::expander::prove_impl::pcs_batch_open_with_scratch::<C>(
                    v, &pc2, ps, &mut tr, Some(scratch));
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
        // Open PCS with commit scratch pads for Orion compatibility
        let challenges = if let Some(cy) = ch.challenge_y() { vec![ch.challenge_x(), cy] } else { vec![ch.challenge_x()] };
        for sc in &challenges {
            for (ci, (v, &ib)) in cvs.iter().zip(tmpl.is_broadcast().iter()).enumerate() {
                let comm_idx = tmpl.commitment_indices()[ci];
                let val_len = v.len();
                let (challenge_for_pcs, _) = crate::zkcuda::proving_system::expander::prove_impl::partition_challenge_and_location_for_pcs_no_mpi::<C::FieldConfig>(
                    sc, val_len, 0, 1, ib);
                let scratch = &commit_states[comm_idx].scratch;
                crate::zkcuda::proving_system::expander::prove_impl::pcs_local_open_with_scratch::<C>(
                    v, &challenge_for_pcs, ps, &mut tr, Some(scratch));
            }
        }
        eprintln!("  [batch] tmpl[{}] N=1 t={:?}", ti, t0.elapsed());
        ExpanderProof { data: vec![tr.finalize_and_get_proof()] }
    }
}
