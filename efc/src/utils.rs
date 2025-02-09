use expander_compiler::{
    circuit::{ir::hint_normalized::witness_solver, layered::witness::Witness},
    frontend::*,
    utils::serde::Serde,
};
use serde::de::DeserializeOwned;
use std::{fs, path::Path, thread, time::Duration};

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

pub fn convert_limbs(limbs: Vec<u8>) -> [M31; 48] {
    let converted: Vec<M31> = limbs.into_iter().map(|x| M31::from(x as u32)).collect();
    converted.try_into().expect("Limbs should have 48 elements")
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

pub fn get_solver<
    C: Config,
    Cir: internal::DumpLoadTwoVariables<Variable> + GenericDefine<C> + Clone,
>(
    dir: &str,
    circuit_name: &str,
    circuit: Cir,
) -> WitnessSolver<C> {
    ensure_directory_exists(dir);
    let file_name = format!("solver_{}.txt", circuit_name);
    let w_s = if std::fs::metadata(&file_name).is_ok() {
        println!("The solver exists!");
        let file = std::fs::File::open(&file_name).unwrap();
        let reader = std::io::BufReader::new(file);
        witness_solver::WitnessSolver::deserialize_from(reader).unwrap()
    } else {
        println!("The solver {} does not exist.", file_name);
        let compile_result = compile_generic(&circuit, CompileOptions::default()).unwrap();
        let file = std::fs::File::create(&file_name).unwrap();
        let writer = std::io::BufWriter::new(file);
        compile_result
            .witness_solver
            .serialize_into(writer)
            .unwrap();
        let CompileResult {
            witness_solver,
            layered_circuit,
        } = compile_result;
        let circuit_name = format!("circuit_{}.txt", circuit_name);
        let file = std::fs::File::create(&circuit_name).unwrap();
        let writer = std::io::BufWriter::new(file);
        layered_circuit.serialize_into(writer).unwrap();
        witness_solver
    };
    w_s
}

pub fn wait_for_file(directory: &str) {
    let path = Path::new(directory);

    loop {
        if let Ok(entries) = fs::read_dir(path) {
            let file_count = entries.count();

            if file_count == 1 {
                println!("Found exactly one file, proceeding...");
                break;
            } else {
                println!("File count: {}. Waiting...", file_count);
            }
        } else {
            println!("Failed to read directory: {}", directory);
        }

        thread::sleep(Duration::from_millis(500));
    }
}
