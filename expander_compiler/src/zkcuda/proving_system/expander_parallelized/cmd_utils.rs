use gkr_engine::{
    ExpanderPCS, FiatShamirHashType, FieldEngine, FieldType, GKREngine, PolynomialCommitmentType,
    Transcript,
};
use std::process::Command;

#[allow(clippy::zombie_processes)]
pub fn start_server<C: GKREngine>(
    binary: &str,
    max_parallel_count: usize,
    port_number: u16,
    batch_pcs: bool,
) {
    let (overscribe, field_name, pcs_name, fiat_shamir_hash) =
        parse_config::<C>(max_parallel_count);

    let batch_pcs_option = if batch_pcs { "--batch-pcs" } else { "" };
    let cmd_str = format!(
        "mpiexec -n {max_parallel_count} {overscribe} {binary} --field-type {field_name} --poly-commit {pcs_name} --port-number {port_number} {batch_pcs_option} --fiat-shamir-hash {fiat_shamir_hash}"
    );
    exec_command(&cmd_str, false);
}

fn parse_config<C: GKREngine>(mpi_size: usize) -> (String, String, String, String)
where
{
    let oversubscription = if mpi_size > num_cpus::get_physical() {
        println!("Warning: Not enough cores available for the requested number of processes. Using oversubscription.");
        "--oversubscribe"
    } else {
        ""
    };

    let field_name = match <C::FieldConfig as FieldEngine>::FIELD_TYPE {
        FieldType::M31x16 => "M31",
        FieldType::GF2Ext128 => "GF2",
        FieldType::Goldilocksx8 => "Goldilocks",
        FieldType::BabyBearx16 => "BabyBear",
        FieldType::BN254 => "BN254",
        _ => panic!("Unsupported field type"),
    };

    let pcs_name = match <C::PCSConfig as ExpanderPCS<C::FieldConfig>>::PCS_TYPE {
        PolynomialCommitmentType::Raw => "Raw",
        PolynomialCommitmentType::Hyrax => "Hyrax",
        PolynomialCommitmentType::KZG => "KZG",
        _ => panic!("Unsupported PCS type"),
    };

    let fiat_shamir_hash = match <C::TranscriptConfig as Transcript>::HASH_TYPE {
        FiatShamirHashType::SHA256 => "SHA256", // default
        FiatShamirHashType::MIMC5 => "MIMC5",   // for recursion
        _ => panic!("Unsupported hash function"),
    };

    (
        oversubscription.to_string(),
        field_name.to_string(),
        pcs_name.to_string(),
        fiat_shamir_hash.to_string(),
    )
}

#[allow(clippy::zombie_processes)]
fn exec_command(cmd: &str, wait_for_completion: bool) {
    println!("Executing command: {cmd}");
    let mut parts = cmd.split_whitespace();
    let command = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();
    let mut child = Command::new(command)
        .args(args)
        .spawn()
        .expect("Failed to start child process");
    if wait_for_completion {
        let _ = child.wait();
    }
}
