use expander_compiler::frontend::*;
use expander_compiler::circuit::layered::{NormalInputType, CrossLayerInputType};
use expander_compiler::Proof;
use expander_config::BN254ConfigSha2Hyrax;
use serdes::ExpSerde;
use serde::{Serialize, Deserialize};
use stacker;
use std::fs;
use std::time::Instant;
use circuit_std_rs::{
    logup::{query_count_hint, rangeproof_hint, LogUpRangeProofTable},
    LogUpCircuit, LogUpParams,
};
#[test]
fn expander_prover() -> std::io::Result<()>{ 
	let compile_result = stacker::grow(12 * 1024 * 1024 * 1024, ||
		{
			println!("Read circuit & witness Begin");
			let start: Instant = Instant::now();
			let file = std::fs::File::open("circuit.txt").unwrap();
			let reader = std::io::BufReader::new(file);
			let layered_circuit = expander_compiler::circuit::layered::Circuit::<BN254Config, NormalInputType>::deserialize_from(reader).unwrap();
			let mut expander_circuit = layered_circuit.export_to_expander_flatten();
			let config = BN254Config::new_expander_config();
			let file = std::fs::File::open("witness.txt").unwrap();
			let reader = std::io::BufReader::new(file);
			let witness = expander_compiler::circuit::layered::witness::Witness::<BN254Config>::deserialize_from(reader).unwrap();
			let (simd_input, simd_public_input) = witness.to_simd();
			expander_circuit.layers[0].input_vals = simd_input;
			expander_circuit.public_input = simd_public_input.clone();
			expander_circuit.evaluate();
			let duration = start.elapsed();
			println!("Read circuit & witness End");
			println!("Read circuit & witness Time: {:?}", duration);
			println!("Prove Begin");
			let start: Instant = Instant::now();
			let (claimed_v, proof) = gkr::executor::prove(&mut expander_circuit, &config);
			let duration = start.elapsed();
			println!("Prove End");
			println!("Proof Generation Time: {:?}", duration);
			let file = std::fs::File::create("proof.txt").unwrap();
			let writer = std::io::BufWriter::new(file);
			proof.serialize_into(writer).unwrap();
		}
	);
	Ok(())
}
