use std::str::FromStr;

use clap::Parser;
use expander_compiler::zkcuda::proving_system::{
    expander::config::{
        ZKCudaBN254Hyrax, ZKCudaBN254HyraxBatchPCS, ZKCudaBN254KZG, ZKCudaBN254KZGBatchPCS,
        ZKCudaBN254MIMCKZG, ZKCudaBN254MIMCKZGBatchPCS,
    },
    expander_parallelized::server_ctrl::{serve, ExpanderExecArgs},
    ExpanderNoOverSubscribe,
};
use gkr_engine::{FiatShamirHashType, PolynomialCommitmentType};

async fn async_main() {
    let expander_exec_args = ExpanderExecArgs::parse();

    let pcs_type = PolynomialCommitmentType::from_str(&expander_exec_args.poly_commit).unwrap();

    let fiat_shamir_hash = match expander_exec_args.fiat_shamir_hash.as_str() {
        "SHA256" => FiatShamirHashType::SHA256,
        "MIMC5" => FiatShamirHashType::MIMC5,
        _ => panic!("Unsupported Fiat-Shamir hash function"),
    };

    match (
        expander_exec_args.field_type.as_str(),
        pcs_type,
        fiat_shamir_hash,
    ) {
        ("BN254", PolynomialCommitmentType::Hyrax, FiatShamirHashType::SHA256) => {
            if expander_exec_args.batch_pcs {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254HyraxBatchPCS>>(
                    expander_exec_args.port_number,
                )
                .await;
            } else {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254Hyrax>>(
                    expander_exec_args.port_number,
                )
                .await;
            }
        }
        ("BN254", PolynomialCommitmentType::KZG, FiatShamirHashType::SHA256) => {
            if expander_exec_args.batch_pcs {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254KZGBatchPCS>>(
                    expander_exec_args.port_number,
                )
                .await;
            } else {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254KZG>>(
                    expander_exec_args.port_number,
                )
                .await;
            }
        }
        ("BN254", PolynomialCommitmentType::KZG, FiatShamirHashType::MIMC5) => {
            if expander_exec_args.batch_pcs {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254MIMCKZGBatchPCS>>(
                    expander_exec_args.port_number,
                )
                .await;
            } else {
                serve::<_, _, ExpanderNoOverSubscribe<ZKCudaBN254MIMCKZG>>(
                    expander_exec_args.port_number,
                )
                .await;
            }
        }
        (field_type, pcs_type, fiat_shamir_hash) => {
            panic!("Combination of {field_type:?}, {pcs_type:?}, and {fiat_shamir_hash:?} not supported for no oversubscribe expander proving system.");
        }
    }
}

pub fn main() {
    println!("Enter expander_server no oversubscribe!");
    let stack_size_mb = std::env::var("THREAD_STACK_SIZE_MB")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(64);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(stack_size_mb * 1024 * 1024) // stack size in MB
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async_main());
}
