use gkr_engine::{ExpanderPCS, FieldEngine, FieldType, GKREngine, PolynomialCommitmentType};
use std::process::Command;

use crate::utils::misc::prev_power_of_two;

#[allow(clippy::zombie_processes)]
pub fn start_server<C: GKREngine>(binary: &str, max_parallel_count: usize, port_number: u16)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let (actual_mpi_size, field_name, pcs_name) = parse_config::<C>(max_parallel_count);

    let cmd_str = format!(
        "mpiexec -n {actual_mpi_size} {binary} --field-type {field_name} --poly-commit {pcs_name} --port-number {port_number}"
    );
    exec_command(&cmd_str, false);
}

fn parse_config<C: GKREngine>(desired_mpi_size: usize) -> (usize, String, String)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let num_cpus_to_use = prev_power_of_two(num_cpus::get_physical());

    let actual_mpi_size = if desired_mpi_size > num_cpus_to_use {
        num_cpus_to_use
    } else {
        desired_mpi_size
    };

    let field_name = match <C::FieldConfig as FieldEngine>::FIELD_TYPE {
        FieldType::M31x16 => "M31",
        FieldType::GF2Ext128 => "GF2",
        FieldType::Goldilocksx8 => "Goldilocks",
        FieldType::BabyBearx16 => "BabyBear",
        FieldType::BN254 => "BN254",
        _ => panic!("Unsupported field type"),
    };

    let pcs_name = match <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::PCS_TYPE {
        PolynomialCommitmentType::Raw => "Raw",
        PolynomialCommitmentType::Hyrax => "Hyrax",
        PolynomialCommitmentType::KZG => "KZG",
        _ => panic!("Unsupported PCS type"),
    };

    (
        actual_mpi_size,
        field_name.to_string(),
        pcs_name.to_string(),
    )
}

#[allow(clippy::zombie_processes)]
fn exec_command(cmd: &str, wait_for_completion: bool) {
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
