use std::io::Cursor;
use std::fs;
use arith::{Field, FieldSerde};
use clap::{Parser, Subcommand};

use expander_config::GKRConfig;
use gkr::{executor::detect_field_type_from_circuit_file, gkr_configs::*, gkr_prove, gkr_verify};
use gkr_field_config::{FieldType, GKRFieldConfig};
use mpi_config::MPIConfig;
use sumcheck::ProverScratchPad;
use expander_transcript::Transcript;
use expander_circuit::Circuit;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct GKRExecArgs {
    /// Fiat-Shamir Hash: SHA256, or Poseidon, or MiMC5
    // #[arg(short, long)]
    // pub fiat_shamir_hash: String,

    /// Circuit File Path
    #[arg(short, long)]
    pub circuit_file: String,

    /// Prove or Verify
    #[clap(subcommand)]
    pub subcommands: GKRExecSubCommand,

    /// Transcript state and input layer claims for PCS
    #[arg(short, long)]
    pub input_layer_state: String,
}

#[derive(Debug, Subcommand, Clone)]
pub enum GKRExecSubCommand {
    Prove {
        /// Witness File Path
        #[arg(short, long)]
        witness_file: String,

        /// Output Proof Path
        #[arg(short, long)]
        output_gkr_proof_file: String,
    },
    Verify {
        /// Witness File Path
        #[arg(short, long)]
        witness_file: String,

        /// Output Proof Path
        #[arg(short, long)]
        input_gkr_proof_file: String,

        /// MPI size
        #[arg(short, long, default_value_t = 1)]
        mpi_size: u32,
    },
}

fn max_n_vars<C: GKRFieldConfig>(circuit: &Circuit<C>) -> (usize, usize) {
    let mut max_num_input_var = 0;
    let mut max_num_output_var = 0;
    for layer in circuit.layers.iter() {
        max_num_input_var = max_num_input_var.max(layer.input_var_num);
        max_num_output_var = max_num_output_var.max(layer.output_var_num);
    }
    (max_num_input_var, max_num_output_var)
}

fn serialize_transcript_state_and_input_layer_claims<C: GKRFieldConfig, T: Transcript<C::ChallengeField>>(
    transcript: &mut T,
    rx: &Vec<C::ChallengeField>,
    ry: &Option<Vec<C::ChallengeField>>,
    rsimd: &Vec<C::ChallengeField>,
    rmpi: &Vec<C::ChallengeField>,
) -> Vec<u8> {
    let mut state = transcript.hash_and_return_state();            
    rx.serialize_into(&mut state).unwrap();
    if let Some(ry) = ry {
        ry.serialize_into(&mut state).unwrap();
    } else {
        // if ry is None, serialize a zero-sized vector
        Vec::<C::ChallengeField>::new().serialize_into(&mut state).unwrap();
    }
    rsimd.serialize_into(&mut state).unwrap();
    rmpi.serialize_into(&mut state).unwrap();
    state
}

fn run_gkr<Cfg: GKRConfig>(args: &GKRExecArgs) {
    let subcommands = args.subcommands.clone();
    let mut mpi_config = MPIConfig::new();

    match subcommands {
        GKRExecSubCommand::Prove {
            witness_file,
            output_gkr_proof_file,
        } => {
            let mut circuit =
                Circuit::<Cfg::FieldConfig>::load_circuit::<Cfg>(&args.circuit_file);
            circuit.load_witness_file(&witness_file);
            let (max_num_input_var, max_num_output_var) = max_n_vars(&circuit);
            let mut prover_scratch = ProverScratchPad::<Cfg::FieldConfig>::new(
                max_num_input_var,
                max_num_output_var,
                mpi_config.world_size as usize,
            );

            let mut transcript = <Cfg>::Transcript::new();
            transcript.append_u8_slice(&[0u8; 32]); // TODO: Replace with the commitment, and hash an additional a few times
            circuit.evaluate();
            let (claimed_v, rx, ry, rsimd, rmpi) = gkr_prove(
                &circuit,
                &mut prover_scratch,
                &mut transcript,
                &MPIConfig::new(),
            );
            assert!(claimed_v.is_zero());

            if mpi_config.is_root() {
                let proof = transcript.finalize_and_get_proof();
                fs::write(output_gkr_proof_file, &proof.bytes).expect("Unable to write proof to file.");
                
                let state = serialize_transcript_state_and_input_layer_claims::<Cfg::FieldConfig, _>(&mut transcript, &rx, &ry, &rsimd, &rmpi);
                fs::write(args.input_layer_state.clone(), &state).expect("Unable to write transcript state to file.");   
            }
        }
        GKRExecSubCommand::Verify {
            witness_file,
            input_gkr_proof_file,
            mpi_size,
        } => {
            assert_eq!(mpi_config.world_size, 1);
            mpi_config.world_size = mpi_size as i32;    

            let mut circuit =
                Circuit::<Cfg::FieldConfig>::load_circuit::<Cfg>(&args.circuit_file);
            circuit.load_witness_file(&witness_file);

            let proof_bytes = fs::read(input_gkr_proof_file).expect("Unable to read proof file.");

            let mut transcript = <Cfg>::Transcript::new();
            transcript.append_u8_slice(&[0u8; 32]); // TODO: Replace with the commitment, and hash an additional a few times
            let mut cursor = Cursor::new(&proof_bytes);
            cursor.set_position(32);
            let (verified, rz0, rz1, r_simd, r_mpi, _claimed_v0, _claimed_v1) = gkr_verify(
                &mpi_config,
                &circuit,
                &[],
                &<Cfg::FieldConfig as GKRFieldConfig>::ChallengeField::ZERO,
                &mut transcript,
                &mut cursor,
            );
            
            let state = serialize_transcript_state_and_input_layer_claims::<Cfg::FieldConfig, _>(&mut transcript, &rz0, &rz1, &r_simd, &r_mpi);
            fs::write(args.input_layer_state.clone(), &state).expect("Unable to write transcript state to file.");
            assert!(verified);
        }    
    }

}

fn main() {
    let gkr_exec_args = GKRExecArgs::parse();
    
    // temporarily use sha2 for all 
    // let fs_hash_type = FiatShamirHashType::from_str(&gkr_exec_args.fiat_shamir_hash).unwrap();
    let field_type = detect_field_type_from_circuit_file(&gkr_exec_args.circuit_file);

    // root_println!(mpi_config, "Fiat-Shamir Hash Type: {:?}", &fs_hash_type);

    #[allow(unreachable_patterns)]
    match field_type.clone() {
        FieldType::M31 => {
            run_gkr::<M31ExtConfigSha2Raw>(
                &gkr_exec_args,
            )
        }
        FieldType::GF2 => {
            run_gkr::<GF2ExtConfigSha2Raw>(
                &gkr_exec_args,
            )
        }
        FieldType::BN254 => {
            run_gkr::<BN254ConfigSha2Raw>(
                &gkr_exec_args,
            )
        }
        _ => panic!(
            "Field: {:?} setting is not yet integrated in gkr-exec",
            field_type
        ),
    }

    MPIConfig::finalize();
}