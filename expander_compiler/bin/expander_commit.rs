mod common;

use std::str::FromStr;

use clap::Parser;
use common::ExpanderExecArgs;
use expander_compiler::frontend::{
    BN254Config, BabyBearConfig, GF2Config, GoldilocksConfig, M31Config,
};
use expander_compiler::zkcuda::proving_system::callee_utils::{
    read_local_vals_to_commit_from_shared_memory, read_selected_pkey_from_shared_memory,
    write_commitment_extra_info_to_shared_memory, write_commitment_to_shared_memory,
};
use expander_compiler::zkcuda::proving_system::{
    ExpanderGKRCommitment, ExpanderGKRCommitmentExtraInfo,
};

use gkr::{BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use gkr_engine::{
    ExpanderPCS, FieldEngine, GKREngine, MPIConfig, MPIEngine, PolynomialCommitmentType,
};
use polynomials::RefMultiLinearPoly;

fn commit<C: GKREngine>()
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
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

    let (local_val_len, p_key) =
        read_selected_pkey_from_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>();

    let local_vals_to_commit =
        read_local_vals_to_commit_from_shared_memory::<C::FieldConfig>(world_rank, world_size);

    let params = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::gen_params(
        local_val_len.ilog2() as usize,
        mpi_config.world_size(),
    );

    let mut scratch = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::init_scratch_pad(
        &params,
        &mpi_config,
    );

    let commitment = <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::commit(
        &params,
        &mpi_config,
        &p_key,
        &RefMultiLinearPoly::from_ref(&local_vals_to_commit),
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

        write_commitment_to_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>(&commitment);
        write_commitment_extra_info_to_shared_memory::<C::PCSField, C::FieldConfig, C::PCSConfig>(
            &extra_info,
        );
    }

    MPIConfig::finalize();
}

fn main() {
    let expander_exec_args = ExpanderExecArgs::parse();
    assert_eq!(
        expander_exec_args.fiat_shamir_hash, "SHA256",
        "Only SHA256 is supported for now"
    );

    let pcs_type = PolynomialCommitmentType::from_str(&expander_exec_args.poly_commit).unwrap();

    match (expander_exec_args.field_type.as_str(), pcs_type) {
        ("M31", PolynomialCommitmentType::Raw) => {
            commit::<M31Config>();
        }
        ("GF2", PolynomialCommitmentType::Raw) => {
            commit::<GF2Config>();
        }
        ("Goldilocks", PolynomialCommitmentType::Raw) => {
            commit::<GoldilocksConfig>();
        }
        ("BabyBear", PolynomialCommitmentType::Raw) => {
            commit::<BabyBearConfig>();
        }
        ("BN254", PolynomialCommitmentType::Raw) => {
            commit::<BN254Config>();
        }
        ("BN254", PolynomialCommitmentType::Hyrax) => {
            commit::<BN254ConfigSha2Hyrax>();
        }
        ("BN254", PolynomialCommitmentType::KZG) => {
            commit::<BN254ConfigSha2KZG>();
        }
        (field_type, pcs_type) => panic!(
            "Combination of {:?} and {:?} not supported",
            field_type, pcs_type
        ),
    }
}
