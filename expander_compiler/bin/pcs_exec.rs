use std::io::{Cursor, Read};
use std::fs::{self, read};
use arith::{Field, FieldSerde};
use clap::{Parser, Subcommand};

use expander_compiler::frontend::extra;
use expander_config::GKRConfig;
use gkr::{executor::detect_field_type_from_circuit_file, gkr_configs::*, gkr_prove, gkr_verify};
use gkr_field_config::{FieldType, GKRFieldConfig};
use mpi_config::MPIConfig;
use poly_commit::PCSForExpanderGKR;
use sumcheck::ProverScratchPad;
use expander_transcript::Transcript;
use expander_circuit::Circuit;

use poly_commit::{
    expander_pcs_init_testing_only, raw::*, ExpanderGKRChallenge, PCSEmptyType,
};
use polynomials::{EqPolynomial, MultiLinearPoly, RefMultiLinearPoly};

use rand::rngs::StdRng;
use rand::SeedableRng;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct PCSExecArgs {
    /// Fiat-Shamir Hash: SHA256, or Poseidon, or MiMC5
    // #[arg(short, long)]
    // pub fiat_shamir_hash: String,

    /// One of M31, GF2, BN254
    /// #[arg(short, long)]
    pub field_type: String,

    /// Commit, Open or Verify
    #[clap(subcommand)]
    pub subcommands: PCSExecSubCommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum PCSExecSubCommand {
    Commit {
        /// If MPI_Size = 1, then the filename is exactly values_file_prefix
        /// Otherwise, it should be values_file_prefix_rank
        /// e.g. values_file_prefix_0, values_file_prefix_1, values_file_prefix_2
        #[arg(short, long)]
        values_file_prefix: String,

        /// Output Commitment and Extra Info File
        #[arg(short, long)]
        output_commitment_file: String,
    },
    Open {
        /// Input Commitment and Extra Info File from the Commit phase
        #[arg(short, long)]
        input_commitment_file: String,

         /// Transcript state and input layer claims for PCS
        #[arg(short, long)]
        gkr_input_layer_state: String,
    },
    Verify {
        /// Input Commitment and Extra Info File from the Commit phase
        #[arg(short, long)]
        input_commitment_file: String,

         /// Transcript state and input layer claims for PCS
        #[arg(short, long)]
        gkr_input_layer_state: String,

        /// MPI size
        #[arg(short, long, default_value_t = 1)]
        mpi_size: u32,
    },
}

fn deserialize_local_values_from_file<F: Field>(values_file_prefix: String, mpi_config: &MPIConfig) -> Vec<F> {
    let actual_values_file = if mpi_config.world_size == 1 {
        values_file_prefix
    } else {
        format!("{}_{}", values_file_prefix, mpi_config.world_rank)
    };
    let mut reader = Cursor::new(fs::read(actual_values_file).unwrap());
    Vec::<F>::deserialize_from(&mut reader).unwrap()
}

fn deserialize_transcript_state_and_input_layer_claims_from_file<C: GKRFieldConfig, T: Transcript<C::ChallengeField>>(
    gkr_input_layer_state: String,
) -> (Vec<u8>, Vec<C::ChallengeField>, Option<Vec<C::ChallengeField>>, Vec<C::ChallengeField>, Vec<C::ChallengeField>) {
    let mut reader = Cursor::new(fs::read(gkr_input_layer_state).unwrap());
    let transcript_state = Vec::<u8>::deserialize_from(&mut reader).unwrap();
    let rx = Vec::<C::ChallengeField>::deserialize_from(&mut reader).unwrap();
    let ry_data = Vec::<C::ChallengeField>::deserialize_from(&mut reader).unwrap();
    let ry = if ry_data.is_empty() { None } else { Some(ry_data) };
    let rsimd = Vec::<C::ChallengeField>::deserialize_from(&mut reader).unwrap();
    let rmpi = Vec::<C::ChallengeField>::deserialize_from(&mut reader).unwrap();
    
    (transcript_state, rx, ry, rsimd, rmpi)
}

fn serialize_commitment_and_extra_info_into_file<C: GKRFieldConfig, T: Transcript<C::ChallengeField>, PCS: PCSForExpanderGKR<C, T>>(
    filename: String,
    commitment: &PCS::Commitment,
    extra_info: &PCS::ScratchPad,
    mpi_config: &MPIConfig,
) {
    if mpi_config.is_root() {
        let mut writer = Cursor::new(Vec::<u8>::new());
        commitment.serialize_into(&mut writer).unwrap();
        let _ = extra_info; // TODO: Implement serialization for ScratchPad
        // extra_info.serialize_into(&mut writer).unwrap();
        fs::write(filename, &writer.into_inner()).expect("Unable to write commitment and extra info to file.");
    }
}

fn deserialize_commitment_and_extra_info_from_file<C: GKRFieldConfig, T: Transcript<C::ChallengeField>, PCS: PCSForExpanderGKR<C, T>>(
    filename: String,
) -> (PCS::Commitment, PCS::ScratchPad) {
    let mut reader = Cursor::new(fs::read(filename).unwrap());
    let commitment = PCS::Commitment::deserialize_from(&mut reader).unwrap();
    // let extra_info = PCS::ScratchPad::deserialize_from(&mut reader).unwrap();
    let extra_info = Default::default();
    (commitment, extra_info)
}

fn pcs_testing_setup_fixed_seed<C: GKRFieldConfig, T: Transcript<C::ChallengeField>>(
    vals: &[C::SimdCircuitField],
) -> (
    RawExpanderGKRParams,
    PCSEmptyType,
    PCSEmptyType,
    RawExpanderGKRScratchPad,
) {
    // We don't have an interface for the potential pcs setup
    // So we're just going to use the testing setup with fixed seed
    let mut rng = StdRng::from_seed([0; 32]);
    expander_pcs_init_testing_only::<
        C,
        T,
        RawExpanderGKR<_, _>,
    >(
        vals.len().trailing_zeros() as usize,
        &MPIConfig::default(),
        &mut rng,
    )
}


fn run_pcs<Cfg: GKRConfig>(args: &PCSExecArgs) {
    let subcommands = args.subcommands.clone();
    let mut mpi_config = MPIConfig::new();

    match subcommands {
        PCSExecSubCommand::Commit {
            values_file_prefix,
            output_commitment_file,
        } => {
            let vals = deserialize_local_values_from_file::<<Cfg::FieldConfig as GKRFieldConfig>::SimdCircuitField>(values_file_prefix, &mpi_config);
            assert!(vals.len() & (vals.len() - 1) == 0);
            let (params, p_key, _v_key, mut scratch) = pcs_testing_setup_fixed_seed::<Cfg::FieldConfig, Cfg::Transcript>(&vals);

            let vals = vals.to_vec();
            let poly_ref = RefMultiLinearPoly::from_ref(&vals);
            let raw_commitment = RawExpanderGKR::<
                Cfg::FieldConfig,
                Cfg::Transcript,
            >::commit(
                &params,
                &MPIConfig::default(),
                &p_key,
                &poly_ref,
                &mut scratch,
            );
            serialize_commitment_and_extra_info_into_file::<Cfg::FieldConfig, Cfg::Transcript, RawExpanderGKR<_, _>>(output_commitment_file, &raw_commitment, &scratch, &mpi_config);
        }
        PCSExecSubCommand::Open {
            input_commitment_file,
            gkr_input_layer_state,
        } => {
            let (commitment, extra_info) = deserialize_commitment_and_extra_info_from_file::<Cfg, extra::Transcript, PCSForExpanderGKR<Cfg>>(input_commitment_file);
            let (transcript_state, rx, ry, rsimd, rmpi
            ) = deserialize_transcript_state_and_input_layer_claims_from_file::<Cfg, extra::Transcript>(gkr_input_layer_state);
        }
        PCSExecSubCommand::Verify {
            input_commitment_file,
            gkr_input_layer_state,
            mpi_size,
        } => {
            let (commitment, extra_info) = deserialize_commitment_and_extra_info_from_file::<Cfg, extra::Transcript, PCSForExpanderGKR<Cfg>>(input_commitment_file);
            let (transcript_state, rx, ry, rsimd, rmpi
            ) = deserialize_transcript_state_and_input_layer_claims_from_file::<Cfg, extra::Transcript>(gkr_input_layer_state);
        }
    }
    MPIConfig::finalize();
}

fn main() {
    let pcs_exec_args = PCSExecArgs::parse();
    
    // temporarily use sha2 for all 
    // let fs_hash_type = FiatShamirHashType::from_str(&gkr_exec_args.fiat_shamir_hash).unwrap();
    let field_type = &pcs_exec_args.field_type;

    // root_println!(mpi_config, "Fiat-Shamir Hash Type: {:?}", &fs_hash_type);

    #[allow(unreachable_patterns)]
    match field_type.as_str() {
        "M31" => {
            run_pcs::<M31ExtConfigSha2Raw>(
                &pcs_exec_args,
            )
        }
        "GF2" => {
            run_pcs::<GF2ExtConfigSha2Raw>(
                &pcs_exec_args,
            )
        }
        "BN254" => {
            run_pcs::<BN254ConfigSha2Raw>(
                &pcs_exec_args,
            )
        }
        _ => panic!(
            "Field: {:?} setting is not yet integrated in gkr-exec",
            field_type
        ),
    }
}