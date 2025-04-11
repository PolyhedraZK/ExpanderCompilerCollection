use expander_compiler::frontend::*;
use expander_compiler::circuit::layered::{NormalInputType, CrossLayerInputType};
use expander_compiler::Proof;
use expander_compiler::field::BN254Fr as BN254;
use expander_compiler::field;
use crate::HintRegistry;
use expander_config::BN254ConfigSha2Raw;
use serdes::ExpSerde;
use serde::{Serialize, Deserialize};
use stacker;
use std::fs;
use std::time::Instant;
use circuit_std_rs::{
    logup::{query_count_hint, rangeproof_hint, LogUpRangeProofTable},
    LogUpCircuit, LogUpParams,
};
declare_circuit!(Circuit {
	output: [[[[Variable]]]], 
	input: [[[[Variable]]]], 
	_features_features_0_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_0_Constant_output_0: Variable, 
	_features_features_0_Constant_1_output_0: Variable, 
	_features_features_0_Div_output_0_r: [[[[Variable]]]], 
	_features_features_0_Div_output_0: [[[[Variable]]]], 
	_features_features_0_Constant_2_output_0: [[[Variable]]], 
	features_0_conv_weight: [[[[Variable]]]], 
	input_mat_ru: [Variable], 
	features_0_conv_weight_mat_rv: [Variable], 
});

#[derive(Serialize, Deserialize, Debug)]
struct Circuit_Input {
	output: Vec<Vec<Vec<Vec<i64>>>>, 
	input: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_Constant_output_0: i64, 
	_features_features_0_Constant_1_output_0: i64, 
	_features_features_0_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	features_0_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
}

fn input_copy(input: &Circuit_Input, assignment: &mut Circuit::<BN254>){
	assignment.input_mat_ru = vec![BN254::default();16384]; 
	assignment.features_0_conv_weight_mat_rv = vec![BN254::default();64]; 
	assignment.output = vec![vec![vec![vec![BN254::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input.output[i][j][k][l] >= 0{
						assignment.output[i][j][k][l] = BN254::from((input.output[i][j][k][l]) as u64); 
					} else {
						assignment.output[i][j][k][l] = -field::BN254Fr::from((-input.output[i][j][k][l]) as u64);
					} 
				}
			}
		}
	}
	assignment.input = vec![vec![vec![vec![field::BN254Fr::default();32];32];3];16];
	for i in 0..16 {
		for j in 0..3 {
			for k in 0..32 {
				for l in 0..32 {
					if input.input[i][j][k][l] >= 0{
						assignment.input[i][j][k][l] = field::BN254Fr::from((input.input[i][j][k][l]) as u64);
					} else {
						assignment.input[i][j][k][l] = -field::BN254Fr::from((-input.input[i][j][k][l]) as u64);
					} 
				}
			}
		}
	}
	assignment._features_features_0_conv_Conv_output_0 = vec![vec![vec![vec![field::BN254Fr::default();32];32];64];16];
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_0_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_0_conv_Conv_output_0[i][j][k][l] = field::BN254Fr::from((input._features_features_0_conv_Conv_output_0[i][j][k][l]) as u64);
					} else {
						assignment._features_features_0_conv_Conv_output_0[i][j][k][l] = -field::BN254Fr::from((-input._features_features_0_conv_Conv_output_0[i][j][k][l]) as u64);
					} 
				}
			}
		}
	}
	assignment._features_features_0_Constant_output_0 = field::BN254Fr::default();
	if input._features_features_0_Constant_output_0 >= 0{
		assignment._features_features_0_Constant_output_0 = field::BN254Fr::from((input._features_features_0_Constant_output_0) as u64);
	} else {
		assignment._features_features_0_Constant_output_0 = -field::BN254Fr::from((-input._features_features_0_Constant_output_0) as u64);
	} 
	assignment._features_features_0_Constant_1_output_0 = field::BN254Fr::default();
	if input._features_features_0_Constant_1_output_0 >= 0{
		assignment._features_features_0_Constant_1_output_0 = field::BN254Fr::from((input._features_features_0_Constant_1_output_0) as u64);
	} else {
		assignment._features_features_0_Constant_1_output_0 = -field::BN254Fr::from((-input._features_features_0_Constant_1_output_0) as u64);
	} 
	assignment._features_features_0_Div_output_0_r = vec![vec![vec![vec![field::BN254Fr::default();32];32];64];16];
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_0_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_0_Div_output_0_r[i][j][k][l] = field::BN254Fr::from((input._features_features_0_Div_output_0_r[i][j][k][l]) as u64);
					} else {
						assignment._features_features_0_Div_output_0_r[i][j][k][l] = -field::BN254Fr::from((-input._features_features_0_Div_output_0_r[i][j][k][l]) as u64);
					} 
				}
			}
		}
	}
	assignment._features_features_0_Div_output_0 = vec![vec![vec![vec![field::BN254Fr::default();32];32];64];16];
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_0_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_0_Div_output_0[i][j][k][l] = field::BN254Fr::from((input._features_features_0_Div_output_0[i][j][k][l]) as u64);
					} else {
						assignment._features_features_0_Div_output_0[i][j][k][l] = -field::BN254Fr::from((-input._features_features_0_Div_output_0[i][j][k][l]) as u64);
					} 
				}
			}
		}
	}
	assignment._features_features_0_Constant_2_output_0 = vec![vec![vec![field::BN254Fr::default();32];32];64];
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				if input._features_features_0_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_0_Constant_2_output_0[i][j][k] = field::BN254Fr::from((input._features_features_0_Constant_2_output_0[i][j][k]) as u64);
				} else {
					assignment._features_features_0_Constant_2_output_0[i][j][k] = -field::BN254Fr::from((-input._features_features_0_Constant_2_output_0[i][j][k]) as u64);
				} 
			}
		}
	}
	assignment.features_0_conv_weight = vec![vec![vec![vec![field::BN254Fr::default();3];3];3];64];
	for i in 0..64 {
		for j in 0..3 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_0_conv_weight[i][j][k][l] >= 0{
						assignment.features_0_conv_weight[i][j][k][l] = field::BN254Fr::from((input.features_0_conv_weight[i][j][k][l]) as u64);
					} else {
						assignment.features_0_conv_weight[i][j][k][l] = -field::BN254Fr::from((-input.features_0_conv_weight[i][j][k][l]) as u64);
					} 
				}
			}
		}
	}
}

#[test]
fn expander_witness() -> std::io::Result<()>{ 
	let compile_result = stacker::grow(12 * 1024 * 1024 * 1024, ||
		{
			let mut hint_registry = HintRegistry::<field::BN254Fr>::new();
			hint_registry.register("myhint.querycounthint", query_count_hint);
			hint_registry.register("myhint.rangeproofhint", rangeproof_hint);
			let file = std::fs::File::open("circuit.txt").unwrap();
			let reader = std::io::BufReader::new(file);
			let layered_circuit = expander_compiler::circuit::layered::Circuit::<BN254Config, NormalInputType>::deserialize_from(reader).unwrap();
			let file = std::fs::File::open("witness_solver.txt").unwrap();
			let reader = std::io::BufReader::new(file);
			let witness_solver = expander_compiler::circuit::ir::hint_normalized::witness_solver::WitnessSolver::<BN254Config>::deserialize_from(reader).unwrap();
			let input_str = fs::read_to_string("input.json").unwrap();
			let input: Circuit_Input = serde_json::from_str(&input_str).unwrap();
			let mut assignment = Circuit::<BN254>::default();
			input_copy(&input, &mut assignment);
			let witness = witness_solver.solve_witness_with_hints(&assignment, &mut hint_registry).unwrap();
			println!("Check result:");
			let res = layered_circuit.run(&witness);
			println!("{:?}", res);
			let file = std::fs::File::create("witness.txt").unwrap();
			let writer = std::io::BufWriter::new(file);
			witness.serialize_into(writer).unwrap();
		}
	);
	Ok(())
}
