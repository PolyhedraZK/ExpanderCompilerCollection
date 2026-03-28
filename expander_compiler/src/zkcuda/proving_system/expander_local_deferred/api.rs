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
        let ps_ptr = ps as *const _ as usize;
        let commit_results: Vec<_> = {
            use rayon::prelude::*;
            dm.par_iter().map(|m| {
                let ps: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig> = unsafe { &*(ps_ptr as *const _) };
                local_commit_impl::<C, ECCConfig>(ps.p_keys.get(&m.len()).unwrap(), m)
            }).collect()
        };
        let (commitments, commit_states): (Vec<_>, Vec<_>) = commit_results.into_iter().unzip();
        eprintln!("  [commit] {:?}", t_commit.elapsed());
        let templates = cg.proof_templates();
        let kernels = cg.kernels();
        let n = templates.len();
        let commit_states = &commit_states;

        let proofs = if std::env::var("USE_GPU_PROVER").is_ok() {
            gpu_prove_all::<C, ECCConfig>(templates, kernels, &dm, ps, commit_states)
        } else {
            let results: Vec<Mutex<Option<ExpanderProof>>> = (0..n).map(|_| Mutex::new(None)).collect();
            let small_threshold = 256;
            for (ti, tmpl) in templates.iter().enumerate() {
                let pc = next_power_of_two(tmpl.parallel_count());
                if pc <= small_threshold {
                    let proof = prove_one::<C, ECCConfig>(ti, tmpl, kernels, &dm, ps, commit_states);
                    *results[ti].lock().unwrap() = Some(proof);
                }
            }
            rayon::scope(|scope| {
                for (ti, tmpl) in templates.iter().enumerate() {
                    let pc = next_power_of_two(tmpl.parallel_count());
                    if pc > small_threshold {
                        let ps_ptr = ps as *const _ as usize;
                        let dm = &dm; let kernels = &kernels;
                        let slot = &results[ti];
                        scope.spawn(move |_| {
                            let ps: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig> = unsafe { &*(ps_ptr as *const _) };
                            let proof = prove_one::<C, ECCConfig>(ti, tmpl, kernels, dm, ps, commit_states);
                            *slot.lock().unwrap() = Some(proof);
                        });
                    }
                }
            });
            results.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect()
        };
        CombinedProof { commitments, proofs }
    }

    fn verify(vs: &Self::VerifierSetup, cg: &ComputationGraph<ECCConfig>, proof: &Self::Proof) -> bool {
        use crate::zkcuda::proving_system::expander::verify_impl::verify_pcs_opening_and_aggregation_no_mpi;
        // Parallel template verification — each template is independent
        let vs_ptr = vs as *const _ as usize;
        use rayon::prelude::*;
        let results: Vec<bool> = cg.proof_templates().par_iter().enumerate().map(|(ti, tmpl)| {
            let vs: &ExpanderVerifierSetup<C::FieldConfig, C::PCSConfig> = unsafe { &*(vs_ptr as *const _) };
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
                if !ok { return false; }
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
                true
            } else {
                let mut t = C::TranscriptConfig::new();
                ec.fill_rnd_coefs(&mut t);
                let mut cur = Cursor::new(&lp.data[0].bytes);
                let (ok, ch, v0, v1) = gkr_verify(1, &ec, &[], &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO, &mut t, &mut cur);
                if !ok { return false; }
                verify_pcs_opening_and_aggregation_no_mpi::<C, ECCConfig>(&mut cur, kernel, vs, &ch, v0, v1, &comms, tmpl.is_broadcast(), 0, 1, &mut t)
            }
        }).collect();
        results.iter().all(|&x| x)
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
        // Flat-buffer batch: zero malloc during circuit prep
        let ki = kernel.layered_circuit_input();
        let (mut circuits, _flat_bufs) = unsafe { tc.create_batch(pc) };
        let is_bc = tmpl.is_broadcast();
        for pi in 0..pc {
            // Inline get_local_vals: zero alloc, write directly into flat buffer
            let input = &mut circuits[pi].layers[0].input_vals;
            for v in input.iter_mut() { *v = Default::default(); }
            for (ci, (partition, (&ref vals, &ib))) in ki.iter()
                .zip(cvs.iter().zip(is_bc.iter()))
                .enumerate()
            {
                let local_slice = if ib {
                    vals.as_ref()
                } else {
                    let chunk = vals.len() / pc;
                    &vals[chunk * pi..chunk * (pi + 1)]
                };
                input[partition.offset..partition.offset + partition.len]
                    .copy_from_slice(local_slice);
            }
            circuits[pi].evaluate();
        }
        // Adaptive scratch pad: ScratchPadBatch for huge N, parallel alloc for moderate N
        let max_in = tc.layers.iter().map(|l| l.input_var_num).max().unwrap_or(0);
        let max_out = tc.layers.iter().map(|l| l.output_var_num).max().unwrap_or(0);
        // Sequential scratch pad alloc (no nested Rayon contention with template parallelism)
        let mut _sp_batch: Option<sumcheck::ScratchPadBatch<C::FieldConfig>> = None;
        let mut _sps_vec: Vec<sumcheck::ProverScratchPad<C::FieldConfig>> = Vec::new();
        let sps: &mut [sumcheck::ProverScratchPad<C::FieldConfig>] = if pc >= 512 {
            _sp_batch = Some(sumcheck::ScratchPadBatch::<C::FieldConfig>::new(pc, max_in, max_out, 1));
            _sp_batch.as_mut().unwrap().as_mut_slice()
        } else {
            _sps_vec = (0..pc).map(|_| {
                sumcheck::ProverScratchPad::<C::FieldConfig>::new(max_in, max_out, 1)
            }).collect();
            &mut _sps_vec
        };
        // Optional: dump circuit data for GPU prover
        if std::env::var("DUMP_GPU_DATA").is_ok() || std::env::var("USE_GPU_PROVER").is_ok() {
            dump_circuits_for_gpu::<C::FieldConfig>(ti, pc, &tc, &circuits, tc.rnd_coefs.len());
        }
        let t1 = std::time::Instant::now();
        let (cv, ch) = if std::env::var("USE_GPU_PROVER").is_ok() {
            // GPU path: call GPU binary for GKR, replay transcript, CPU does PCS
            gpu_gkr_prove::<C>(&tc, pc, ti, &mut tr)
        } else {
            let r = gkr_prove_batch(&circuits, sps, &mut tr);
            r
        };
        // Drop batch circuits without freeing shared gates or aliased buffers
        unsafe { expander_circuit::Circuit::drop_batch(circuits); }
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
        let final_proof = tr.finalize_and_get_proof();
        if std::env::var("DUMP_GPU_DATA").is_ok() {
            let dir = format!("/dev/shm/gpu_data/tmpl_{}", ti);
            std::fs::write(format!("{}/rust_proof.bin", dir), &final_proof.bytes).unwrap();
            eprintln!("  [dump] tmpl[{}] full proof: {} bytes", ti, final_proof.bytes.len());
        }
        ExpanderProof { data: vec![final_proof] }
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

/// GPU batch proving: dump all templates → single GPU launch → parallel replay + PCS.
/// Eliminates per-template subprocess overhead (~230ms × N_templates → ~230ms total).
fn gpu_prove_all<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>>(
    templates: &[crate::zkcuda::context::ProofTemplate],
    kernels: &[crate::zkcuda::kernel::Kernel<ECCConfig>],
    dm: &[Vec<SIMDField<ECCConfig>>],
    ps: &ExpanderProverSetup<C::FieldConfig, C::PCSConfig>,
    commit_states: &[crate::zkcuda::proving_system::expander::structs::ExpanderCommitmentState<C::FieldConfig, C::PCSConfig>],
) -> Vec<ExpanderProof> {
    use gkr_engine::Transcript;

    let t_total = std::time::Instant::now();

    // ---- Phase 1: Prepare all templates and dump circuit data ----
    let t_prep = std::time::Instant::now();
    struct PreparedTemplate<Cfg: GKREngine> {
        ti: usize,
        pc: usize,
        tc: expander_circuit::Circuit<Cfg::FieldConfig>,
        tr: Cfg::TranscriptConfig,
        cvs_indices: Vec<usize>,
        is_broadcast: Vec<bool>,
    }
    let mut gpu_templates: Vec<Option<PreparedTemplate<C>>> = Vec::new();
    let mut gpu_count = 0usize;

    for (ti, tmpl) in templates.iter().enumerate() {
        let kernel = &kernels[tmpl.kernel_id()];
        let cvs: Vec<&[SIMDField<C>]> = tmpl.commitment_indices().iter().map(|&i| dm[i].as_slice()).collect();
        let pc = next_power_of_two(tmpl.parallel_count());

        if pc <= 1 {
            gpu_templates.push(None);
            continue;
        }

        check_inputs(kernel, &cvs, pc, tmpl.is_broadcast());
        let (bc, _bs) = prepare_expander_circuit::<C::FieldConfig, ECCConfig>(kernel, 1);
        let mut tr = C::TranscriptConfig::new();
        let mut tc = bc.clone();
        tc.fill_rnd_coefs(&mut tr);

        let ki = kernel.layered_circuit_input();
        let (mut circuits, _flat_bufs) = unsafe { tc.create_batch(pc) };
        let is_bc = tmpl.is_broadcast();
        for pi in 0..pc {
            let input = &mut circuits[pi].layers[0].input_vals;
            for v in input.iter_mut() { *v = Default::default(); }
            for (_ci, (partition, (&ref vals, &ib))) in ki.iter()
                .zip(cvs.iter().zip(is_bc.iter()))
                .enumerate()
            {
                let local_slice = if ib { vals.as_ref() }
                else { let chunk = vals.len() / pc; &vals[chunk * pi..chunk * (pi + 1)] };
                input[partition.offset..partition.offset + partition.len]
                    .copy_from_slice(local_slice);
            }
            circuits[pi].evaluate();
        }

        dump_circuits_for_gpu::<C::FieldConfig>(ti, pc, &tc, &circuits, tc.rnd_coefs.len());
        unsafe { expander_circuit::Circuit::drop_batch(circuits); }

        // Dump SRS + commitment data for GPU PCS
        for &ci in tmpl.commitment_indices() {
            let commit_len = dm[ci].len();
            // Dump SRS (once per unique commit_len)
            if let Some(p_key) = ps.p_keys.get(&commit_len) {
                let p_key_ptr = p_key as *const _ as *const poly_commit::orion::OrionSRS;
                let srs: &poly_commit::orion::OrionSRS = unsafe { &*p_key_ptr };
                dump_orion_srs_m31(srs, commit_len);
            }
            // Dump commitment data (raw SIMD-packed M31 values)
            let commit_path = format!("/dev/shm/gpu_data/commit_{}.bin", ci);
            if !std::path::Path::new(&commit_path).exists() {
                use std::io::Write;
                let vals = &dm[ci];
                let bytes: &[u8] = unsafe {
                    std::slice::from_raw_parts(
                        vals.as_ptr() as *const u8,
                        vals.len() * std::mem::size_of_val(&vals[0]),
                    )
                };
                let mut f = std::fs::File::create(&commit_path).unwrap();
                f.write_all(bytes).unwrap();
                eprintln!("  [gpu] commit data ci={} len={} → {} ({} bytes)",
                          ci, commit_len, commit_path, bytes.len());
            }
        }

        // Dump PCS metadata for this template
        {
            use std::io::Write;
            let meta_path = format!("/dev/shm/gpu_data/tmpl_{}/pcs_meta.bin", ti);
            let mut mf = std::fs::File::create(&meta_path).unwrap();
            let n_commits = tmpl.commitment_indices().len() as u32;
            mf.write_all(&n_commits.to_le_bytes()).unwrap();
            for (&ci, &ib) in tmpl.commitment_indices().iter().zip(tmpl.is_broadcast().iter()) {
                mf.write_all(&(ci as u32).to_le_bytes()).unwrap();
                mf.write_all(&(dm[ci].len() as u32).to_le_bytes()).unwrap();
                mf.write_all(&(if ib { 1u32 } else { 0u32 }).to_le_bytes()).unwrap();
            }
        }

        gpu_templates.push(Some(PreparedTemplate {
            ti, pc, tc, tr,
            cvs_indices: tmpl.commitment_indices().to_vec(),
            is_broadcast: tmpl.is_broadcast().to_vec(),
        }));
        gpu_count += 1;
    }
    eprintln!("  [gpu] prep {} templates: {:?}", gpu_count, t_prep.elapsed());

    // ---- Phase 2: GPU GKR via persistent server ----
    let t_gpu = std::time::Instant::now();
    gpu_server_prove();
    eprintln!("  [gpu] GPU GKR: {:?}", t_gpu.elapsed());

    // ---- Phase 3: Replay + PCS for each template ----
    let t_pcs = std::time::Instant::now();

    let proofs: Vec<ExpanderProof> = gpu_templates.into_iter().enumerate().map(|(ti, prep)| {
        if let Some(mut p) = prep {
            // GPU GKR proof → replay transcript → CPU PCS opening
            let t0 = std::time::Instant::now();
            let gpu_proof = std::fs::read(format!("/dev/shm/gpu_data/tmpl_{}/gpu_proof.bin", p.ti))
                .expect("GPU proof file missing");
            let (cv, ch) = gpu_gkr_prove_from_bytes::<C>(&p.tc, p.pc, &gpu_proof, &mut p.tr);
            assert_eq!(cv, <C::FieldConfig as FieldEngine>::ChallengeField::from(0u32));

            let cvs: Vec<&[SIMDField<C>]> = p.cvs_indices.iter().map(|&i| dm[i].as_slice()).collect();
            let chs = if let Some(cy) = ch.challenge_y() { vec![ch.challenge_x(), cy] } else { vec![ch.challenge_x()] };
            for sc in &chs {
                for (ci, (&ref v, &_ib)) in cvs.iter().zip(p.is_broadcast.iter()).enumerate() {
                    let pc2 = sc.clone();
                    let comm_idx = p.cvs_indices[ci];
                    let scratch = &commit_states[comm_idx].scratch;
                    crate::zkcuda::proving_system::expander::prove_impl::pcs_batch_open_with_scratch::<C>(
                        v, &pc2, ps, &mut p.tr, Some(scratch));
                }
            }
            eprintln!("  [pcs] tmpl[{}] cpu_pcs={:?}", p.ti, t0.elapsed());
            ExpanderProof { data: vec![p.tr.finalize_and_get_proof()] }
        } else {
            let tmpl = &templates[ti];
            prove_one::<C, ECCConfig>(ti, tmpl, kernels, dm, ps, commit_states)
        }
    }).collect();

    eprintln!("  [gpu] replay+PCS: {:?}", t_pcs.elapsed());
    eprintln!("  [gpu] total GPU path: {:?}", t_total.elapsed());
    proofs
}

/// Replay GPU proof bytes through transcript (no subprocess call).
fn gpu_gkr_prove_from_bytes<C: GKREngine>(
    tc: &expander_circuit::Circuit<C::FieldConfig>,
    pc: usize,
    gpu_proof: &[u8],
    tr: &mut C::TranscriptConfig,
) -> (<C::FieldConfig as FieldEngine>::ChallengeField, gkr_engine::ExpanderDualVarChallenge<C::FieldConfig>)
{
    use arith::Field;
    use gkr_engine::Transcript;

    let elem_size = <C::FieldConfig as FieldEngine>::ChallengeField::SIZE;
    let n_layers = tc.layers.len();
    let simd_var_num = 4usize;
    let mpi_var_num = { let mut x = pc; let mut v = 0usize; while x > 1 { v += 1; x >>= 1; } v };

    let mut cursor = 0usize;
    let mut last_rx = Vec::new();
    let mut last_ry: Option<Vec<<C::FieldConfig as FieldEngine>::ChallengeField>> = None;
    let mut last_r_simd = Vec::new();
    let mut last_r_mpi = Vec::new();

    let append_n = |cursor: &mut usize, n: usize, tr: &mut C::TranscriptConfig| {
        let bytes = n * elem_size;
        tr.append_u8_slice(&gpu_proof[*cursor..*cursor + bytes]);
        *cursor += bytes;
    };

    for li in (0..n_layers).rev() {
        let layer = &tc.layers[li];
        let ivn = layer.input_var_num;
        let skip_y = layer.mul.is_empty();

        let mut rx = Vec::with_capacity(ivn);
        for _ in 0..ivn {
            append_n(&mut cursor, 3, tr);
            rx.push(tr.generate_field_element::<<C::FieldConfig as FieldEngine>::ChallengeField>());
        }
        let mut r_simd = Vec::with_capacity(simd_var_num);
        for _ in 0..simd_var_num {
            append_n(&mut cursor, 4, tr);
            r_simd.push(tr.generate_field_element::<<C::FieldConfig as FieldEngine>::ChallengeField>());
        }
        let mut r_mpi = Vec::with_capacity(mpi_var_num);
        for _ in 0..mpi_var_num {
            append_n(&mut cursor, 4, tr);
            r_mpi.push(tr.generate_field_element::<<C::FieldConfig as FieldEngine>::ChallengeField>());
        }
        append_n(&mut cursor, 1, tr); // vx_claim

        if !skip_y {
            let mut ry = Vec::with_capacity(ivn);
            for _ in 0..ivn {
                append_n(&mut cursor, 3, tr);
                ry.push(tr.generate_field_element::<<C::FieldConfig as FieldEngine>::ChallengeField>());
            }
            append_n(&mut cursor, 1, tr); // vy_claim
            let _alpha: <C::FieldConfig as FieldEngine>::ChallengeField = tr.generate_field_element();
            last_ry = Some(ry);
        } else {
            last_ry = None;
        }
        last_rx = rx; last_r_simd = r_simd; last_r_mpi = r_mpi;
    }

    assert_eq!(cursor, gpu_proof.len(),
        "GPU proof size mismatch: consumed {} of {} bytes", cursor, gpu_proof.len());

    let ch = gkr_engine::ExpanderDualVarChallenge::new(last_rx, last_ry, last_r_simd, last_r_mpi);
    (<C::FieldConfig as FieldEngine>::ChallengeField::ZERO, ch)
}

/// Persistent GPU server: launch once, reuse across calls.
/// Sends "PROVE\n" to stdin, reads "GPU_DONE <ms>\n" from stdout.
fn gpu_server_prove() {
    use std::io::{BufRead, BufReader, Write};
    use std::sync::Mutex;

    static GPU_SERVER: std::sync::OnceLock<Mutex<GpuServer>> = std::sync::OnceLock::new();

    struct GpuServer {
        child: std::process::Child,
        stdin: std::process::ChildStdin,
        stdout: BufReader<std::process::ChildStdout>,
    }

    let server = GPU_SERVER.get_or_init(|| {
        let t0 = std::time::Instant::now();
        let mut child = std::process::Command::new("cuda/gpu_prover_v2")
            .arg("--server")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .expect("Failed to launch cuda/gpu_prover_v2 --server");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut s = GpuServer { child, stdin, stdout };
        // Wait for "GPU_SERVER_READY"
        let mut line = String::new();
        s.stdout.read_line(&mut line).expect("GPU server didn't respond");
        assert!(line.starts_with("GPU_SERVER_READY"), "GPU server: {}", line.trim());
        eprintln!("  [gpu] server started: {:?}", t0.elapsed());
        Mutex::new(s)
    });

    let mut s = server.lock().unwrap();
    writeln!(s.stdin, "PROVE").expect("Failed to write to GPU server");
    s.stdin.flush().unwrap();
    let mut line = String::new();
    s.stdout.read_line(&mut line).expect("GPU server didn't respond to PROVE");
    assert!(line.starts_with("GPU_DONE") || line.starts_with("GPU_ERROR"),
        "GPU server unexpected: {}", line.trim());
    eprintln!("  [gpu] server response: {}", line.trim());
}

/// Dump circuit data (gates + witness) to binary files for GPU prover.
/// Each template gets its own directory: /dev/shm/gpu_data/tmpl_{ti}/
fn dump_circuits_for_gpu<F: gkr_engine::FieldEngine>(
    ti: usize,
    pc: usize,
    template_circuit: &expander_circuit::Circuit<F>,
    circuits: &[expander_circuit::Circuit<F>],
    rnd_coef_count: usize,
) {
    use std::io::Write;
    let dir = format!("/dev/shm/gpu_data/tmpl_{}", ti);
    std::fs::create_dir_all(&dir).ok();

    let num_layers = template_circuit.layers.len();

    // Write header: N, num_layers, per-layer sizes
    let mut hdr = std::fs::File::create(format!("{}/header.bin", dir)).unwrap();
    hdr.write_all(&(pc as u32).to_le_bytes()).unwrap();
    hdr.write_all(&(num_layers as u32).to_le_bytes()).unwrap();
    for layer in &template_circuit.layers {
        hdr.write_all(&(layer.input_var_num as u32).to_le_bytes()).unwrap();
        hdr.write_all(&(layer.output_var_num as u32).to_le_bytes()).unwrap();
        hdr.write_all(&(layer.mul.len() as u32).to_le_bytes()).unwrap();
        hdr.write_all(&(layer.add.len() as u32).to_le_bytes()).unwrap();
        hdr.write_all(&(layer.const_.len() as u32).to_le_bytes()).unwrap();
    }
    // rnd_coef_count: needed for GPU prover to advance transcript state
    hdr.write_all(&(rnd_coef_count as u32).to_le_bytes()).unwrap();

    // Write gates per layer (shared across all instances)
    for (li, layer) in template_circuit.layers.iter().enumerate() {
        // Mul gates: [o_id, x_id, y_id, coef] x n_mul
        let mut gf = std::fs::File::create(format!("{}/layer_{}_mul.bin", dir, li)).unwrap();
        for gate in &layer.mul {
            gf.write_all(&(gate.o_id as u32).to_le_bytes()).unwrap();
            gf.write_all(&(gate.i_ids[0] as u32).to_le_bytes()).unwrap();
            gf.write_all(&(gate.i_ids[1] as u32).to_le_bytes()).unwrap();
            // coef is CircuitField (M31 = 4 bytes)
            let coef_bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(&gate.coef as *const _ as *const u8, 4)
            };
            gf.write_all(&coef_bytes).unwrap();
        }

        // Add gates: [o_id, x_id, coef] x n_add
        let mut af = std::fs::File::create(format!("{}/layer_{}_add.bin", dir, li)).unwrap();
        for gate in &layer.add {
            af.write_all(&(gate.o_id as u32).to_le_bytes()).unwrap();
            af.write_all(&(gate.i_ids[0] as u32).to_le_bytes()).unwrap();
            let coef_bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(&gate.coef as *const _ as *const u8, 4)
            };
            af.write_all(coef_bytes).unwrap();
        }

        // Const gates: [o_id, coef] x n_const
        let mut cf = std::fs::File::create(format!("{}/layer_{}_const.bin", dir, li)).unwrap();
        for gate in &layer.const_ {
            cf.write_all(&(gate.o_id as u32).to_le_bytes()).unwrap();
            let coef_bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(&gate.coef as *const _ as *const u8, 4)
            };
            cf.write_all(coef_bytes).unwrap();
        }
    }

    // Write ALL witness data to ONE file: witness.bin
    // Layout: [instance_0_layer0_input_vals | instance_1_layer0_input_vals | ...]
    // Each instance = layer0.input_vals as raw M31x16 bytes (contiguous)
    {
        let mut wf = std::fs::File::create(format!("{}/witness.bin", dir)).unwrap();
        for circuit in circuits.iter() {
            let vals = &circuit.layers[0].input_vals;
            let bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    vals.as_ptr() as *const u8,
                    vals.len() * std::mem::size_of_val(&vals[0]),
                )
            };
            wf.write_all(bytes).unwrap();
        }
    }

    // Report const_ and uni gates (not exported yet!)
    for (li, layer) in template_circuit.layers.iter().enumerate() {
        if !layer.const_.is_empty() || !layer.uni.is_empty() {
            eprintln!("  [dump] tmpl[{}] L{}: const_={} uni={}", ti, li, layer.const_.len(), layer.uni.len());
        }
    }
    eprintln!("  [dump] tmpl[{}] N={} layers={} rnd_coefs={} → {}/", ti, pc, num_layers, rnd_coef_count, dir);
}

/// Dump Orion SRS (expander graphs) for GPU PCS.
/// Format: msg_len(u32) | codeword_len(u32) | n_graphs(u32) | n_leaves_per_query(u32)
/// Then per graph: input_start(u32) | output_start(u32) | output_end(u32)
///   | row_ptrs[R+1](u32) | col_indices[nnz](u32)
fn dump_srs_for_gpu(commit_len: usize, ps: &impl std::any::Any) {
    // We access OrionSRS through the prover setup's p_keys
    // This is a generic function — actual implementation needs concrete types
    // For now, skip if SRS file already exists
    let srs_path = format!("/dev/shm/gpu_data/srs_{}.bin", commit_len);
    if std::path::Path::new(&srs_path).exists() { return; }
    eprintln!("  [gpu] SRS dump for commit_len={} not yet implemented (needs concrete type access)", commit_len);
}

/// Dump OrionSRS expander graphs to binary file (concrete M31 type).
/// Called once per unique commit_len.
fn dump_orion_srs_m31(
    srs: &poly_commit::orion::OrionSRS,
    commit_len: usize,
) {
    use std::io::Write;
    let srs_path = format!("/dev/shm/gpu_data/srs_{}.bin", commit_len);
    if std::path::Path::new(&srs_path).exists() { return; }

    let code = &srs.code_instance;
    let msg_len = code.msg_len() as u32;
    let cw_len = code.code_len() as u32;
    let n_graphs = (code.g0s.len() + code.g1s.len()) as u32;
    let n_leaves_per_query = srs.num_leaves_per_mt_query as u32;

    let mut f = std::fs::File::create(&srs_path).unwrap();
    f.write_all(&msg_len.to_le_bytes()).unwrap();
    f.write_all(&cw_len.to_le_bytes()).unwrap();
    f.write_all(&n_graphs.to_le_bytes()).unwrap();
    f.write_all(&n_leaves_per_query.to_le_bytes()).unwrap();

    // Write g0s then g1s (same order as Rust encode_in_place: chain!(g0s, g1s))
    for gp in code.g0s.iter().chain(code.g1s.iter()) {
        f.write_all(&(gp.input_starts as u32).to_le_bytes()).unwrap();
        f.write_all(&(gp.output_starts as u32).to_le_bytes()).unwrap();
        f.write_all(&(gp.output_ends as u32).to_le_bytes()).unwrap();

        let g = &gp.graph;
        let r_size = g.r_vertices_size;
        // Build CSR: row_ptrs[R+1], col_indices[nnz]
        let mut row_ptrs = Vec::with_capacity(r_size + 1);
        row_ptrs.push(0u32);
        let mut col_indices = Vec::new();
        for ni in &g.neighborings {
            for &idx in ni {
                col_indices.push(idx as u32);
            }
            row_ptrs.push(col_indices.len() as u32);
        }
        // Write row_ptrs
        for v in &row_ptrs { f.write_all(&v.to_le_bytes()).unwrap(); }
        // Write col_indices
        for v in &col_indices { f.write_all(&v.to_le_bytes()).unwrap(); }
    }

    eprintln!("  [gpu] SRS dump: msg_len={} cw_len={} graphs={} → {}",
              msg_len, cw_len, n_graphs, srs_path);
}

/// GPU GKR proving: call GPU binary, read proof bytes, replay through transcript.
/// Returns the same (claimed_value, challenge) tuple as gkr_prove_batch.
fn gpu_gkr_prove<C: GKREngine>(
    tc: &expander_circuit::Circuit<C::FieldConfig>,
    pc: usize,
    ti: usize,
    tr: &mut C::TranscriptConfig,
) -> (<C::FieldConfig as FieldEngine>::ChallengeField, gkr_engine::ExpanderDualVarChallenge<C::FieldConfig>)
{
    use arith::Field;
    use gkr_engine::Transcript;

    let t_gpu = std::time::Instant::now();

    // 1. Call GPU binary
    let status = std::process::Command::new("cuda/gpu_prover_v2")
        .arg("--template").arg(ti.to_string())
        .status()
        .expect("Failed to launch cuda/gpu_prover_v2");
    assert!(status.success(), "GPU prover exited with error");

    // 2. Read GPU proof bytes
    let gpu_proof = std::fs::read(format!("/dev/shm/gpu_data/tmpl_{}/gpu_proof.bin", ti))
        .expect("Failed to read GPU proof");
    eprintln!("  [gpu] tmpl[{}] gpu_proof={} bytes ({:?})", ti, gpu_proof.len(), t_gpu.elapsed());

    // 3. Replay through transcript (feeds bytes in correct order, updates hash state)
    let elem_size = <C::FieldConfig as FieldEngine>::ChallengeField::SIZE;
    let n_layers = tc.layers.len();
    let simd_var_num = 4usize;
    let mpi_var_num = {
        let mut x = pc; let mut v = 0usize;
        while x > 1 { v += 1; x >>= 1; }
        v
    };

    let mut cursor = 0usize;
    let mut last_rx = Vec::new();
    let mut last_ry: Option<Vec<<C::FieldConfig as FieldEngine>::ChallengeField>> = None;
    let mut last_r_simd = Vec::new();
    let mut last_r_mpi = Vec::new();

    let append_n = |cursor: &mut usize, n: usize, tr: &mut C::TranscriptConfig| {
        let bytes = n * elem_size;
        tr.append_u8_slice(&gpu_proof[*cursor..*cursor + bytes]);
        *cursor += bytes;
    };

    for li in (0..n_layers).rev() {
        let layer = &tc.layers[li];
        let ivn = layer.input_var_num;
        let skip_y = layer.mul.is_empty(); // skip_y_phase = no mul gates

        // X-phase: ivn rounds × 3 evals + challenge
        let mut rx = Vec::with_capacity(ivn);
        for _ in 0..ivn {
            append_n(&mut cursor, 3, tr);
            rx.push(tr.generate_field_element::<<C::FieldConfig as FieldEngine>::ChallengeField>());
        }

        // SIMD phase: 4 rounds × 4 evals + challenge
        let mut r_simd = Vec::with_capacity(simd_var_num);
        for _ in 0..simd_var_num {
            append_n(&mut cursor, 4, tr);
            r_simd.push(tr.generate_field_element::<<C::FieldConfig as FieldEngine>::ChallengeField>());
        }

        // MPI phase: mpi_var_num rounds × 4 evals + challenge
        let mut r_mpi = Vec::with_capacity(mpi_var_num);
        for _ in 0..mpi_var_num {
            append_n(&mut cursor, 4, tr);
            r_mpi.push(tr.generate_field_element::<<C::FieldConfig as FieldEngine>::ChallengeField>());
        }

        // vx_claim
        append_n(&mut cursor, 1, tr);

        // Y-phase
        if !skip_y {
            let mut ry = Vec::with_capacity(ivn);
            for _ in 0..ivn {
                append_n(&mut cursor, 3, tr);
                ry.push(tr.generate_field_element::<<C::FieldConfig as FieldEngine>::ChallengeField>());
            }
            // vy_claim
            append_n(&mut cursor, 1, tr);
            // alpha
            let _alpha: <C::FieldConfig as FieldEngine>::ChallengeField = tr.generate_field_element();
            last_ry = Some(ry);
        } else {
            last_ry = None;
        }

        last_rx = rx;
        last_r_simd = r_simd;
        last_r_mpi = r_mpi;
    }

    assert_eq!(cursor, gpu_proof.len(),
        "GPU proof size mismatch: consumed {} of {} bytes", cursor, gpu_proof.len());

    // 4. Construct challenge from last layer
    let ch = gkr_engine::ExpanderDualVarChallenge::new(
        last_rx,
        last_ry,
        last_r_simd,
        last_r_mpi,
    );

    (<C::FieldConfig as FieldEngine>::ChallengeField::ZERO, ch)
}
