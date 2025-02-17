use std::str::FromStr;
use clap::{Parser, Subcommand};

use expander_config::{FiatShamirHashType, traits::, Config, GKRScheme};
use gkr::{executor::detect_field_type_from_circuit_file, gkr_configs::*};
use gkr_field_config::{M31ExtConfig, GF2ExtConfig, BN254Config};
use mpi_config::{MPIConfig, root_println};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct GKRExecArgs {
    /// Fiat-Shamir Hash: SHA256, or Poseidon, or MiMC5
    #[arg(short, long)]
    pub fiat_shamir_hash: String,

    /// Circuit File Path
    #[arg(short, long)]
    pub circuit_file: String,

    /// Prove or Verify
    #[clap(subcommand)]
    pub subcommands: GKRExecSubCommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum GKRExecSubCommand {
    Prove {
        /// Witness File Path
        #[arg(short, long)]
        witness_file: String,

        /// Output Proof Path
        #[arg(short, long)]
        output_proof_file: String,
    },
    Verify {
        /// Witness File Path
        #[arg(short, long)]
        witness_file: String,

        /// Output Proof Path
        #[arg(short, long)]
        input_proof_file: String,

        /// MPI size
        #[arg(short, long, default_value_t = 1)]
        mpi_size: u32,
    },
}

fn run_gkr<C: GKRFieldConfig>(args: &GKRExecArgs) {
    // let circuit = Circuit::from_file(&args.circuit_file);
    // let gkr = GKR::<C>::new(circuit, config);
    // match args.subcommands {
    //     GKRExecSubCommand::Prove {
    //         witness_file,
    //         output_proof_file,
    //     } => {
    //         let witness = Witness::from_file(&witness_file);
    //         let proof = gkr.prove(witness);
    //         proof.to_file(&output_proof_file);
    //     }
    //     GKRExecSubCommand::Verify {
    //         witness_file,
    //         input_proof_file,
    //         mpi_size,
    //     } => {
    //         let witness = Witness::from_file(&witness_file);
    //         let proof = Proof::from_file(&input_proof_file);
    //         gkr.verify(witness, proof, mpi_size);
    //     }
    // }
}

fn main() {
    let gkr_exec_args = GKRExecArgs::parse();
    let fs_hash_type = FiatShamirHashType::from_str(&gkr_exec_args.fiat_shamir_hash).unwrap();
    let field_type = detect_field_type_from_circuit_file(&gkr_exec_args.circuit_file);

    let mut mpi_config = MPIConfig::new();
    root_println!(mpi_config, "Fiat-Shamir Hash Type: {:?}", &fs_hash_type);
    if let GKRExecSubCommand::Verify {
        witness_file: _,
        input_proof_file: _,
        mpi_size,
    } = &gkr_exec_args.subcommands
    {
        assert_eq!(mpi_config.world_size, 1);
        mpi_config.world_size = *mpi_size as i32;
    }

    match (fs_hash_type.clone(), field_type.clone()) {
        (FiatShamirHashType::SHA256, FieldType::M31) => {
            run_gkr::<M31ExtConfigSha2Orion>(
                &gkr_exec_args,
                Config::new(GKRScheme::Vanilla, mpi_config.clone()),
            )
        }
        (FiatShamirHashType::Poseidon, FieldType::M31) => {
            run_gkr::<M31ExtConfigPoseidonRaw>(
                &gkr_exec_args,
                Config::new(GKRScheme::Vanilla, mpi_config.clone()),
            )
        }
        (FiatShamirHashType::MIMC5, PolynomialCommitmentType::Raw, FieldType::BN254) => {
            run_gkr::<BN254ConfigMIMC5Raw>(
                &gkr_exec_args,
                Config::new(GKRScheme::Vanilla, mpi_config.clone()),
            )
        }
        (FiatShamirHashType::SHA256, PolynomialCommitmentType::Raw, FieldType::BN254) => {
            run_gkr::<BN254ConfigSha2Raw>(
                &gkr_exec_args,
                Config::new(GKRScheme::Vanilla, mpi_config.clone()),
            )
        }
        (FiatShamirHashType::SHA256, PolynomialCommitmentType::Orion, FieldType::GF2) => {
            run_gkr::<GF2ExtConfigSha2Orion>(
                &gkr_exec_args,
                Config::new(GKRScheme::Vanilla, mpi_config.clone()),
            )
        }
        _ => panic!(
            "FS: {:?}, Field: {:?} setting is not yet integrated in gkr-exec",
            fs_hash_type, field_type
        ),
    }

    MPIConfig::finalize();
}