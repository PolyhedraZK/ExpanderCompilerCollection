use expander_compiler::circuit::config::Config;
use expander_compiler::frontend::M31Config;
use expander_compiler::zkcuda::proving_system::callee_utils::{
    read_commit_vals_from_shared_memory, read_selected_pkey_from_shared_memory,
    write_commitment_extra_info_to_shared_memory, write_commitment_to_shared_memory,
};
use expander_config::GKRConfig;
use mpi_config::MPIConfig;
use poly_commit::PCSForExpanderGKR;
use polynomials::MultiLinearPoly;

macro_rules! field {
    ($config: ident) => {
        $config::DefaultGKRFieldConfig
    };
}

macro_rules! transcript {
    ($config: ident) => {
        <$config::DefaultGKRConfig as GKRConfig>::Transcript
    };
}

macro_rules! pcs {
    ($config: ident) => {
        <$config::DefaultGKRConfig as GKRConfig>::PCS
    };
}

fn commit<C: Config>() {
    let mpi_config = MPIConfig::new();
    let rank = mpi_config.world_rank();
    let world_size = mpi_config.world_size();
    assert!(
        world_size > 1,
        "In case world_size is 1, we should not use the mpi version of the prover"
    );
    if rank == 0 {
        println!("Expander Commit Exec Called with world size {}", world_size);
    }

    let (local_val_len, p_key) = read_selected_pkey_from_shared_memory::<C>();

    // TODO: remove the redundancy
    let global_vals_to_commit = read_commit_vals_from_shared_memory::<C>();
    let local_vals_to_commit =
        global_vals_to_commit[local_val_len * rank..local_val_len * (rank + 1)].to_vec();
    drop(global_vals_to_commit);

    let params = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::gen_params(
        local_val_len.ilog2() as usize,
    );

    let mut scratch = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::init_scratch_pad(
        &params,
        &mpi_config,
    );

    let commitment = <pcs!(C) as PCSForExpanderGKR<field!(C), transcript!(C)>>::commit(
        &params,
        &mpi_config,
        &p_key,
        &MultiLinearPoly::new(local_vals_to_commit),
        &mut scratch,
    )
    .unwrap();

    write_commitment_to_shared_memory::<C>(&commitment);
    write_commitment_extra_info_to_shared_memory::<C>(&scratch);
}

fn main() {
    // TODO: Add command line argument parsing
    commit::<M31Config>();
}
