use arith::Field;
use expander_circuit::Circuit;
use gkr::gkr_prove;
use gkr_engine::{ExpanderDualVarChallenge, FieldEngine, GKREngine, MPIConfig};
use sumcheck::ProverScratchPad;

use crate::{frontend::Config, zkcuda::kernel::{Kernel, LayeredCircuitInputVec}};

/// ECCCircuit -> ExpanderCircuit
/// Returns an additional prover scratch pad for later use in GKR.
pub fn prepare_expander_circuit<C, ECCConfig> (
    kernel: &Kernel<ECCConfig>,
    mpi_world_size: usize,
) -> (
    Circuit<C::FieldConfig>,
    ProverScratchPad<C::FieldConfig>
)
where 
    C: GKREngine,
    ECCConfig: Config<FieldConfig = C::FieldConfig>,
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField> 
{
    let mut expander_circuit = kernel.layered_circuit.export_to_expander().flatten::<C>();
    expander_circuit.pre_process_gkr::<C>();
    let (max_num_input_var, max_num_output_var) = super::utils::max_n_vars(&expander_circuit);
    let prover_scratch =
        ProverScratchPad::<C::FieldConfig>::new(max_num_input_var, max_num_output_var, mpi_world_size);
    
    (expander_circuit, prover_scratch)
}

/// Global values consist of several components, each of which can be either broadcasted or partitioned.
/// If it is broadcasted, the same value is used across all parallel instances. 
///   i.e. global_vals[i] is the same for all parallel instances.
/// If it is partitioned, each parallel instance gets a slice of the values.
///   i.e. global_vals[i] is partitioned equally into parallel_num slices, and each
///     parallel instance gets one slice.
/// 
/// This function returns the local values for each parallel instance based on the global values and the broadcast information.
pub fn get_local_vals<'vals_life, F: Field>(
    global_vals: &'vals_life [impl AsRef<[F]>],
    is_broadcast: &[bool],
    parallel_index: usize,
    parallel_num: usize,
) -> Vec<&'vals_life [F]> {
    global_vals
        .iter()
        .zip(is_broadcast.iter())
        .map(|(vals, is_broadcast)| {
            if *is_broadcast {
                &vals.as_ref()
            } else {
                let local_val_len = vals.as_ref().len() / parallel_num;
                &vals.as_ref()[local_val_len * parallel_index..local_val_len * (parallel_num + 1)]
            }
        })
        .collect::<Vec<_>>()
}

/// Local values consist of several components, whose location and length are specified by `partition_info`.
/// This function targets at relocating the local values into a single vector based on the partition information.
pub fn prepare_inputs_with_local_vals<F: Field>(
    input_len: usize,
    partition_info: &[LayeredCircuitInputVec],
    local_commitment_values: &[impl AsRef<[F]>],
) -> Vec<F> {
    let mut input_vals = vec![F::ZERO; input_len];
    for (partition, val) in partition_info.iter().zip(local_commitment_values.iter()) {
        assert!(partition.len == val.as_ref().len());
        input_vals[partition.offset..partition.offset + partition.len]
            .copy_from_slice(val.as_ref());
    }
    input_vals
}

pub fn prove_gkr_with_local_vals<C: GKREngine>(
    expander_circuit: &mut Circuit<C::FieldConfig>,
    prover_scratch: &mut ProverScratchPad<C::FieldConfig>,
    local_commitment_values: &[impl AsRef<[<C::FieldConfig as FieldEngine>::SimdCircuitField]>],
    partition_info: &[LayeredCircuitInputVec],
    transcript: &mut C::TranscriptConfig,
    mpi_config: &MPIConfig,
) -> ExpanderDualVarChallenge<C::FieldConfig>
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    expander_circuit.layers[0].input_vals = prepare_inputs_with_local_vals(
        1 << expander_circuit.log_input_size(),
        partition_info,
        local_commitment_values,
    );
    expander_circuit.fill_rnd_coefs(transcript);
    expander_circuit.evaluate();
    let (claimed_v, challenge) = gkr_prove(
        &expander_circuit,
        prover_scratch,
        transcript,
        &mpi_config,
    );
    assert_eq!(
        claimed_v,
        <C::FieldConfig as FieldEngine>::ChallengeField::from(0)
    );
    challenge
}