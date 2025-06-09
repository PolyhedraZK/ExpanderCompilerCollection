use gkr_engine::{ExpanderPCS, FieldEngine, FieldType, GKREngine, PolynomialCommitmentType};
use std::process::Command;

#[allow(clippy::zombie_processes)]
pub fn start_server<C: GKREngine>(max_parallel_count: usize)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
{
    let (overscribe, field_name, pcs_name) = parse_config::<C>(max_parallel_count);

    let cmd_str = format!(
        "mpiexec -n {max_parallel_count} {overscribe} ../target/release/expander_serve --field-type {field_name} --poly-commit {pcs_name}",
    );
    exec_command(&cmd_str, false);
}

fn parse_config<C: GKREngine>(mpi_size: usize) -> (String, String, String)
where
    C::FieldConfig: FieldEngine<SimdCircuitField = C::PCSField>,
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

    let pcs_name = match <C::PCSConfig as ExpanderPCS<C::FieldConfig, C::PCSField>>::PCS_TYPE {
        PolynomialCommitmentType::Raw => "Raw",
        PolynomialCommitmentType::Hyrax => "Hyrax",
        PolynomialCommitmentType::KZG => "KZG",
        _ => panic!("Unsupported PCS type"),
    };

    (
        oversubscription.to_string(),
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
