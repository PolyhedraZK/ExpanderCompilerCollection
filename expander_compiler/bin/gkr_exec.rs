use clap::{Parser, Subcommand};
use expander_config::FiatShamirHashType;
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
    pub subcommands: ExpanderExecSubCommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum ExpanderExecSubCommand {
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


fn main() {
    let gkr_exec_args = GKRExecArgs::parse();
    let fs_hash_type = FiatShamirHashType::from_str(&gkr_exec_args.fiat_shamir_hash).unwrap();
    let mut mpi_config = MPIConfig::new();
    root_println!(mpi_config, "Fiat-Shamir Hash Type: {:?}", &fs_hash_type);


    println!("Hello, world! I'm GKR EXEC");
    MPIConfig::finalize();
}