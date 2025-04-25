use expander_compiler::circuit::config::Config;
use expander_compiler::frontend::M31Config;
use expander_compiler::zkcuda::proving_system::callee_utils::{
    read_commit_vals_from_shared_memory, read_selected_pkey_from_shared_memory,
    write_commitment_extra_info_to_shared_memory, write_commitment_to_shared_memory,
};
use expander_compiler::zkcuda::proving_system::{
    ExpanderGKRCommitment, ExpanderGKRCommitmentExtraInfo,
};

use gkr_engine::{ExpanderPCS, MPIConfig, MPIEngine};
use polynomials::MultiLinearPoly;

macro_rules! field {
    ($config: ident) => {
        $config::FieldConfig
    };
}

macro_rules! pcs {
    ($config: ident) => {
        $config::PCSConfig
    };
}

fn commit<C: Config>() {
    let mpi_config = MPIConfig::prover_new();
    let world_rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    assert!(
        world_size > 1,
        "In case world_size is 1, we should not use the mpi version of the prover"
    );
    if world_rank == 0 {
        println!("Expander Commit Exec Called with world size {}", world_size);
    }

    let (local_val_len, p_key) = read_selected_pkey_from_shared_memory::<C>();

    // TODO: remove the redundancy
    let global_vals_to_commit = read_commit_vals_from_shared_memory::<C>();
    let local_vals_to_commit = global_vals_to_commit
        [local_val_len * world_rank..local_val_len * (world_rank + 1)]
        .to_vec();
    drop(global_vals_to_commit);

    let params = <pcs!(C) as ExpanderPCS<field!(C)>>::gen_params(local_val_len.ilog2() as usize);

    let mut scratch = <pcs!(C) as ExpanderPCS<field!(C)>>::init_scratch_pad(&params, &mpi_config);

    let commitment = <pcs!(C) as ExpanderPCS<field!(C)>>::commit(
        &params,
        &mpi_config,
        &p_key,
        &MultiLinearPoly::new(local_vals_to_commit),
        &mut scratch,
    );

    if world_rank == 0 {
        let commitment = ExpanderGKRCommitment {
            vals_len: local_val_len,
            commitment: vec![commitment.unwrap()],
        };
        let extra_info = ExpanderGKRCommitmentExtraInfo {
            scratch: vec![scratch],
        };

        write_commitment_to_shared_memory::<C>(&commitment);
        write_commitment_extra_info_to_shared_memory::<C>(&extra_info);
    }

    MPIConfig::finalize();
}

fn main() {
    // TODO: Add command line argument parsing
    commit::<M31Config>();
}
