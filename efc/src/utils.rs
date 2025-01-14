use expander_compiler::{circuit::layered::witness::Witness, frontend::*};
use serde::de::DeserializeOwned;
use std::{fs, path::Path};

pub fn run_circuit<C: Config>(compile_result: &CompileResult<C>, witness: Witness<C>) {
    //can be skipped
    let output = compile_result.layered_circuit.run(&witness);
    for x in output.iter() {
        assert!(*x);
    }

    // ########## EXPANDER ##########

    //compile
    let mut expander_circuit = compile_result
        .layered_circuit
        .export_to_expander::<C::DefaultGKRFieldConfig>()
        .flatten();
    let config = expander_config::Config::<C::DefaultGKRConfig>::new(
        expander_config::GKRScheme::Vanilla,
        mpi_config::MPIConfig::new(),
    );

    let (simd_input, simd_public_input) = witness.to_simd::<C::DefaultSimdField>();
    println!("{} {}", simd_input.len(), simd_public_input.len());
    expander_circuit.layers[0].input_vals = simd_input;
    expander_circuit.public_input = simd_public_input.clone();

    // prove
    expander_circuit.evaluate();
    let mut prover = gkr::Prover::new(&config);
    prover.prepare_mem(&expander_circuit);
    let (claimed_v, proof) = gkr::executor::prove(&mut expander_circuit, &config);

    // verify
    assert!(gkr::executor::verify(
        &mut expander_circuit,
        &config,
        &proof,
        &claimed_v
    ));
}

pub fn read_from_json_file<T: DeserializeOwned + std::fmt::Debug>(
    file_path: &str,
) -> Result<T, Box<dyn std::error::Error>> {
    let json_content = fs::read_to_string(file_path)?;

    let data: T = serde_json::from_str(&json_content)?;

    Ok(data)
}

pub fn ensure_directory_exists(dir: &str) {
    let path = Path::new(dir);

    if !path.exists() {
        fs::create_dir_all(path).expect("Failed to create directory");
        println!("Directory created: {}", dir);
    } else {
        println!("Directory already exists: {}", dir);
    }
}
