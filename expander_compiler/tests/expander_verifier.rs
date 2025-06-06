use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proof::ComputationGraph;
use expander_compiler::zkcuda::proving_system::{ExpanderGKRProvingSystem, ParallelizedExpanderGKRProvingSystem, ProvingSystem,};
use expander_compiler::zkcuda::{context::*, kernel::*};
use gkr::BN254ConfigSha2Hyrax;
use gkr_engine::FieldEngine;
use serdes::ExpSerde;
use serde::{Deserialize, Serialize};
use std::fs;
#[test]
fn expander_verifier() -> std::io::Result<()>{ 
	let compile_result = stacker::grow(32 * 1024 * 1024 * 1024, ||
		{
			let mut ctx: Context<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>> = Context::default();
			let file = std::fs::File::open("graph.txt").unwrap();
			let reader = std::io::BufReader::new(file);
			let computation_graph = ComputationGraph::<BN254Config>::deserialize_from(reader).unwrap();
			let (_, verifier_setup) = ctx.proving_system_setup(&computation_graph);
			let file = std::fs::File::open("proof.txt").unwrap();
			let reader = std::io::BufReader::new(file);
			let proof = CombinedProof::<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>>::deserialize_from(reader).unwrap();
			assert!(computation_graph.verify(&proof, &verifier_setup));
			<ParallelizedExpanderGKRProvingSystem::<BN254ConfigSha2Hyrax> as ProvingSystem<BN254Config>>::post_process();
		}
	);
	Ok(())
}
