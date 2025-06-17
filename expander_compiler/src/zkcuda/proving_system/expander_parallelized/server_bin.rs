use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use expander_compiler::{
    frontend::{BN254Config, BabyBearConfig, Config, GF2Config, GoldilocksConfig, M31Config},
    zkcuda::{
        proof::ComputationGraph,
        proving_system::{
            expander::structs::{ExpanderProverSetup, ExpanderVerifierSetup},
            expander_parallelized::{
                server_utils::{
                    root_main, serve, worker_main, ServerState, GLOBAL_COMMUNICATOR, SERVER_IP,
                    UNIVERSE,
                },
                structs::ServerFns,
                ParallelizedExpander,
            },
        },
    },
};
use gkr::{BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use gkr_engine::{FieldEngine, GKREngine, MPIConfig, MPIEngine, PolynomialCommitmentType};
use tokio::sync::{oneshot, Mutex};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ExpanderExecArgs {
    /// M31, GF2, BN254, Goldilocks, BabyBear
    #[arg(short, long, default_value = "M31")]
    pub field_type: String,

    /// Fiat-Shamir Hash: SHA256, or Poseidon, or MiMC5
    #[arg(short, long, default_value = "SHA256")]
    pub fiat_shamir_hash: String,

    /// Polynomial Commitment Scheme: Raw, or Orion
    #[arg(short, long, default_value = "Raw")]
    pub poly_commit: String,

    /// The port number for the server to listen on.
    #[arg(short, long, default_value = "Port")]
    pub port_number: String,
}

#[tokio::main]
pub async fn main() {
    let expander_exec_args = ExpanderExecArgs::parse();
    assert_eq!(
        expander_exec_args.fiat_shamir_hash, "SHA256",
        "Only SHA256 is supported for now"
    );

    let pcs_type = PolynomialCommitmentType::from_str(&expander_exec_args.poly_commit).unwrap();

    match (expander_exec_args.field_type.as_str(), pcs_type) {
        ("M31", PolynomialCommitmentType::Raw) => {
            serve::<M31Config, M31Config, ParallelizedExpander<_>>(expander_exec_args.port_number)
                .await;
        }
        ("GF2", PolynomialCommitmentType::Raw) => {
            serve::<GF2Config, GF2Config, ParallelizedExpander<_>>(expander_exec_args.port_number)
                .await;
        }
        ("Goldilocks", PolynomialCommitmentType::Raw) => {
            serve::<GoldilocksConfig, GoldilocksConfig, ParallelizedExpander<_>>(
                expander_exec_args.port_number,
            )
            .await;
        }
        ("BabyBear", PolynomialCommitmentType::Raw) => {
            serve::<BabyBearConfig, BabyBearConfig, ParallelizedExpander<_>>(
                expander_exec_args.port_number,
            )
            .await;
        }
        ("BN254", PolynomialCommitmentType::Raw) => {
            serve::<BN254Config, BN254Config, ParallelizedExpander<_>>(
                expander_exec_args.port_number,
            )
            .await;
        }
        ("BN254", PolynomialCommitmentType::Hyrax) => {
            serve::<BN254ConfigSha2Hyrax, BN254Config, ParallelizedExpander<_>>(
                expander_exec_args.port_number,
            )
            .await;
        }
        ("BN254", PolynomialCommitmentType::KZG) => {
            serve::<BN254ConfigSha2KZG, BN254Config, ParallelizedExpander<_>>(
                expander_exec_args.port_number,
            )
            .await;
        }
        (field_type, pcs_type) => {
            panic!("Combination of {field_type:?} and {pcs_type:?} not supported for parallelized expander proving system.");
        }
    }
}
