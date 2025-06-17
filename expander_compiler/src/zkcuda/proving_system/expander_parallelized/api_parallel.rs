use std::fs;
use std::io::{Cursor, Read};

use crate::circuit::config::Config;
use crate::utils::misc::next_power_of_two;
use crate::zkcuda::kernel::Kernel;
use crate::zkcuda::proof::ComputationGraph;
use crate::zkcuda::proving_system::expander::structs::{
    ExpanderCommitment, ExpanderProof, ExpanderVerifierSetup,
};
use crate::zkcuda::proving_system::expander_parallelized::client::{
    request_exit, request_prove, request_setup,
};
use crate::zkcuda::proving_system::expander_parallelized::cmd_utils::start_server;
use crate::zkcuda::proving_system::expander_parallelized::shared_memory_utils::SharedMemoryEngine;
use crate::zkcuda::proving_system::{CombinedProof, Commitment, ProvingSystem};

use super::super::Expander;
use super::server_utils::{SERVER_IP, SERVER_PORT};
use arith::Field;
use expander_utils::timer::Timer;

use gkr::gkr_verify;
use gkr_engine::{ExpanderPCS, ExpanderSingleVarChallenge, FieldEngine, GKREngine, Transcript};
use polynomials::EqPolynomial;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use reqwest::Client;
use serdes::ExpSerde;

pub struct ParallelizedExpander<C: GKREngine> {
    _config: std::marker::PhantomData<C>,
}
pub fn parse_port_number() -> u16 {
    let mut port = SERVER_PORT.lock().unwrap();
    *port = std::env::var("PORT_NUMBER")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(*port);
    *port
}

impl<C: GKREngine> ParallelizedExpander<C>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    fn verify_kernel<ECCConfig: Config<FieldConfig = C::FieldConfig>>(
        verifier_setup: &ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        kernel: &Kernel<ECCConfig>,
        proof: &ExpanderProof,
        commitments: &[&ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>],
        parallel_count: usize,
        is_broadcast: &[bool],
    ) -> bool {
        let timer = Timer::new("verify", true);
        let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten::<C>();
        expander_circuit.pre_process_gkr::<C>();

        let mut transcript = C::TranscriptConfig::new();
        expander_circuit.fill_rnd_coefs(&mut transcript);

        let mut cursor = Cursor::new(&proof.data[0].bytes);
        let (mut verified, challenge, claimed_v0, claimed_v1) = gkr_verify(
            parallel_count,
            &expander_circuit,
            &[],
            &<C::FieldConfig as FieldEngine>::ChallengeField::ZERO,
            &mut transcript,
            &mut cursor,
        );

        if !verified {
            println!("Failed to verify GKR proof");
            return false;
        }

        verified &= Self::verify_input_claim::<ECCConfig>(
            &mut cursor,
            kernel,
            verifier_setup,
            &challenge.challenge_x(),
            &claimed_v0,
            commitments,
            is_broadcast,
            parallel_count,
            &mut transcript,
        );
        if let Some(challenge_y) = challenge.challenge_y() {
            verified &= Self::verify_input_claim::<ECCConfig>(
                &mut cursor,
                kernel,
                verifier_setup,
                &challenge_y,
                &claimed_v1.unwrap(),
                commitments,
                is_broadcast,
                parallel_count,
                &mut transcript,
            );
        }
        if !verified {
            println!("Failed to verify overall pcs");
            return false;
        }
        timer.stop();
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn verify_input_claim<ECCConfig: Config<FieldConfig = C::FieldConfig>>(
        mut proof_reader: impl Read,
        kernel: &Kernel<ECCConfig>,
        v_keys: &ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>,
        challenge: &ExpanderSingleVarChallenge<C::FieldConfig>,
        y: &<C::FieldConfig as FieldEngine>::ChallengeField,
        commitments: &[&ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig>],
        is_broadcast: &[bool],
        parallel_count: usize,
        transcript: &mut C::TranscriptConfig,
    ) -> bool {
        let mut target_y = <C::FieldConfig as FieldEngine>::ChallengeField::ZERO;
        for ((input, commitment), ib) in kernel
            .layered_circuit_input
            .iter()
            .zip(commitments.iter())
            .zip(is_broadcast)
        {
            let val_len =
                <ExpanderCommitment<C::PCSField, C::FieldConfig, C::PCSConfig> as Commitment<
                    ECCConfig,
                >>::vals_len(commitment);
            let (challenge_for_pcs, component_idx_vars) =
                get_challenge_for_pcs_with_mpi(challenge, val_len, parallel_count, *ib);

            let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
                val_len.ilog2() as usize,
                1,
            );
            let v_key = v_keys.v_keys.get(&val_len).unwrap();

            let claim = <C::FieldConfig as FieldEngine>::ChallengeField::deserialize_from(
                &mut proof_reader,
            )
            .unwrap();
            transcript.append_field_element(&claim);

            let opening =
                <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::Opening::deserialize_from(
                    &mut proof_reader,
                )
                .unwrap();

            transcript.lock_proof();
            let verified = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::verify(
                &params,
                v_key,
                &commitment.commitment,
                &challenge_for_pcs,
                claim,
                transcript,
                &opening,
            );
            transcript.unlock_proof();

            if !verified {
                println!("Failed to verify single pcs opening");
                return false;
            }

            let mut buffer = vec![];
            opening
                .serialize_into(&mut buffer)
                .expect("Failed to serialize opening");
            transcript.append_u8_slice(&buffer);

            let component_index = input.offset / input.len;
            let v_index = EqPolynomial::ith_eq_vec_elem(&component_idx_vars, component_index);

            target_y += v_index * claim;
        }

        // overall claim verification
        *y == target_y
    }
}

impl<C: GKREngine, ECCConfig: Config<FieldConfig = C::FieldConfig>> ProvingSystem<ECCConfig>
    for ParallelizedExpander<C>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    type ProverSetup = ExpanderProverSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type VerifierSetup = ExpanderVerifierSetup<C::PCSField, C::FieldConfig, C::PCSConfig>;
    type Proof = CombinedProof<ECCConfig, Expander<C>>;

    fn setup(
        computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
    ) -> (Self::ProverSetup, Self::VerifierSetup) {
        let setup_timer = Timer::new("setup", true);

        let mut bytes = vec![];
        computation_graph.serialize_into(&mut bytes).unwrap();
        // append current timestamp to the file name to avoid conflicts
        let setup_filename = format!(
            "/tmp/computation_graph_{}.bin",
            chrono::Utc::now().timestamp_millis()
        );
        fs::write(&setup_filename, bytes).expect("Failed to write computation graph to file");

        let max_parallel_count = computation_graph
            .proof_templates
            .iter()
            .map(|t| t.parallel_count)
            .max()
            .unwrap_or(1);

        // Keep trying until the server is ready
        let port = parse_port_number();
        let server_url = format!("{SERVER_IP}:{port}");
        start_server::<C>(
            "../target/release/expander_server",
            next_power_of_two(max_parallel_count),
            port,
        );
        loop {
            match wait_async(Client::new().get(format!("http://{server_url}/")).send()) {
                Ok(_) => break,
                Err(_) => std::thread::sleep(std::time::Duration::from_secs(1)),
            }
        }

        wait_async(request_setup(&setup_filename));

        setup_timer.stop();

        SharedMemoryEngine::read_pcs_setup_from_shared_memory()
    }

    fn prove(
        _prover_setup: &Self::ProverSetup,
        _computation_graph: &crate::zkcuda::proof::ComputationGraph<ECCConfig>,
        device_memories: &[crate::zkcuda::context::DeviceMemory<ECCConfig>],
    ) -> Self::Proof {
        let timer = Timer::new("prove", true);

        SharedMemoryEngine::write_witness_to_shared_memory::<C::FieldConfig>(
            &device_memories
                .iter()
                .map(|m| &m.values[..])
                .collect::<Vec<_>>(),
        );
        wait_async(request_prove());

        timer.stop();
        SharedMemoryEngine::read_proof_from_shared_memory()
    }

    fn verify(
        verifier_setup: &Self::VerifierSetup,
        computation_graph: &ComputationGraph<ECCConfig>,
        proof: &Self::Proof,
    ) -> bool {
        let verified = proof
            .proofs
            .par_iter()
            .zip(computation_graph.proof_templates.par_iter())
            .map(|(local_proof, template)| {
                let local_commitments = template
                    .commitment_indices
                    .iter()
                    .map(|idx| &proof.commitments[*idx])
                    .collect::<Vec<_>>();

                Self::verify_kernel(
                    verifier_setup,
                    &computation_graph.kernels[template.kernel_id],
                    local_proof,
                    &local_commitments,
                    next_power_of_two(template.parallel_count),
                    &template.is_broadcast,
                )
            })
            .collect::<Vec<_>>();

        verified.iter().all(|x| *x)
    }

    fn post_process() {
        wait_async(request_exit())
    }
}

/// Run an async function in a blocking context.
#[inline(always)]
fn wait_async<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(f)
}
