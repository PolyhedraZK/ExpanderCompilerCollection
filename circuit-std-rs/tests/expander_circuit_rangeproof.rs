// use expander_compiler::frontend::*;
// use expander_compiler::zkcuda::proving_system::DummyProvingSystem;
// use expander_compiler::zkcuda::{context::*, kernel::*};
// use extra::Serde;
// use serde::{Deserialize, Serialize};

// use std::time::Instant;



use std::fs;
use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proof::ComputationGraph;
use expander_compiler::zkcuda::proving_system::{
    ExpanderGKRProvingSystem, ParallelizedExpanderGKRProvingSystem, ProvingSystem,
};
use expander_compiler::zkcuda::{context::*, kernel::*};
use gkr::BN254ConfigSha2Hyrax;
use gkr::BN254ConfigSha2KZG;
use gkr_engine::FieldEngine;
use serdes::ExpSerde;
use serde::{Deserialize, Serialize};

use circuit_std_rs::{
    logup::{query_count_hint, rangeproof_hint, LogUpRangeProofTable},
};

struct Circuit {
	output: Vec<Vec<BN254Fr>>, 
	input: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_Constant_output_0: BN254Fr, 
	_features_features_0_Constant_1_output_0: BN254Fr, 
	_features_features_0_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_2_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_Constant_output_0: BN254Fr, 
	_features_features_3_Constant_1_output_0: BN254Fr, 
	_features_features_3_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_5_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_6_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_7_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_7_Constant_output_0: BN254Fr, 
	_features_features_7_Constant_1_output_0: BN254Fr, 
	_features_features_7_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_7_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_7_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_9_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_10_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_10_Constant_output_0: BN254Fr, 
	_features_features_10_Constant_1_output_0: BN254Fr, 
	_features_features_10_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_10_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_10_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_12_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_13_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_14_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_14_Constant_output_0: BN254Fr, 
	_features_features_14_Constant_1_output_0: BN254Fr, 
	_features_features_14_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_14_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_14_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_16_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_17_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_17_Constant_output_0: BN254Fr, 
	_features_features_17_Constant_1_output_0: BN254Fr, 
	_features_features_17_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_17_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_17_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_19_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_20_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_20_Constant_output_0: BN254Fr, 
	_features_features_20_Constant_1_output_0: BN254Fr, 
	_features_features_20_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_20_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_20_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_22_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_23_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_24_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_24_Constant_output_0: BN254Fr, 
	_features_features_24_Constant_1_output_0: BN254Fr, 
	_features_features_24_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_24_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_24_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_26_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_27_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_27_Constant_output_0: BN254Fr, 
	_features_features_27_Constant_1_output_0: BN254Fr, 
	_features_features_27_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_27_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_27_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_29_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_30_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_30_Constant_output_0: BN254Fr, 
	_features_features_30_Constant_1_output_0: BN254Fr, 
	_features_features_30_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_30_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_30_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_32_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_33_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_34_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_34_Constant_output_0: BN254Fr, 
	_features_features_34_Constant_1_output_0: BN254Fr, 
	_features_features_34_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_34_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_34_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_36_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_37_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_37_Constant_output_0: BN254Fr, 
	_features_features_37_Constant_1_output_0: BN254Fr, 
	_features_features_37_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_37_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_37_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_39_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_40_conv_Conv_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_40_Constant_output_0: BN254Fr, 
	_features_features_40_Constant_1_output_0: BN254Fr, 
	_features_features_40_Div_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_40_Div_output_0_r: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_40_Constant_2_output_0: Vec<Vec<Vec<BN254Fr>>>, 
	_features_features_42_relu_Relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_43_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_classifier_classifier_0_linear_MatMul_output_0: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_0_Constant_output_0: BN254Fr, 
	_classifier_classifier_0_Constant_1_output_0: BN254Fr, 
	_classifier_classifier_0_Div_output_0: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_0_Div_output_0_r: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_0_Constant_2_output_0: Vec<BN254Fr>, 
	_classifier_classifier_1_relu_Relu_output_0: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_3_linear_MatMul_output_0: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_3_Constant_output_0: BN254Fr, 
	_classifier_classifier_3_Constant_1_output_0: BN254Fr, 
	_classifier_classifier_3_Div_output_0: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_3_Div_output_0_r: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_3_Constant_2_output_0: Vec<BN254Fr>, 
	_classifier_classifier_4_relu_Relu_output_0: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_6_linear_MatMul_output_0: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_6_Constant_output_0: BN254Fr, 
	_classifier_classifier_6_Constant_1_output_0: BN254Fr, 
	_classifier_classifier_6_Div_output_0: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_6_Div_output_0_r: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_6_Constant_2_output_0: Vec<BN254Fr>, 
	features_0_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_3_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_7_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_10_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_14_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_17_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_20_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_24_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_27_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_30_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_34_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_37_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	features_40_conv_weight: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx__MatMul_215: Vec<Vec<BN254Fr>>, 
	onnx__MatMul_216: Vec<Vec<BN254Fr>>, 
	onnx__MatMul_217: Vec<Vec<BN254Fr>>, 
}

#[derive(Serialize, Deserialize, Debug)]
struct Circuit_Input {
	output: Vec<Vec<i64>>, 
	input: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_Constant_output_0: i64, 
	_features_features_0_Constant_1_output_0: i64, 
	_features_features_0_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_2_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_Constant_output_0: i64, 
	_features_features_3_Constant_1_output_0: i64, 
	_features_features_3_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_5_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_6_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_7_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_7_Constant_output_0: i64, 
	_features_features_7_Constant_1_output_0: i64, 
	_features_features_7_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_7_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_7_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_9_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_10_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_10_Constant_output_0: i64, 
	_features_features_10_Constant_1_output_0: i64, 
	_features_features_10_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_10_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_10_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_12_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_13_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_14_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_14_Constant_output_0: i64, 
	_features_features_14_Constant_1_output_0: i64, 
	_features_features_14_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_14_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_14_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_16_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_17_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_17_Constant_output_0: i64, 
	_features_features_17_Constant_1_output_0: i64, 
	_features_features_17_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_17_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_17_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_19_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_20_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_20_Constant_output_0: i64, 
	_features_features_20_Constant_1_output_0: i64, 
	_features_features_20_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_20_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_20_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_22_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_23_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_24_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_24_Constant_output_0: i64, 
	_features_features_24_Constant_1_output_0: i64, 
	_features_features_24_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_24_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_24_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_26_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_27_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_27_Constant_output_0: i64, 
	_features_features_27_Constant_1_output_0: i64, 
	_features_features_27_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_27_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_27_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_29_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_30_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_30_Constant_output_0: i64, 
	_features_features_30_Constant_1_output_0: i64, 
	_features_features_30_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_30_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_30_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_32_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_33_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_34_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_34_Constant_output_0: i64, 
	_features_features_34_Constant_1_output_0: i64, 
	_features_features_34_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_34_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_34_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_36_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_37_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_37_Constant_output_0: i64, 
	_features_features_37_Constant_1_output_0: i64, 
	_features_features_37_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_37_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_37_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_39_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_40_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_40_Constant_output_0: i64, 
	_features_features_40_Constant_1_output_0: i64, 
	_features_features_40_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_40_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_40_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_42_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_43_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_classifier_classifier_0_linear_MatMul_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_0_Constant_output_0: i64, 
	_classifier_classifier_0_Constant_1_output_0: i64, 
	_classifier_classifier_0_Div_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_0_Div_output_0_r: Vec<Vec<i64>>, 
	_classifier_classifier_0_Constant_2_output_0: Vec<i64>, 
	_classifier_classifier_1_relu_Relu_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_3_linear_MatMul_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_3_Constant_output_0: i64, 
	_classifier_classifier_3_Constant_1_output_0: i64, 
	_classifier_classifier_3_Div_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_3_Div_output_0_r: Vec<Vec<i64>>, 
	_classifier_classifier_3_Constant_2_output_0: Vec<i64>, 
	_classifier_classifier_4_relu_Relu_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_6_linear_MatMul_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_6_Constant_output_0: i64, 
	_classifier_classifier_6_Constant_1_output_0: i64, 
	_classifier_classifier_6_Div_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_6_Div_output_0_r: Vec<Vec<i64>>, 
	_classifier_classifier_6_Constant_2_output_0: Vec<i64>, 
	features_0_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_3_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_7_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_10_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_14_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_17_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_20_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_24_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_27_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_30_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_34_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_37_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	features_40_conv_weight: Vec<Vec<Vec<Vec<i64>>>>, 
	onnx__MatMul_215: Vec<Vec<i64>>, 
	onnx__MatMul_216: Vec<Vec<i64>>, 
	onnx__MatMul_217: Vec<Vec<i64>>, 
}

fn input_copy(i_input: &Circuit_Input) -> Circuit{
	let mut output = vec![vec![BN254Fr::default();10];16]; 
	for i in 0..16 {
		for j in 0..10 {
			if i_input.output[i][j] >= 0{
				output[i][j] = BN254Fr::from((i_input.output[i][j]) as u64); 
			} else {
				output[i][j] = -BN254Fr::from((-i_input.output[i][j]) as u64); 
			} 
		}
	}
	let mut input = vec![vec![vec![vec![BN254Fr::default();32];32];3];16]; 
	for i in 0..16 {
		for j in 0..3 {
			for k in 0..32 {
				for l in 0..32 {
					if i_input.input[i][j][k][l] >= 0{
						input[i][j][k][l] = BN254Fr::from((i_input.input[i][j][k][l]) as u64); 
					} else {
						input[i][j][k][l] = -BN254Fr::from((-i_input.input[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_0_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if i_input._features_features_0_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_0_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_0_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_0_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_0_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_0_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_0_Constant_output_0 >= 0{
		_features_features_0_Constant_output_0 = BN254Fr::from((i_input._features_features_0_Constant_output_0) as u64); 
	} else {
		_features_features_0_Constant_output_0 = -BN254Fr::from((-i_input._features_features_0_Constant_output_0) as u64); 
	} 
	let mut _features_features_0_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_0_Constant_1_output_0 >= 0{
		_features_features_0_Constant_1_output_0 = BN254Fr::from((i_input._features_features_0_Constant_1_output_0) as u64); 
	} else {
		_features_features_0_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_0_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_0_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if i_input._features_features_0_Div_output_0[i][j][k][l] >= 0{
						_features_features_0_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_0_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_0_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_0_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_0_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if i_input._features_features_0_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_0_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_0_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_0_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_0_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_0_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();32];32];64]; 
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				if i_input._features_features_0_Constant_2_output_0[i][j][k] >= 0{
					_features_features_0_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_0_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_0_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_0_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_2_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if i_input._features_features_2_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_2_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_2_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_2_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_2_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_3_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if i_input._features_features_3_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_3_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_3_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_3_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_3_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_3_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_3_Constant_output_0 >= 0{
		_features_features_3_Constant_output_0 = BN254Fr::from((i_input._features_features_3_Constant_output_0) as u64); 
	} else {
		_features_features_3_Constant_output_0 = -BN254Fr::from((-i_input._features_features_3_Constant_output_0) as u64); 
	} 
	let mut _features_features_3_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_3_Constant_1_output_0 >= 0{
		_features_features_3_Constant_1_output_0 = BN254Fr::from((i_input._features_features_3_Constant_1_output_0) as u64); 
	} else {
		_features_features_3_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_3_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_3_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if i_input._features_features_3_Div_output_0[i][j][k][l] >= 0{
						_features_features_3_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_3_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_3_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_3_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_3_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if i_input._features_features_3_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_3_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_3_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_3_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_3_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_3_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();32];32];64]; 
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				if i_input._features_features_3_Constant_2_output_0[i][j][k] >= 0{
					_features_features_3_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_3_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_3_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_3_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_5_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if i_input._features_features_5_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_5_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_5_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_5_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_5_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_6_maxpool_MaxPool_output_0 = vec![vec![vec![vec![BN254Fr::default();16];16];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..16 {
				for l in 0..16 {
					if i_input._features_features_6_maxpool_MaxPool_output_0[i][j][k][l] >= 0{
						_features_features_6_maxpool_MaxPool_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_6_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_6_maxpool_MaxPool_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_6_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_7_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if i_input._features_features_7_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_7_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_7_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_7_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_7_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_7_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_7_Constant_output_0 >= 0{
		_features_features_7_Constant_output_0 = BN254Fr::from((i_input._features_features_7_Constant_output_0) as u64); 
	} else {
		_features_features_7_Constant_output_0 = -BN254Fr::from((-i_input._features_features_7_Constant_output_0) as u64); 
	} 
	let mut _features_features_7_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_7_Constant_1_output_0 >= 0{
		_features_features_7_Constant_1_output_0 = BN254Fr::from((i_input._features_features_7_Constant_1_output_0) as u64); 
	} else {
		_features_features_7_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_7_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_7_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if i_input._features_features_7_Div_output_0[i][j][k][l] >= 0{
						_features_features_7_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_7_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_7_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_7_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_7_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if i_input._features_features_7_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_7_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_7_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_7_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_7_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_7_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();16];16];128]; 
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				if i_input._features_features_7_Constant_2_output_0[i][j][k] >= 0{
					_features_features_7_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_7_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_7_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_7_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_9_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if i_input._features_features_9_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_9_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_9_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_9_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_9_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_10_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if i_input._features_features_10_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_10_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_10_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_10_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_10_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_10_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_10_Constant_output_0 >= 0{
		_features_features_10_Constant_output_0 = BN254Fr::from((i_input._features_features_10_Constant_output_0) as u64); 
	} else {
		_features_features_10_Constant_output_0 = -BN254Fr::from((-i_input._features_features_10_Constant_output_0) as u64); 
	} 
	let mut _features_features_10_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_10_Constant_1_output_0 >= 0{
		_features_features_10_Constant_1_output_0 = BN254Fr::from((i_input._features_features_10_Constant_1_output_0) as u64); 
	} else {
		_features_features_10_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_10_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_10_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if i_input._features_features_10_Div_output_0[i][j][k][l] >= 0{
						_features_features_10_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_10_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_10_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_10_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_10_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if i_input._features_features_10_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_10_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_10_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_10_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_10_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_10_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();16];16];128]; 
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				if i_input._features_features_10_Constant_2_output_0[i][j][k] >= 0{
					_features_features_10_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_10_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_10_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_10_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_12_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if i_input._features_features_12_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_12_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_12_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_12_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_12_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_13_maxpool_MaxPool_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_13_maxpool_MaxPool_output_0[i][j][k][l] >= 0{
						_features_features_13_maxpool_MaxPool_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_13_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_13_maxpool_MaxPool_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_13_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_14_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_14_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_14_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_14_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_14_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_14_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_14_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_14_Constant_output_0 >= 0{
		_features_features_14_Constant_output_0 = BN254Fr::from((i_input._features_features_14_Constant_output_0) as u64); 
	} else {
		_features_features_14_Constant_output_0 = -BN254Fr::from((-i_input._features_features_14_Constant_output_0) as u64); 
	} 
	let mut _features_features_14_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_14_Constant_1_output_0 >= 0{
		_features_features_14_Constant_1_output_0 = BN254Fr::from((i_input._features_features_14_Constant_1_output_0) as u64); 
	} else {
		_features_features_14_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_14_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_14_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_14_Div_output_0[i][j][k][l] >= 0{
						_features_features_14_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_14_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_14_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_14_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_14_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_14_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_14_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_14_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_14_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_14_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_14_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();8];8];256]; 
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				if i_input._features_features_14_Constant_2_output_0[i][j][k] >= 0{
					_features_features_14_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_14_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_14_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_14_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_16_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_16_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_16_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_16_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_16_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_16_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_17_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_17_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_17_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_17_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_17_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_17_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_17_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_17_Constant_output_0 >= 0{
		_features_features_17_Constant_output_0 = BN254Fr::from((i_input._features_features_17_Constant_output_0) as u64); 
	} else {
		_features_features_17_Constant_output_0 = -BN254Fr::from((-i_input._features_features_17_Constant_output_0) as u64); 
	} 
	let mut _features_features_17_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_17_Constant_1_output_0 >= 0{
		_features_features_17_Constant_1_output_0 = BN254Fr::from((i_input._features_features_17_Constant_1_output_0) as u64); 
	} else {
		_features_features_17_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_17_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_17_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_17_Div_output_0[i][j][k][l] >= 0{
						_features_features_17_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_17_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_17_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_17_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_17_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_17_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_17_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_17_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_17_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_17_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_17_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();8];8];256]; 
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				if i_input._features_features_17_Constant_2_output_0[i][j][k] >= 0{
					_features_features_17_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_17_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_17_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_17_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_19_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_19_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_19_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_19_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_19_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_19_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_20_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_20_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_20_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_20_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_20_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_20_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_20_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_20_Constant_output_0 >= 0{
		_features_features_20_Constant_output_0 = BN254Fr::from((i_input._features_features_20_Constant_output_0) as u64); 
	} else {
		_features_features_20_Constant_output_0 = -BN254Fr::from((-i_input._features_features_20_Constant_output_0) as u64); 
	} 
	let mut _features_features_20_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_20_Constant_1_output_0 >= 0{
		_features_features_20_Constant_1_output_0 = BN254Fr::from((i_input._features_features_20_Constant_1_output_0) as u64); 
	} else {
		_features_features_20_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_20_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_20_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_20_Div_output_0[i][j][k][l] >= 0{
						_features_features_20_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_20_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_20_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_20_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_20_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_20_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_20_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_20_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_20_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_20_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_20_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();8];8];256]; 
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				if i_input._features_features_20_Constant_2_output_0[i][j][k] >= 0{
					_features_features_20_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_20_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_20_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_20_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_22_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if i_input._features_features_22_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_22_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_22_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_22_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_22_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_23_maxpool_MaxPool_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_23_maxpool_MaxPool_output_0[i][j][k][l] >= 0{
						_features_features_23_maxpool_MaxPool_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_23_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_23_maxpool_MaxPool_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_23_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_24_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_24_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_24_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_24_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_24_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_24_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_24_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_24_Constant_output_0 >= 0{
		_features_features_24_Constant_output_0 = BN254Fr::from((i_input._features_features_24_Constant_output_0) as u64); 
	} else {
		_features_features_24_Constant_output_0 = -BN254Fr::from((-i_input._features_features_24_Constant_output_0) as u64); 
	} 
	let mut _features_features_24_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_24_Constant_1_output_0 >= 0{
		_features_features_24_Constant_1_output_0 = BN254Fr::from((i_input._features_features_24_Constant_1_output_0) as u64); 
	} else {
		_features_features_24_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_24_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_24_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_24_Div_output_0[i][j][k][l] >= 0{
						_features_features_24_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_24_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_24_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_24_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_24_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_24_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_24_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_24_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_24_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_24_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_24_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();4];4];512]; 
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				if i_input._features_features_24_Constant_2_output_0[i][j][k] >= 0{
					_features_features_24_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_24_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_24_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_24_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_26_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_26_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_26_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_26_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_26_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_26_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_27_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_27_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_27_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_27_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_27_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_27_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_27_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_27_Constant_output_0 >= 0{
		_features_features_27_Constant_output_0 = BN254Fr::from((i_input._features_features_27_Constant_output_0) as u64); 
	} else {
		_features_features_27_Constant_output_0 = -BN254Fr::from((-i_input._features_features_27_Constant_output_0) as u64); 
	} 
	let mut _features_features_27_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_27_Constant_1_output_0 >= 0{
		_features_features_27_Constant_1_output_0 = BN254Fr::from((i_input._features_features_27_Constant_1_output_0) as u64); 
	} else {
		_features_features_27_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_27_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_27_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_27_Div_output_0[i][j][k][l] >= 0{
						_features_features_27_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_27_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_27_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_27_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_27_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_27_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_27_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_27_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_27_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_27_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_27_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();4];4];512]; 
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				if i_input._features_features_27_Constant_2_output_0[i][j][k] >= 0{
					_features_features_27_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_27_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_27_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_27_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_29_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_29_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_29_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_29_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_29_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_29_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_30_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_30_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_30_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_30_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_30_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_30_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_30_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_30_Constant_output_0 >= 0{
		_features_features_30_Constant_output_0 = BN254Fr::from((i_input._features_features_30_Constant_output_0) as u64); 
	} else {
		_features_features_30_Constant_output_0 = -BN254Fr::from((-i_input._features_features_30_Constant_output_0) as u64); 
	} 
	let mut _features_features_30_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_30_Constant_1_output_0 >= 0{
		_features_features_30_Constant_1_output_0 = BN254Fr::from((i_input._features_features_30_Constant_1_output_0) as u64); 
	} else {
		_features_features_30_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_30_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_30_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_30_Div_output_0[i][j][k][l] >= 0{
						_features_features_30_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_30_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_30_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_30_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_30_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_30_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_30_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_30_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_30_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_30_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_30_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();4];4];512]; 
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				if i_input._features_features_30_Constant_2_output_0[i][j][k] >= 0{
					_features_features_30_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_30_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_30_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_30_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_32_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if i_input._features_features_32_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_32_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_32_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_32_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_32_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_33_maxpool_MaxPool_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_33_maxpool_MaxPool_output_0[i][j][k][l] >= 0{
						_features_features_33_maxpool_MaxPool_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_33_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_33_maxpool_MaxPool_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_33_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_34_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_34_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_34_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_34_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_34_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_34_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_34_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_34_Constant_output_0 >= 0{
		_features_features_34_Constant_output_0 = BN254Fr::from((i_input._features_features_34_Constant_output_0) as u64); 
	} else {
		_features_features_34_Constant_output_0 = -BN254Fr::from((-i_input._features_features_34_Constant_output_0) as u64); 
	} 
	let mut _features_features_34_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_34_Constant_1_output_0 >= 0{
		_features_features_34_Constant_1_output_0 = BN254Fr::from((i_input._features_features_34_Constant_1_output_0) as u64); 
	} else {
		_features_features_34_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_34_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_34_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_34_Div_output_0[i][j][k][l] >= 0{
						_features_features_34_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_34_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_34_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_34_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_34_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_34_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_34_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_34_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_34_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_34_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_34_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();2];2];512]; 
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				if i_input._features_features_34_Constant_2_output_0[i][j][k] >= 0{
					_features_features_34_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_34_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_34_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_34_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_36_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_36_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_36_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_36_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_36_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_36_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_37_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_37_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_37_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_37_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_37_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_37_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_37_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_37_Constant_output_0 >= 0{
		_features_features_37_Constant_output_0 = BN254Fr::from((i_input._features_features_37_Constant_output_0) as u64); 
	} else {
		_features_features_37_Constant_output_0 = -BN254Fr::from((-i_input._features_features_37_Constant_output_0) as u64); 
	} 
	let mut _features_features_37_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_37_Constant_1_output_0 >= 0{
		_features_features_37_Constant_1_output_0 = BN254Fr::from((i_input._features_features_37_Constant_1_output_0) as u64); 
	} else {
		_features_features_37_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_37_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_37_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_37_Div_output_0[i][j][k][l] >= 0{
						_features_features_37_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_37_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_37_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_37_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_37_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_37_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_37_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_37_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_37_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_37_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_37_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();2];2];512]; 
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				if i_input._features_features_37_Constant_2_output_0[i][j][k] >= 0{
					_features_features_37_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_37_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_37_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_37_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_39_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_39_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_39_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_39_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_39_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_39_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_40_conv_Conv_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_40_conv_Conv_output_0[i][j][k][l] >= 0{
						_features_features_40_conv_Conv_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_40_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_40_conv_Conv_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_40_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_40_Constant_output_0 = BN254Fr::default(); 
	if i_input._features_features_40_Constant_output_0 >= 0{
		_features_features_40_Constant_output_0 = BN254Fr::from((i_input._features_features_40_Constant_output_0) as u64); 
	} else {
		_features_features_40_Constant_output_0 = -BN254Fr::from((-i_input._features_features_40_Constant_output_0) as u64); 
	} 
	let mut _features_features_40_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._features_features_40_Constant_1_output_0 >= 0{
		_features_features_40_Constant_1_output_0 = BN254Fr::from((i_input._features_features_40_Constant_1_output_0) as u64); 
	} else {
		_features_features_40_Constant_1_output_0 = -BN254Fr::from((-i_input._features_features_40_Constant_1_output_0) as u64); 
	} 
	let mut _features_features_40_Div_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_40_Div_output_0[i][j][k][l] >= 0{
						_features_features_40_Div_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_40_Div_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_40_Div_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_40_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_40_Div_output_0_r = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_40_Div_output_0_r[i][j][k][l] >= 0{
						_features_features_40_Div_output_0_r[i][j][k][l] = BN254Fr::from((i_input._features_features_40_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						_features_features_40_Div_output_0_r[i][j][k][l] = -BN254Fr::from((-i_input._features_features_40_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_40_Constant_2_output_0 = vec![vec![vec![BN254Fr::default();2];2];512]; 
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				if i_input._features_features_40_Constant_2_output_0[i][j][k] >= 0{
					_features_features_40_Constant_2_output_0[i][j][k] = BN254Fr::from((i_input._features_features_40_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					_features_features_40_Constant_2_output_0[i][j][k] = -BN254Fr::from((-i_input._features_features_40_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	let mut _features_features_42_relu_Relu_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if i_input._features_features_42_relu_Relu_output_0[i][j][k][l] >= 0{
						_features_features_42_relu_Relu_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_42_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_42_relu_Relu_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_42_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _features_features_43_maxpool_MaxPool_output_0 = vec![vec![vec![vec![BN254Fr::default();1];1];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..1 {
				for l in 0..1 {
					if i_input._features_features_43_maxpool_MaxPool_output_0[i][j][k][l] >= 0{
						_features_features_43_maxpool_MaxPool_output_0[i][j][k][l] = BN254Fr::from((i_input._features_features_43_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} else {
						_features_features_43_maxpool_MaxPool_output_0[i][j][k][l] = -BN254Fr::from((-i_input._features_features_43_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut _classifier_classifier_0_linear_MatMul_output_0 = vec![vec![BN254Fr::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if i_input._classifier_classifier_0_linear_MatMul_output_0[i][j] >= 0{
				_classifier_classifier_0_linear_MatMul_output_0[i][j] = BN254Fr::from((i_input._classifier_classifier_0_linear_MatMul_output_0[i][j]) as u64); 
			} else {
				_classifier_classifier_0_linear_MatMul_output_0[i][j] = -BN254Fr::from((-i_input._classifier_classifier_0_linear_MatMul_output_0[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_0_Constant_output_0 = BN254Fr::default(); 
	if i_input._classifier_classifier_0_Constant_output_0 >= 0{
		_classifier_classifier_0_Constant_output_0 = BN254Fr::from((i_input._classifier_classifier_0_Constant_output_0) as u64); 
	} else {
		_classifier_classifier_0_Constant_output_0 = -BN254Fr::from((-i_input._classifier_classifier_0_Constant_output_0) as u64); 
	} 
	let mut _classifier_classifier_0_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._classifier_classifier_0_Constant_1_output_0 >= 0{
		_classifier_classifier_0_Constant_1_output_0 = BN254Fr::from((i_input._classifier_classifier_0_Constant_1_output_0) as u64); 
	} else {
		_classifier_classifier_0_Constant_1_output_0 = -BN254Fr::from((-i_input._classifier_classifier_0_Constant_1_output_0) as u64); 
	} 
	let mut _classifier_classifier_0_Div_output_0 = vec![vec![BN254Fr::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if i_input._classifier_classifier_0_Div_output_0[i][j] >= 0{
				_classifier_classifier_0_Div_output_0[i][j] = BN254Fr::from((i_input._classifier_classifier_0_Div_output_0[i][j]) as u64); 
			} else {
				_classifier_classifier_0_Div_output_0[i][j] = -BN254Fr::from((-i_input._classifier_classifier_0_Div_output_0[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_0_Div_output_0_r = vec![vec![BN254Fr::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if i_input._classifier_classifier_0_Div_output_0_r[i][j] >= 0{
				_classifier_classifier_0_Div_output_0_r[i][j] = BN254Fr::from((i_input._classifier_classifier_0_Div_output_0_r[i][j]) as u64); 
			} else {
				_classifier_classifier_0_Div_output_0_r[i][j] = -BN254Fr::from((-i_input._classifier_classifier_0_Div_output_0_r[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_0_Constant_2_output_0 = vec![BN254Fr::default();512]; 
	for i in 0..512 {
		if i_input._classifier_classifier_0_Constant_2_output_0[i] >= 0{
			_classifier_classifier_0_Constant_2_output_0[i] = BN254Fr::from((i_input._classifier_classifier_0_Constant_2_output_0[i]) as u64); 
		} else {
			_classifier_classifier_0_Constant_2_output_0[i] = -BN254Fr::from((-i_input._classifier_classifier_0_Constant_2_output_0[i]) as u64); 
		} 
	}
	let mut _classifier_classifier_1_relu_Relu_output_0 = vec![vec![BN254Fr::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if i_input._classifier_classifier_1_relu_Relu_output_0[i][j] >= 0{
				_classifier_classifier_1_relu_Relu_output_0[i][j] = BN254Fr::from((i_input._classifier_classifier_1_relu_Relu_output_0[i][j]) as u64); 
			} else {
				_classifier_classifier_1_relu_Relu_output_0[i][j] = -BN254Fr::from((-i_input._classifier_classifier_1_relu_Relu_output_0[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_3_linear_MatMul_output_0 = vec![vec![BN254Fr::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if i_input._classifier_classifier_3_linear_MatMul_output_0[i][j] >= 0{
				_classifier_classifier_3_linear_MatMul_output_0[i][j] = BN254Fr::from((i_input._classifier_classifier_3_linear_MatMul_output_0[i][j]) as u64); 
			} else {
				_classifier_classifier_3_linear_MatMul_output_0[i][j] = -BN254Fr::from((-i_input._classifier_classifier_3_linear_MatMul_output_0[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_3_Constant_output_0 = BN254Fr::default(); 
	if i_input._classifier_classifier_3_Constant_output_0 >= 0{
		_classifier_classifier_3_Constant_output_0 = BN254Fr::from((i_input._classifier_classifier_3_Constant_output_0) as u64); 
	} else {
		_classifier_classifier_3_Constant_output_0 = -BN254Fr::from((-i_input._classifier_classifier_3_Constant_output_0) as u64); 
	} 
	let mut _classifier_classifier_3_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._classifier_classifier_3_Constant_1_output_0 >= 0{
		_classifier_classifier_3_Constant_1_output_0 = BN254Fr::from((i_input._classifier_classifier_3_Constant_1_output_0) as u64); 
	} else {
		_classifier_classifier_3_Constant_1_output_0 = -BN254Fr::from((-i_input._classifier_classifier_3_Constant_1_output_0) as u64); 
	} 
	let mut _classifier_classifier_3_Div_output_0 = vec![vec![BN254Fr::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if i_input._classifier_classifier_3_Div_output_0[i][j] >= 0{
				_classifier_classifier_3_Div_output_0[i][j] = BN254Fr::from((i_input._classifier_classifier_3_Div_output_0[i][j]) as u64); 
			} else {
				_classifier_classifier_3_Div_output_0[i][j] = -BN254Fr::from((-i_input._classifier_classifier_3_Div_output_0[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_3_Div_output_0_r = vec![vec![BN254Fr::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if i_input._classifier_classifier_3_Div_output_0_r[i][j] >= 0{
				_classifier_classifier_3_Div_output_0_r[i][j] = BN254Fr::from((i_input._classifier_classifier_3_Div_output_0_r[i][j]) as u64); 
			} else {
				_classifier_classifier_3_Div_output_0_r[i][j] = -BN254Fr::from((-i_input._classifier_classifier_3_Div_output_0_r[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_3_Constant_2_output_0 = vec![BN254Fr::default();512]; 
	for i in 0..512 {
		if i_input._classifier_classifier_3_Constant_2_output_0[i] >= 0{
			_classifier_classifier_3_Constant_2_output_0[i] = BN254Fr::from((i_input._classifier_classifier_3_Constant_2_output_0[i]) as u64); 
		} else {
			_classifier_classifier_3_Constant_2_output_0[i] = -BN254Fr::from((-i_input._classifier_classifier_3_Constant_2_output_0[i]) as u64); 
		} 
	}
	let mut _classifier_classifier_4_relu_Relu_output_0 = vec![vec![BN254Fr::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if i_input._classifier_classifier_4_relu_Relu_output_0[i][j] >= 0{
				_classifier_classifier_4_relu_Relu_output_0[i][j] = BN254Fr::from((i_input._classifier_classifier_4_relu_Relu_output_0[i][j]) as u64); 
			} else {
				_classifier_classifier_4_relu_Relu_output_0[i][j] = -BN254Fr::from((-i_input._classifier_classifier_4_relu_Relu_output_0[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_6_linear_MatMul_output_0 = vec![vec![BN254Fr::default();10];16]; 
	for i in 0..16 {
		for j in 0..10 {
			if i_input._classifier_classifier_6_linear_MatMul_output_0[i][j] >= 0{
				_classifier_classifier_6_linear_MatMul_output_0[i][j] = BN254Fr::from((i_input._classifier_classifier_6_linear_MatMul_output_0[i][j]) as u64); 
			} else {
				_classifier_classifier_6_linear_MatMul_output_0[i][j] = -BN254Fr::from((-i_input._classifier_classifier_6_linear_MatMul_output_0[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_6_Constant_output_0 = BN254Fr::default(); 
	if i_input._classifier_classifier_6_Constant_output_0 >= 0{
		_classifier_classifier_6_Constant_output_0 = BN254Fr::from((i_input._classifier_classifier_6_Constant_output_0) as u64); 
	} else {
		_classifier_classifier_6_Constant_output_0 = -BN254Fr::from((-i_input._classifier_classifier_6_Constant_output_0) as u64); 
	} 
	let mut _classifier_classifier_6_Constant_1_output_0 = BN254Fr::default(); 
	if i_input._classifier_classifier_6_Constant_1_output_0 >= 0{
		_classifier_classifier_6_Constant_1_output_0 = BN254Fr::from((i_input._classifier_classifier_6_Constant_1_output_0) as u64); 
	} else {
		_classifier_classifier_6_Constant_1_output_0 = -BN254Fr::from((-i_input._classifier_classifier_6_Constant_1_output_0) as u64); 
	} 
	let mut _classifier_classifier_6_Div_output_0 = vec![vec![BN254Fr::default();10];16]; 
	for i in 0..16 {
		for j in 0..10 {
			if i_input._classifier_classifier_6_Div_output_0[i][j] >= 0{
				_classifier_classifier_6_Div_output_0[i][j] = BN254Fr::from((i_input._classifier_classifier_6_Div_output_0[i][j]) as u64); 
			} else {
				_classifier_classifier_6_Div_output_0[i][j] = -BN254Fr::from((-i_input._classifier_classifier_6_Div_output_0[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_6_Div_output_0_r = vec![vec![BN254Fr::default();10];16]; 
	for i in 0..16 {
		for j in 0..10 {
			if i_input._classifier_classifier_6_Div_output_0_r[i][j] >= 0{
				_classifier_classifier_6_Div_output_0_r[i][j] = BN254Fr::from((i_input._classifier_classifier_6_Div_output_0_r[i][j]) as u64); 
			} else {
				_classifier_classifier_6_Div_output_0_r[i][j] = -BN254Fr::from((-i_input._classifier_classifier_6_Div_output_0_r[i][j]) as u64); 
			} 
		}
	}
	let mut _classifier_classifier_6_Constant_2_output_0 = vec![BN254Fr::default();10]; 
	for i in 0..10 {
		if i_input._classifier_classifier_6_Constant_2_output_0[i] >= 0{
			_classifier_classifier_6_Constant_2_output_0[i] = BN254Fr::from((i_input._classifier_classifier_6_Constant_2_output_0[i]) as u64); 
		} else {
			_classifier_classifier_6_Constant_2_output_0[i] = -BN254Fr::from((-i_input._classifier_classifier_6_Constant_2_output_0[i]) as u64); 
		} 
	}
	let mut features_0_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];3];64]; 
	for i in 0..64 {
		for j in 0..3 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_0_conv_weight[i][j][k][l] >= 0{
						features_0_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_0_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_0_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_0_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_3_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];64];64]; 
	for i in 0..64 {
		for j in 0..64 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_3_conv_weight[i][j][k][l] >= 0{
						features_3_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_3_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_3_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_3_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_7_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];64];128]; 
	for i in 0..128 {
		for j in 0..64 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_7_conv_weight[i][j][k][l] >= 0{
						features_7_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_7_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_7_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_7_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_10_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];128];128]; 
	for i in 0..128 {
		for j in 0..128 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_10_conv_weight[i][j][k][l] >= 0{
						features_10_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_10_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_10_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_10_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_14_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];128];256]; 
	for i in 0..256 {
		for j in 0..128 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_14_conv_weight[i][j][k][l] >= 0{
						features_14_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_14_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_14_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_14_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_17_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];256];256]; 
	for i in 0..256 {
		for j in 0..256 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_17_conv_weight[i][j][k][l] >= 0{
						features_17_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_17_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_17_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_17_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_20_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];256];256]; 
	for i in 0..256 {
		for j in 0..256 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_20_conv_weight[i][j][k][l] >= 0{
						features_20_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_20_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_20_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_20_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_24_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];256];512]; 
	for i in 0..512 {
		for j in 0..256 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_24_conv_weight[i][j][k][l] >= 0{
						features_24_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_24_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_24_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_24_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_27_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_27_conv_weight[i][j][k][l] >= 0{
						features_27_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_27_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_27_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_27_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_30_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_30_conv_weight[i][j][k][l] >= 0{
						features_30_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_30_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_30_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_30_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_34_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_34_conv_weight[i][j][k][l] >= 0{
						features_34_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_34_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_34_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_34_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_37_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_37_conv_weight[i][j][k][l] >= 0{
						features_37_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_37_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_37_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_37_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut features_40_conv_weight = vec![vec![vec![vec![BN254Fr::default();3];3];512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			for k in 0..3 {
				for l in 0..3 {
					if i_input.features_40_conv_weight[i][j][k][l] >= 0{
						features_40_conv_weight[i][j][k][l] = BN254Fr::from((i_input.features_40_conv_weight[i][j][k][l]) as u64); 
					} else {
						features_40_conv_weight[i][j][k][l] = -BN254Fr::from((-i_input.features_40_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	let mut onnx__MatMul_215 = vec![vec![BN254Fr::default();512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			if i_input.onnx__MatMul_215[i][j] >= 0{
				onnx__MatMul_215[i][j] = BN254Fr::from((i_input.onnx__MatMul_215[i][j]) as u64); 
			} else {
				onnx__MatMul_215[i][j] = -BN254Fr::from((-i_input.onnx__MatMul_215[i][j]) as u64); 
			} 
		}
	}
	let mut onnx__MatMul_216 = vec![vec![BN254Fr::default();512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			if i_input.onnx__MatMul_216[i][j] >= 0{
				onnx__MatMul_216[i][j] = BN254Fr::from((i_input.onnx__MatMul_216[i][j]) as u64); 
			} else {
				onnx__MatMul_216[i][j] = -BN254Fr::from((-i_input.onnx__MatMul_216[i][j]) as u64); 
			} 
		}
	}
	let mut onnx__MatMul_217 = vec![vec![BN254Fr::default();10];512]; 
	for i in 0..512 {
		for j in 0..10 {
			if i_input.onnx__MatMul_217[i][j] >= 0{
				onnx__MatMul_217[i][j] = BN254Fr::from((i_input.onnx__MatMul_217[i][j]) as u64); 
			} else {
				onnx__MatMul_217[i][j] = -BN254Fr::from((-i_input.onnx__MatMul_217[i][j]) as u64); 
			} 
		}
	}
	let ass = Circuit{output,input,_features_features_0_conv_Conv_output_0,_features_features_0_Constant_output_0,_features_features_0_Constant_1_output_0,_features_features_0_Div_output_0,_features_features_0_Div_output_0_r,_features_features_0_Constant_2_output_0,_features_features_2_relu_Relu_output_0,_features_features_3_conv_Conv_output_0,_features_features_3_Constant_output_0,_features_features_3_Constant_1_output_0,_features_features_3_Div_output_0,_features_features_3_Div_output_0_r,_features_features_3_Constant_2_output_0,_features_features_5_relu_Relu_output_0,_features_features_6_maxpool_MaxPool_output_0,_features_features_7_conv_Conv_output_0,_features_features_7_Constant_output_0,_features_features_7_Constant_1_output_0,_features_features_7_Div_output_0,_features_features_7_Div_output_0_r,_features_features_7_Constant_2_output_0,_features_features_9_relu_Relu_output_0,_features_features_10_conv_Conv_output_0,_features_features_10_Constant_output_0,_features_features_10_Constant_1_output_0,_features_features_10_Div_output_0,_features_features_10_Div_output_0_r,_features_features_10_Constant_2_output_0,_features_features_12_relu_Relu_output_0,_features_features_13_maxpool_MaxPool_output_0,_features_features_14_conv_Conv_output_0,_features_features_14_Constant_output_0,_features_features_14_Constant_1_output_0,_features_features_14_Div_output_0,_features_features_14_Div_output_0_r,_features_features_14_Constant_2_output_0,_features_features_16_relu_Relu_output_0,_features_features_17_conv_Conv_output_0,_features_features_17_Constant_output_0,_features_features_17_Constant_1_output_0,_features_features_17_Div_output_0,_features_features_17_Div_output_0_r,_features_features_17_Constant_2_output_0,_features_features_19_relu_Relu_output_0,_features_features_20_conv_Conv_output_0,_features_features_20_Constant_output_0,_features_features_20_Constant_1_output_0,_features_features_20_Div_output_0,_features_features_20_Div_output_0_r,_features_features_20_Constant_2_output_0,_features_features_22_relu_Relu_output_0,_features_features_23_maxpool_MaxPool_output_0,_features_features_24_conv_Conv_output_0,_features_features_24_Constant_output_0,_features_features_24_Constant_1_output_0,_features_features_24_Div_output_0,_features_features_24_Div_output_0_r,_features_features_24_Constant_2_output_0,_features_features_26_relu_Relu_output_0,_features_features_27_conv_Conv_output_0,_features_features_27_Constant_output_0,_features_features_27_Constant_1_output_0,_features_features_27_Div_output_0,_features_features_27_Div_output_0_r,_features_features_27_Constant_2_output_0,_features_features_29_relu_Relu_output_0,_features_features_30_conv_Conv_output_0,_features_features_30_Constant_output_0,_features_features_30_Constant_1_output_0,_features_features_30_Div_output_0,_features_features_30_Div_output_0_r,_features_features_30_Constant_2_output_0,_features_features_32_relu_Relu_output_0,_features_features_33_maxpool_MaxPool_output_0,_features_features_34_conv_Conv_output_0,_features_features_34_Constant_output_0,_features_features_34_Constant_1_output_0,_features_features_34_Div_output_0,_features_features_34_Div_output_0_r,_features_features_34_Constant_2_output_0,_features_features_36_relu_Relu_output_0,_features_features_37_conv_Conv_output_0,_features_features_37_Constant_output_0,_features_features_37_Constant_1_output_0,_features_features_37_Div_output_0,_features_features_37_Div_output_0_r,_features_features_37_Constant_2_output_0,_features_features_39_relu_Relu_output_0,_features_features_40_conv_Conv_output_0,_features_features_40_Constant_output_0,_features_features_40_Constant_1_output_0,_features_features_40_Div_output_0,_features_features_40_Div_output_0_r,_features_features_40_Constant_2_output_0,_features_features_42_relu_Relu_output_0,_features_features_43_maxpool_MaxPool_output_0,_classifier_classifier_0_linear_MatMul_output_0,_classifier_classifier_0_Constant_output_0,_classifier_classifier_0_Constant_1_output_0,_classifier_classifier_0_Div_output_0,_classifier_classifier_0_Div_output_0_r,_classifier_classifier_0_Constant_2_output_0,_classifier_classifier_1_relu_Relu_output_0,_classifier_classifier_3_linear_MatMul_output_0,_classifier_classifier_3_Constant_output_0,_classifier_classifier_3_Constant_1_output_0,_classifier_classifier_3_Div_output_0,_classifier_classifier_3_Div_output_0_r,_classifier_classifier_3_Constant_2_output_0,_classifier_classifier_4_relu_Relu_output_0,_classifier_classifier_6_linear_MatMul_output_0,_classifier_classifier_6_Constant_output_0,_classifier_classifier_6_Constant_1_output_0,_classifier_classifier_6_Div_output_0,_classifier_classifier_6_Div_output_0_r,_classifier_classifier_6_Constant_2_output_0,features_0_conv_weight,features_3_conv_weight,features_7_conv_weight,features_10_conv_weight,features_14_conv_weight,features_17_conv_weight,features_20_conv_weight,features_24_conv_weight,features_27_conv_weight,features_30_conv_weight,features_34_conv_weight,features_37_conv_weight,features_40_conv_weight,onnx__MatMul_215,onnx__MatMul_216,onnx__MatMul_217};
	ass
}

		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_0_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_conv_Conv_output_0: &[[[InputVariable;32];32];64],
	_features_features_0_Constant_output_0: &InputVariable,
	_features_features_0_Mul_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_0_Mul_output_0[i][j][k] = api.mul(_features_features_0_conv_Conv_output_0[i][j][k], _features_features_0_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_0_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_Mul_output_0: &[[[InputVariable;32];32];64],
	_features_features_0_Constant_1_output_0: &InputVariable,
	_features_features_0_Div_output_0: &[[[InputVariable;32];32];64],
	_features_features_0_Div_output_0_r: &[[[InputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				let tmp1 = api.mul(_features_features_0_Div_output_0[i][j][k], _features_features_0_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_0_Mul_output_0[i][j][k], _features_features_0_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_0_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_Div_output_0: &[[[InputVariable;32];32];64],
	_features_features_0_Cast_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_0_Cast_output_0[i][j][k] = _features_features_0_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_0_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_Cast_output_0: &[[[InputVariable;32];32];64],
	_features_features_0_Cast_1_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_0_Cast_1_output_0[i][j][k] = _features_features_0_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_0_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_Cast_1_output_0: &[[[InputVariable;32];32];64],
	_features_features_0_Constant_2_output_0: &[[[InputVariable;32];32];64],
	_features_features_0_Add_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_0_Add_output_0[i][j][k] = api.add(_features_features_0_Cast_1_output_0[i][j][k], _features_features_0_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_2_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_Add_output_0: &[[[InputVariable;32];32];64],
	_features_features_2_relu_Cast_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_2_relu_Cast_output_0[i][j][k] = _features_features_0_Add_output_0[i][j][k];
			}
		}
	}
}

#[kernel]		// cast operation
fn _features_features_2_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_2_relu_Relu_output_0: &[[[InputVariable;32];32];64],
	_features_features_2_relu_Cast_1_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_2_relu_Cast_1_output_0[i][j][k] = _features_features_2_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
	// relu operation
#[kernel]		// cast operation
fn _features_features_2_relu_rangeproof_cast0_macro<C: Config>(
	api: &mut API<C>,
	_features_features_2_relu_Relu_output_0: &[[[InputVariable;32];32];64],
	_features_features_2_relu_Cast_output_0: &[[[InputVariable;32];32];64],
	_features_features_5_relu_Relu_output_0: &[[[InputVariable;32];32];64],
	_features_features_5_relu_Cast_output_0: &[[[InputVariable;32];32];64],
	_features_features_9_relu_Relu_output_0: &[[[InputVariable;16];16];128],
	_features_features_9_relu_Cast_output_0: &[[[InputVariable;16];16];128],
	_features_features_12_relu_Relu_output_0: &[[[InputVariable;16];16];128],
	_features_features_12_relu_Cast_output_0: &[[[InputVariable;16];16];128],
	_features_features_16_relu_Relu_output_0: &[[[InputVariable;8];8];256],
	_features_features_16_relu_Cast_output_0: &[[[InputVariable;8];8];256],
	_features_features_19_relu_Relu_output_0: &[[[InputVariable;8];8];256],
	_features_features_19_relu_Cast_output_0: &[[[InputVariable;8];8];256],
	_features_features_22_relu_Relu_output_0: &[[[InputVariable;8];8];256],
	_features_features_22_relu_Cast_output_0: &[[[InputVariable;8];8];256],
	_features_features_26_relu_Relu_output_0: &[[[InputVariable;4];4];512],
	_features_features_26_relu_Cast_output_0: &[[[InputVariable;4];4];512],
	_features_features_29_relu_Relu_output_0: &[[[InputVariable;4];4];512],
	_features_features_29_relu_Cast_output_0: &[[[InputVariable;4];4];512],
	_features_features_32_relu_Relu_output_0: &[[[InputVariable;4];4];512],
	_features_features_32_relu_Cast_output_0: &[[[InputVariable;4];4];512],
	_features_features_36_relu_Relu_output_0: &[[[InputVariable;2];2];512],
	_features_features_36_relu_Cast_output_0: &[[[InputVariable;2];2];512],
	_features_features_39_relu_Relu_output_0: &[[[InputVariable;2];2];512],
	_features_features_39_relu_Cast_output_0: &[[[InputVariable;2];2];512],
	_features_features_42_relu_Relu_output_0: &[[[InputVariable;2];2];512],
	_features_features_42_relu_Cast_output_0: &[[[InputVariable;2];2];512],
	_classifier_classifier_1_relu_Relu_output_0: &[InputVariable;512],
	_classifier_classifier_1_relu_Cast_output_0: &[InputVariable;512],
	_classifier_classifier_4_relu_Relu_output_0: &[InputVariable;512],
	_classifier_classifier_4_relu_Cast_output_0: &[InputVariable;512],
	// divide operation
	//_features_features_0_Div_output_0:  &[[[InputVariable;32];32];64],,
) {
	let mut table = LogUpRangeProofTable::new(16);
    table.initial(api);

	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				let features_2_relu_tmp1 = api.add(_features_features_2_relu_Relu_output_0[i][j][k], _features_features_2_relu_Relu_output_0[i][j][k]);
				let features_2_relu_tmp2 = api.sub(features_2_relu_tmp1, _features_features_2_relu_Cast_output_0[i][j][k]);
				table.rangeproof(api, features_2_relu_tmp2, 16);
				//_features_features_2_relu_Cast_1_output_0[i][j][k] = _features_features_2_relu_Relu_output_0[i][j][k];

				let features_5_relu_tmp1 = api.add(_features_features_5_relu_Relu_output_0[i][j][k], _features_features_5_relu_Relu_output_0[i][j][k]);
				let features_5_relu_tmp2 = api.sub(features_5_relu_tmp1, _features_features_5_relu_Cast_output_0[i][j][k]);
				table.rangeproof(api, features_5_relu_tmp2, 16);
			}
		}
	}


	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				let features_9_relu_tmp1 = api.add(_features_features_9_relu_Relu_output_0[i][j][k], _features_features_9_relu_Relu_output_0[i][j][k]);
				let features_9_relu_tmp2 = api.sub(features_9_relu_tmp1, _features_features_9_relu_Cast_output_0[i][j][k]);
				table.rangeproof(api, features_9_relu_tmp2, 16);

				let features_12_relu_tmp1 = api.add(_features_features_12_relu_Relu_output_0[i][j][k], _features_features_12_relu_Relu_output_0[i][j][k]);
				let features_12_relu_tmp2 = api.sub(features_12_relu_tmp1, _features_features_12_relu_Cast_output_0[i][j][k]);
				table.rangeproof(api, features_12_relu_tmp2, 16);
			}
		}
	}

	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				let features_26_relu_tmp1 = api.add(_features_features_26_relu_Relu_output_0[i][j][k], _features_features_26_relu_Relu_output_0[i][j][k]);
				let features_26_relu_tmp2 = api.sub(features_26_relu_tmp1, _features_features_26_relu_Cast_output_0[i][j][k]);
				table.rangeproof(api, features_26_relu_tmp2, 16);

				let features_29_relu_tmp1 = api.add(_features_features_29_relu_Relu_output_0[i][j][k], _features_features_29_relu_Relu_output_0[i][j][k]);
				let features_29_relu_tmp2 = api.sub(features_29_relu_tmp1, _features_features_29_relu_Cast_output_0[i][j][k]);
				table.rangeproof(api, features_29_relu_tmp2, 16);

				let features_32_relu_tmp1 = api.add(_features_features_32_relu_Relu_output_0[i][j][k], _features_features_32_relu_Relu_output_0[i][j][k]);
				let features_32_relu_tmp2 = api.sub(features_32_relu_tmp1, _features_features_32_relu_Cast_output_0[i][j][k]);
				table.rangeproof(api, features_32_relu_tmp2, 16);

			}
		}
	}

	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				
				let features_36_relu_tmp1 = api.add(_features_features_36_relu_Relu_output_0[i][j][k], _features_features_36_relu_Relu_output_0[i][j][k]);
				let features_36_relu_tmp2 = api.sub(features_36_relu_tmp1, _features_features_36_relu_Cast_output_0[i][j][k]);
				table.rangeproof(api, features_36_relu_tmp2, 16);

				let features_39_relu_tmp1 = api.add(_features_features_39_relu_Relu_output_0[i][j][k], _features_features_39_relu_Relu_output_0[i][j][k]);
				let features_39_relu_tmp2 = api.sub(features_39_relu_tmp1, _features_features_39_relu_Cast_output_0[i][j][k]);
				table.rangeproof(api, features_39_relu_tmp2, 16);

				let features_42_relu_tmp1 = api.add(_features_features_42_relu_Relu_output_0[i][j][k], _features_features_42_relu_Relu_output_0[i][j][k]);
				let features_42_relu_tmp2 = api.sub(features_42_relu_tmp1, _features_features_42_relu_Cast_output_0[i][j][k]);
				table.rangeproof(api, features_42_relu_tmp2, 16);
			}
		}
	}

	for i in 0..512 {
		let classifier_1_relu_tmp1 = api.add(_classifier_classifier_1_relu_Relu_output_0[i], _classifier_classifier_1_relu_Relu_output_0[i]);
		let classifier_1_reluu_tmp2 = api.sub(classifier_1_relu_tmp1, _classifier_classifier_1_relu_Cast_output_0[i]);
		table.rangeproof(api, classifier_1_reluu_tmp2, 16);

		let classifier_4_relu_tmp1 = api.add(_classifier_classifier_4_relu_Relu_output_0[i], _classifier_classifier_4_relu_Relu_output_0[i]);
		let classifier_4_relu_tmp2 = api.sub(classifier_4_relu_tmp1, _classifier_classifier_4_relu_Cast_output_0[i]);
		table.rangeproof(api, classifier_4_relu_tmp2, 16);
	}

	table.final_check(api);

 }
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_3_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_3_conv_Conv_output_0: &[[[InputVariable;32];32];64],
	_features_features_3_Constant_output_0: &InputVariable,
	_features_features_3_Mul_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_3_Mul_output_0[i][j][k] = api.mul(_features_features_3_conv_Conv_output_0[i][j][k], _features_features_3_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_3_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_3_Mul_output_0: &[[[InputVariable;32];32];64],
	_features_features_3_Constant_1_output_0: &InputVariable,
	_features_features_3_Div_output_0: &[[[InputVariable;32];32];64],
	_features_features_3_Div_output_0_r: &[[[InputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				let tmp1 = api.mul(_features_features_3_Div_output_0[i][j][k], _features_features_3_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_3_Mul_output_0[i][j][k], _features_features_3_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_3_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_3_Div_output_0: &[[[InputVariable;32];32];64],
	_features_features_3_Cast_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_3_Cast_output_0[i][j][k] = _features_features_3_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_3_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_3_Cast_output_0: &[[[InputVariable;32];32];64],
	_features_features_3_Cast_1_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_3_Cast_1_output_0[i][j][k] = _features_features_3_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_3_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_3_Cast_1_output_0: &[[[InputVariable;32];32];64],
	_features_features_3_Constant_2_output_0: &[[[InputVariable;32];32];64],
	_features_features_3_Add_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_3_Add_output_0[i][j][k] = api.add(_features_features_3_Cast_1_output_0[i][j][k], _features_features_3_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_5_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_3_Add_output_0: &[[[InputVariable;32];32];64],
	_features_features_5_relu_Cast_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_5_relu_Cast_output_0[i][j][k] = _features_features_3_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_5_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_5_relu_Relu_output_0: &[[[InputVariable;32];32];64],
	_features_features_5_relu_Cast_1_output_0: &mut [[[OutputVariable;32];32];64],
) {
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				_features_features_5_relu_Cast_1_output_0[i][j][k] = _features_features_5_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// maxpool operation
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_7_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_7_conv_Conv_output_0: &[[[InputVariable;16];16];128],
	_features_features_7_Constant_output_0: &InputVariable,
	_features_features_7_Mul_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_7_Mul_output_0[i][j][k] = api.mul(_features_features_7_conv_Conv_output_0[i][j][k], _features_features_7_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_7_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_7_Mul_output_0: &[[[InputVariable;16];16];128],
	_features_features_7_Constant_1_output_0: &InputVariable,
	_features_features_7_Div_output_0: &[[[InputVariable;16];16];128],
	_features_features_7_Div_output_0_r: &[[[InputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				let tmp1 = api.mul(_features_features_7_Div_output_0[i][j][k], _features_features_7_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_7_Mul_output_0[i][j][k], _features_features_7_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_7_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_7_Div_output_0: &[[[InputVariable;16];16];128],
	_features_features_7_Cast_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_7_Cast_output_0[i][j][k] = _features_features_7_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_7_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_7_Cast_output_0: &[[[InputVariable;16];16];128],
	_features_features_7_Cast_1_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_7_Cast_1_output_0[i][j][k] = _features_features_7_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_7_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_7_Cast_1_output_0: &[[[InputVariable;16];16];128],
	_features_features_7_Constant_2_output_0: &[[[InputVariable;16];16];128],
	_features_features_7_Add_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_7_Add_output_0[i][j][k] = api.add(_features_features_7_Cast_1_output_0[i][j][k], _features_features_7_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_9_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_7_Add_output_0: &[[[InputVariable;16];16];128],
	_features_features_9_relu_Cast_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_9_relu_Cast_output_0[i][j][k] = _features_features_7_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_9_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_9_relu_Relu_output_0: &[[[InputVariable;16];16];128],
	_features_features_9_relu_Cast_1_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_9_relu_Cast_1_output_0[i][j][k] = _features_features_9_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_10_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_10_conv_Conv_output_0: &[[[InputVariable;16];16];128],
	_features_features_10_Constant_output_0: &InputVariable,
	_features_features_10_Mul_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_10_Mul_output_0[i][j][k] = api.mul(_features_features_10_conv_Conv_output_0[i][j][k], _features_features_10_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_10_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_10_Mul_output_0: &[[[InputVariable;16];16];128],
	_features_features_10_Constant_1_output_0: &InputVariable,
	_features_features_10_Div_output_0: &[[[InputVariable;16];16];128],
	_features_features_10_Div_output_0_r: &[[[InputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				let tmp1 = api.mul(_features_features_10_Div_output_0[i][j][k], _features_features_10_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_10_Mul_output_0[i][j][k], _features_features_10_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_10_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_10_Div_output_0: &[[[InputVariable;16];16];128],
	_features_features_10_Cast_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_10_Cast_output_0[i][j][k] = _features_features_10_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_10_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_10_Cast_output_0: &[[[InputVariable;16];16];128],
	_features_features_10_Cast_1_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_10_Cast_1_output_0[i][j][k] = _features_features_10_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_10_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_10_Cast_1_output_0: &[[[InputVariable;16];16];128],
	_features_features_10_Constant_2_output_0: &[[[InputVariable;16];16];128],
	_features_features_10_Add_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_10_Add_output_0[i][j][k] = api.add(_features_features_10_Cast_1_output_0[i][j][k], _features_features_10_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_12_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_10_Add_output_0: &[[[InputVariable;16];16];128],
	_features_features_12_relu_Cast_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_12_relu_Cast_output_0[i][j][k] = _features_features_10_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_12_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_12_relu_Relu_output_0: &[[[InputVariable;16];16];128],
	_features_features_12_relu_Cast_1_output_0: &mut [[[OutputVariable;16];16];128],
) {
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				_features_features_12_relu_Cast_1_output_0[i][j][k] = _features_features_12_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// maxpool operation
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_14_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_14_conv_Conv_output_0: &[[[InputVariable;8];8];256],
	_features_features_14_Constant_output_0: &InputVariable,
	_features_features_14_Mul_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_14_Mul_output_0[i][j][k] = api.mul(_features_features_14_conv_Conv_output_0[i][j][k], _features_features_14_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_14_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_14_Mul_output_0: &[[[InputVariable;8];8];256],
	_features_features_14_Constant_1_output_0: &InputVariable,
	_features_features_14_Div_output_0: &[[[InputVariable;8];8];256],
	_features_features_14_Div_output_0_r: &[[[InputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				let tmp1 = api.mul(_features_features_14_Div_output_0[i][j][k], _features_features_14_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_14_Mul_output_0[i][j][k], _features_features_14_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_14_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_14_Div_output_0: &[[[InputVariable;8];8];256],
	_features_features_14_Cast_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_14_Cast_output_0[i][j][k] = _features_features_14_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_14_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_14_Cast_output_0: &[[[InputVariable;8];8];256],
	_features_features_14_Cast_1_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_14_Cast_1_output_0[i][j][k] = _features_features_14_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_14_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_14_Cast_1_output_0: &[[[InputVariable;8];8];256],
	_features_features_14_Constant_2_output_0: &[[[InputVariable;8];8];256],
	_features_features_14_Add_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_14_Add_output_0[i][j][k] = api.add(_features_features_14_Cast_1_output_0[i][j][k], _features_features_14_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_16_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_14_Add_output_0: &[[[InputVariable;8];8];256],
	_features_features_16_relu_Cast_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_16_relu_Cast_output_0[i][j][k] = _features_features_14_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_16_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_16_relu_Relu_output_0: &[[[InputVariable;8];8];256],
	_features_features_16_relu_Cast_1_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_16_relu_Cast_1_output_0[i][j][k] = _features_features_16_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_17_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_17_conv_Conv_output_0: &[[[InputVariable;8];8];256],
	_features_features_17_Constant_output_0: &InputVariable,
	_features_features_17_Mul_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_17_Mul_output_0[i][j][k] = api.mul(_features_features_17_conv_Conv_output_0[i][j][k], _features_features_17_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_17_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_17_Mul_output_0: &[[[InputVariable;8];8];256],
	_features_features_17_Constant_1_output_0: &InputVariable,
	_features_features_17_Div_output_0: &[[[InputVariable;8];8];256],
	_features_features_17_Div_output_0_r: &[[[InputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				let tmp1 = api.mul(_features_features_17_Div_output_0[i][j][k], _features_features_17_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_17_Mul_output_0[i][j][k], _features_features_17_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_17_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_17_Div_output_0: &[[[InputVariable;8];8];256],
	_features_features_17_Cast_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_17_Cast_output_0[i][j][k] = _features_features_17_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_17_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_17_Cast_output_0: &[[[InputVariable;8];8];256],
	_features_features_17_Cast_1_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_17_Cast_1_output_0[i][j][k] = _features_features_17_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_17_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_17_Cast_1_output_0: &[[[InputVariable;8];8];256],
	_features_features_17_Constant_2_output_0: &[[[InputVariable;8];8];256],
	_features_features_17_Add_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_17_Add_output_0[i][j][k] = api.add(_features_features_17_Cast_1_output_0[i][j][k], _features_features_17_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_19_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_17_Add_output_0: &[[[InputVariable;8];8];256],
	_features_features_19_relu_Cast_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_19_relu_Cast_output_0[i][j][k] = _features_features_17_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_19_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_19_relu_Relu_output_0: &[[[InputVariable;8];8];256],
	_features_features_19_relu_Cast_1_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_19_relu_Cast_1_output_0[i][j][k] = _features_features_19_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_20_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_20_conv_Conv_output_0: &[[[InputVariable;8];8];256],
	_features_features_20_Constant_output_0: &InputVariable,
	_features_features_20_Mul_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_20_Mul_output_0[i][j][k] = api.mul(_features_features_20_conv_Conv_output_0[i][j][k], _features_features_20_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_20_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_20_Mul_output_0: &[[[InputVariable;8];8];256],
	_features_features_20_Constant_1_output_0: &InputVariable,
	_features_features_20_Div_output_0: &[[[InputVariable;8];8];256],
	_features_features_20_Div_output_0_r: &[[[InputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				let tmp1 = api.mul(_features_features_20_Div_output_0[i][j][k], _features_features_20_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_20_Mul_output_0[i][j][k], _features_features_20_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_20_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_20_Div_output_0: &[[[InputVariable;8];8];256],
	_features_features_20_Cast_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_20_Cast_output_0[i][j][k] = _features_features_20_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_20_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_20_Cast_output_0: &[[[InputVariable;8];8];256],
	_features_features_20_Cast_1_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_20_Cast_1_output_0[i][j][k] = _features_features_20_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_20_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_20_Cast_1_output_0: &[[[InputVariable;8];8];256],
	_features_features_20_Constant_2_output_0: &[[[InputVariable;8];8];256],
	_features_features_20_Add_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_20_Add_output_0[i][j][k] = api.add(_features_features_20_Cast_1_output_0[i][j][k], _features_features_20_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_22_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_20_Add_output_0: &[[[InputVariable;8];8];256],
	_features_features_22_relu_Cast_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_22_relu_Cast_output_0[i][j][k] = _features_features_20_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_22_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_22_relu_Relu_output_0: &[[[InputVariable;8];8];256],
	_features_features_22_relu_Cast_1_output_0: &mut [[[OutputVariable;8];8];256],
) {
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				_features_features_22_relu_Cast_1_output_0[i][j][k] = _features_features_22_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// maxpool operation
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_24_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_24_conv_Conv_output_0: &[[[InputVariable;4];4];512],
	_features_features_24_Constant_output_0: &InputVariable,
	_features_features_24_Mul_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_24_Mul_output_0[i][j][k] = api.mul(_features_features_24_conv_Conv_output_0[i][j][k], _features_features_24_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_24_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_24_Mul_output_0: &[[[InputVariable;4];4];512],
	_features_features_24_Constant_1_output_0: &InputVariable,
	_features_features_24_Div_output_0: &[[[InputVariable;4];4];512],
	_features_features_24_Div_output_0_r: &[[[InputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				let tmp1 = api.mul(_features_features_24_Div_output_0[i][j][k], _features_features_24_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_24_Mul_output_0[i][j][k], _features_features_24_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_24_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_24_Div_output_0: &[[[InputVariable;4];4];512],
	_features_features_24_Cast_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_24_Cast_output_0[i][j][k] = _features_features_24_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_24_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_24_Cast_output_0: &[[[InputVariable;4];4];512],
	_features_features_24_Cast_1_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_24_Cast_1_output_0[i][j][k] = _features_features_24_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_24_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_24_Cast_1_output_0: &[[[InputVariable;4];4];512],
	_features_features_24_Constant_2_output_0: &[[[InputVariable;4];4];512],
	_features_features_24_Add_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_24_Add_output_0[i][j][k] = api.add(_features_features_24_Cast_1_output_0[i][j][k], _features_features_24_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_26_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_24_Add_output_0: &[[[InputVariable;4];4];512],
	_features_features_26_relu_Cast_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_26_relu_Cast_output_0[i][j][k] = _features_features_24_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_26_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_26_relu_Relu_output_0: &[[[InputVariable;4];4];512],
	_features_features_26_relu_Cast_1_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_26_relu_Cast_1_output_0[i][j][k] = _features_features_26_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_27_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_27_conv_Conv_output_0: &[[[InputVariable;4];4];512],
	_features_features_27_Constant_output_0: &InputVariable,
	_features_features_27_Mul_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_27_Mul_output_0[i][j][k] = api.mul(_features_features_27_conv_Conv_output_0[i][j][k], _features_features_27_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_27_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_27_Mul_output_0: &[[[InputVariable;4];4];512],
	_features_features_27_Constant_1_output_0: &InputVariable,
	_features_features_27_Div_output_0: &[[[InputVariable;4];4];512],
	_features_features_27_Div_output_0_r: &[[[InputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				let tmp1 = api.mul(_features_features_27_Div_output_0[i][j][k], _features_features_27_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_27_Mul_output_0[i][j][k], _features_features_27_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_27_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_27_Div_output_0: &[[[InputVariable;4];4];512],
	_features_features_27_Cast_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_27_Cast_output_0[i][j][k] = _features_features_27_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_27_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_27_Cast_output_0: &[[[InputVariable;4];4];512],
	_features_features_27_Cast_1_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_27_Cast_1_output_0[i][j][k] = _features_features_27_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_27_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_27_Cast_1_output_0: &[[[InputVariable;4];4];512],
	_features_features_27_Constant_2_output_0: &[[[InputVariable;4];4];512],
	_features_features_27_Add_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_27_Add_output_0[i][j][k] = api.add(_features_features_27_Cast_1_output_0[i][j][k], _features_features_27_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_29_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_27_Add_output_0: &[[[InputVariable;4];4];512],
	_features_features_29_relu_Cast_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_29_relu_Cast_output_0[i][j][k] = _features_features_27_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_29_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_29_relu_Relu_output_0: &[[[InputVariable;4];4];512],
	_features_features_29_relu_Cast_1_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_29_relu_Cast_1_output_0[i][j][k] = _features_features_29_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_30_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_30_conv_Conv_output_0: &[[[InputVariable;4];4];512],
	_features_features_30_Constant_output_0: &InputVariable,
	_features_features_30_Mul_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_30_Mul_output_0[i][j][k] = api.mul(_features_features_30_conv_Conv_output_0[i][j][k], _features_features_30_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_30_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_30_Mul_output_0: &[[[InputVariable;4];4];512],
	_features_features_30_Constant_1_output_0: &InputVariable,
	_features_features_30_Div_output_0: &[[[InputVariable;4];4];512],
	_features_features_30_Div_output_0_r: &[[[InputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				let tmp1 = api.mul(_features_features_30_Div_output_0[i][j][k], _features_features_30_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_30_Mul_output_0[i][j][k], _features_features_30_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_30_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_30_Div_output_0: &[[[InputVariable;4];4];512],
	_features_features_30_Cast_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_30_Cast_output_0[i][j][k] = _features_features_30_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_30_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_30_Cast_output_0: &[[[InputVariable;4];4];512],
	_features_features_30_Cast_1_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_30_Cast_1_output_0[i][j][k] = _features_features_30_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_30_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_30_Cast_1_output_0: &[[[InputVariable;4];4];512],
	_features_features_30_Constant_2_output_0: &[[[InputVariable;4];4];512],
	_features_features_30_Add_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_30_Add_output_0[i][j][k] = api.add(_features_features_30_Cast_1_output_0[i][j][k], _features_features_30_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_32_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_30_Add_output_0: &[[[InputVariable;4];4];512],
	_features_features_32_relu_Cast_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_32_relu_Cast_output_0[i][j][k] = _features_features_30_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_32_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_32_relu_Relu_output_0: &[[[InputVariable;4];4];512],
	_features_features_32_relu_Cast_1_output_0: &mut [[[OutputVariable;4];4];512],
) {
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				_features_features_32_relu_Cast_1_output_0[i][j][k] = _features_features_32_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// maxpool operation
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_34_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_34_conv_Conv_output_0: &[[[InputVariable;2];2];512],
	_features_features_34_Constant_output_0: &InputVariable,
	_features_features_34_Mul_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_34_Mul_output_0[i][j][k] = api.mul(_features_features_34_conv_Conv_output_0[i][j][k], _features_features_34_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_34_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_34_Mul_output_0: &[[[InputVariable;2];2];512],
	_features_features_34_Constant_1_output_0: &InputVariable,
	_features_features_34_Div_output_0: &[[[InputVariable;2];2];512],
	_features_features_34_Div_output_0_r: &[[[InputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				let tmp1 = api.mul(_features_features_34_Div_output_0[i][j][k], _features_features_34_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_34_Mul_output_0[i][j][k], _features_features_34_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_34_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_34_Div_output_0: &[[[InputVariable;2];2];512],
	_features_features_34_Cast_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_34_Cast_output_0[i][j][k] = _features_features_34_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_34_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_34_Cast_output_0: &[[[InputVariable;2];2];512],
	_features_features_34_Cast_1_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_34_Cast_1_output_0[i][j][k] = _features_features_34_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_34_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_34_Cast_1_output_0: &[[[InputVariable;2];2];512],
	_features_features_34_Constant_2_output_0: &[[[InputVariable;2];2];512],
	_features_features_34_Add_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_34_Add_output_0[i][j][k] = api.add(_features_features_34_Cast_1_output_0[i][j][k], _features_features_34_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_36_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_34_Add_output_0: &[[[InputVariable;2];2];512],
	_features_features_36_relu_Cast_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_36_relu_Cast_output_0[i][j][k] = _features_features_34_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_36_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_36_relu_Relu_output_0: &[[[InputVariable;2];2];512],
	_features_features_36_relu_Cast_1_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_36_relu_Cast_1_output_0[i][j][k] = _features_features_36_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_37_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_37_conv_Conv_output_0: &[[[InputVariable;2];2];512],
	_features_features_37_Constant_output_0: &InputVariable,
	_features_features_37_Mul_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_37_Mul_output_0[i][j][k] = api.mul(_features_features_37_conv_Conv_output_0[i][j][k], _features_features_37_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_37_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_37_Mul_output_0: &[[[InputVariable;2];2];512],
	_features_features_37_Constant_1_output_0: &InputVariable,
	_features_features_37_Div_output_0: &[[[InputVariable;2];2];512],
	_features_features_37_Div_output_0_r: &[[[InputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				let tmp1 = api.mul(_features_features_37_Div_output_0[i][j][k], _features_features_37_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_37_Mul_output_0[i][j][k], _features_features_37_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_37_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_37_Div_output_0: &[[[InputVariable;2];2];512],
	_features_features_37_Cast_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_37_Cast_output_0[i][j][k] = _features_features_37_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_37_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_37_Cast_output_0: &[[[InputVariable;2];2];512],
	_features_features_37_Cast_1_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_37_Cast_1_output_0[i][j][k] = _features_features_37_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_37_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_37_Cast_1_output_0: &[[[InputVariable;2];2];512],
	_features_features_37_Constant_2_output_0: &[[[InputVariable;2];2];512],
	_features_features_37_Add_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_37_Add_output_0[i][j][k] = api.add(_features_features_37_Cast_1_output_0[i][j][k], _features_features_37_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_39_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_37_Add_output_0: &[[[InputVariable;2];2];512],
	_features_features_39_relu_Cast_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_39_relu_Cast_output_0[i][j][k] = _features_features_37_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_39_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_39_relu_Relu_output_0: &[[[InputVariable;2];2];512],
	_features_features_39_relu_Cast_1_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_39_relu_Cast_1_output_0[i][j][k] = _features_features_39_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// conv operation
		// constant operation
#[kernel]		// multiply operation
fn _features_features_40_Mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_40_conv_Conv_output_0: &[[[InputVariable;2];2];512],
	_features_features_40_Constant_output_0: &InputVariable,
	_features_features_40_Mul_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_40_Mul_output_0[i][j][k] = api.mul(_features_features_40_conv_Conv_output_0[i][j][k], _features_features_40_Constant_output_0);
			}
		}
	}
}
		// constant operation
#[kernel]		// divide operation
fn _features_features_40_Div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_40_Mul_output_0: &[[[InputVariable;2];2];512],
	_features_features_40_Constant_1_output_0: &InputVariable,
	_features_features_40_Div_output_0: &[[[InputVariable;2];2];512],
	_features_features_40_Div_output_0_r: &[[[InputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				let tmp1 = api.mul(_features_features_40_Div_output_0[i][j][k], _features_features_40_Constant_1_output_0);
				let tmp2 = api.sub(_features_features_40_Mul_output_0[i][j][k], _features_features_40_Div_output_0_r[i][j][k]);
				api.assert_is_equal(tmp1, tmp2);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_40_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_40_Div_output_0: &[[[InputVariable;2];2];512],
	_features_features_40_Cast_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_40_Cast_output_0[i][j][k] = _features_features_40_Div_output_0[i][j][k];
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_40_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_40_Cast_output_0: &[[[InputVariable;2];2];512],
	_features_features_40_Cast_1_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_40_Cast_1_output_0[i][j][k] = _features_features_40_Cast_output_0[i][j][k];
			}
		}
	}
}
		// constant operation
#[kernel]		// add operation
fn _features_features_40_Add_macro<C: Config>(
	api: &mut API<C>,
	_features_features_40_Cast_1_output_0: &[[[InputVariable;2];2];512],
	_features_features_40_Constant_2_output_0: &[[[InputVariable;2];2];512],
	_features_features_40_Add_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_40_Add_output_0[i][j][k] = api.add(_features_features_40_Cast_1_output_0[i][j][k], _features_features_40_Constant_2_output_0[i][j][k]);
			}
		}
	}
}
#[kernel]		// cast operation
fn _features_features_42_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_features_features_40_Add_output_0: &[[[InputVariable;2];2];512],
	_features_features_42_relu_Cast_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_42_relu_Cast_output_0[i][j][k] = _features_features_40_Add_output_0[i][j][k];
			}
		}
	}
}
		// relu operation
#[kernel]		// cast operation
fn _features_features_42_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_features_features_42_relu_Relu_output_0: &[[[InputVariable;2];2];512],
	_features_features_42_relu_Cast_1_output_0: &mut [[[OutputVariable;2];2];512],
) {
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				_features_features_42_relu_Cast_1_output_0[i][j][k] = _features_features_42_relu_Relu_output_0[i][j][k];
			}
		}
	}
}
		// maxpool operation
		// flatten operation
		// matmul operation
		// constant operation
#[kernel]		// multiply operation
fn _classifier_classifier_0_Mul_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_0_linear_MatMul_output_0: &[InputVariable;512],
	_classifier_classifier_0_Constant_output_0: &InputVariable,
	_classifier_classifier_0_Mul_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_0_Mul_output_0[i] = api.mul(_classifier_classifier_0_linear_MatMul_output_0[i], _classifier_classifier_0_Constant_output_0);
	}
}
		// constant operation
#[kernel]		// divide operation
fn _classifier_classifier_0_Div_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_0_Mul_output_0: &[InputVariable;512],
	_classifier_classifier_0_Constant_1_output_0: &InputVariable,
	_classifier_classifier_0_Div_output_0: &[InputVariable;512],
	_classifier_classifier_0_Div_output_0_r: &[InputVariable;512],
) {
	for i in 0..512 {
		let tmp1 = api.mul(_classifier_classifier_0_Div_output_0[i], _classifier_classifier_0_Constant_1_output_0);
		let tmp2 = api.sub(_classifier_classifier_0_Mul_output_0[i], _classifier_classifier_0_Div_output_0_r[i]);
		api.assert_is_equal(tmp1, tmp2);
	}
}
#[kernel]		// cast operation
fn _classifier_classifier_0_Cast_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_0_Div_output_0: &[InputVariable;512],
	_classifier_classifier_0_Cast_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_0_Cast_output_0[i] = _classifier_classifier_0_Div_output_0[i];
	}
}
#[kernel]		// cast operation
fn _classifier_classifier_0_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_0_Cast_output_0: &[InputVariable;512],
	_classifier_classifier_0_Cast_1_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_0_Cast_1_output_0[i] = _classifier_classifier_0_Cast_output_0[i];
	}
}
		// constant operation
#[kernel]		// add operation
fn _classifier_classifier_0_Add_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_0_Cast_1_output_0: &[InputVariable;512],
	_classifier_classifier_0_Constant_2_output_0: &[InputVariable;512],
	_classifier_classifier_0_Add_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_0_Add_output_0[i] = api.add(_classifier_classifier_0_Cast_1_output_0[i], _classifier_classifier_0_Constant_2_output_0[i]);
	}
}
#[kernel]		// cast operation
fn _classifier_classifier_1_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_0_Add_output_0: &[InputVariable;512],
	_classifier_classifier_1_relu_Cast_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_1_relu_Cast_output_0[i] = _classifier_classifier_0_Add_output_0[i];
	}
}
		// relu operation
#[kernel]		// cast operation
fn _classifier_classifier_1_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_1_relu_Relu_output_0: &[InputVariable;512],
	_classifier_classifier_1_relu_Cast_1_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_1_relu_Cast_1_output_0[i] = _classifier_classifier_1_relu_Relu_output_0[i];
	}
}
		// matmul operation
		// constant operation
#[kernel]		// multiply operation
fn _classifier_classifier_3_Mul_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_3_linear_MatMul_output_0: &[InputVariable;512],
	_classifier_classifier_3_Constant_output_0: &InputVariable,
	_classifier_classifier_3_Mul_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_3_Mul_output_0[i] = api.mul(_classifier_classifier_3_linear_MatMul_output_0[i], _classifier_classifier_3_Constant_output_0);
	}
}
		// constant operation
#[kernel]		// divide operation
fn _classifier_classifier_3_Div_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_3_Mul_output_0: &[InputVariable;512],
	_classifier_classifier_3_Constant_1_output_0: &InputVariable,
	_classifier_classifier_3_Div_output_0: &[InputVariable;512],
	_classifier_classifier_3_Div_output_0_r: &[InputVariable;512],
) {
	for i in 0..512 {
		let tmp1 = api.mul(_classifier_classifier_3_Div_output_0[i], _classifier_classifier_3_Constant_1_output_0);
		let tmp2 = api.sub(_classifier_classifier_3_Mul_output_0[i], _classifier_classifier_3_Div_output_0_r[i]);
		api.assert_is_equal(tmp1, tmp2);
	}
}
#[kernel]		// cast operation
fn _classifier_classifier_3_Cast_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_3_Div_output_0: &[InputVariable;512],
	_classifier_classifier_3_Cast_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_3_Cast_output_0[i] = _classifier_classifier_3_Div_output_0[i];
	}
}
#[kernel]		// cast operation
fn _classifier_classifier_3_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_3_Cast_output_0: &[InputVariable;512],
	_classifier_classifier_3_Cast_1_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_3_Cast_1_output_0[i] = _classifier_classifier_3_Cast_output_0[i];
	}
}
		// constant operation
#[kernel]		// add operation
fn _classifier_classifier_3_Add_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_3_Cast_1_output_0: &[InputVariable;512],
	_classifier_classifier_3_Constant_2_output_0: &[InputVariable;512],
	_classifier_classifier_3_Add_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_3_Add_output_0[i] = api.add(_classifier_classifier_3_Cast_1_output_0[i], _classifier_classifier_3_Constant_2_output_0[i]);
	}
}
#[kernel]		// cast operation
fn _classifier_classifier_4_relu_Cast_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_3_Add_output_0: &[InputVariable;512],
	_classifier_classifier_4_relu_Cast_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_4_relu_Cast_output_0[i] = _classifier_classifier_3_Add_output_0[i];
	}
}
		// relu operation
#[kernel]		// cast operation
fn _classifier_classifier_4_relu_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_4_relu_Relu_output_0: &[InputVariable;512],
	_classifier_classifier_4_relu_Cast_1_output_0: &mut [OutputVariable;512],
) {
	for i in 0..512 {
		_classifier_classifier_4_relu_Cast_1_output_0[i] = _classifier_classifier_4_relu_Relu_output_0[i];
	}
}
		// matmul operation
		// constant operation
#[kernel]		// multiply operation
fn _classifier_classifier_6_Mul_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_6_linear_MatMul_output_0: &[InputVariable;10],
	_classifier_classifier_6_Constant_output_0: &InputVariable,
	_classifier_classifier_6_Mul_output_0: &mut [OutputVariable;10],
) {
	for i in 0..10 {
		_classifier_classifier_6_Mul_output_0[i] = api.mul(_classifier_classifier_6_linear_MatMul_output_0[i], _classifier_classifier_6_Constant_output_0);
	}
}
		// constant operation
#[kernel]		// divide operation
fn _classifier_classifier_6_Div_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_6_Mul_output_0: &[InputVariable;10],
	_classifier_classifier_6_Constant_1_output_0: &InputVariable,
	_classifier_classifier_6_Div_output_0: &[InputVariable;10],
	_classifier_classifier_6_Div_output_0_r: &[InputVariable;10],
) {
	for i in 0..10 {
		let tmp1 = api.mul(_classifier_classifier_6_Div_output_0[i], _classifier_classifier_6_Constant_1_output_0);
		let tmp2 = api.sub(_classifier_classifier_6_Mul_output_0[i], _classifier_classifier_6_Div_output_0_r[i]);
		api.assert_is_equal(tmp1, tmp2);
	}
}
#[kernel]		// cast operation
fn _classifier_classifier_6_Cast_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_6_Div_output_0: &[InputVariable;10],
	_classifier_classifier_6_Cast_output_0: &mut [OutputVariable;10],
) {
	for i in 0..10 {
		_classifier_classifier_6_Cast_output_0[i] = _classifier_classifier_6_Div_output_0[i];
	}
}
#[kernel]		// cast operation
fn _classifier_classifier_6_Cast_1_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_6_Cast_output_0: &[InputVariable;10],
	_classifier_classifier_6_Cast_1_output_0: &mut [OutputVariable;10],
) {
	for i in 0..10 {
		_classifier_classifier_6_Cast_1_output_0[i] = _classifier_classifier_6_Cast_output_0[i];
	}
}
		// constant operation
#[kernel]		// add operation
fn _classifier_classifier_6_Add_macro<C: Config>(
	api: &mut API<C>,
	_classifier_classifier_6_Cast_1_output_0: &[InputVariable;10],
	_classifier_classifier_6_Constant_2_output_0: &[InputVariable;10],
	output: &mut [OutputVariable;10],
) {
	for i in 0..10 {
		output[i] = api.add(_classifier_classifier_6_Cast_1_output_0[i], _classifier_classifier_6_Constant_2_output_0[i]);
	}
}

// #[kernel]
// fn rangeproof_test_kernel<C: Config>(builder: &mut API<C>, test: &InputVariable) {
//     let mut table = LogUpRangeProofTable::new(8);
//     table.initial(builder);
//     table.rangeproof(builder, *test, 10);
//     table.final_check(builder);
// }

#[test]
fn expander_circuit() -> std::io::Result<()>{ 
	let compile_result = stacker::grow(32 * 1024 * 1024 * 1024, ||
		{
			let kernel__features_features_0_Mul: Kernel<BN254Config> = compile__features_features_0_Mul_macro().unwrap();
			let kernel__features_features_0_Div: Kernel<BN254Config> = compile__features_features_0_Div_macro().unwrap();
			let kernel__features_features_0_Cast: Kernel<BN254Config> = compile__features_features_0_Cast_macro().unwrap();
			let kernel__features_features_0_Cast_1: Kernel<BN254Config> = compile__features_features_0_Cast_1_macro().unwrap();
			let kernel__features_features_0_Add: Kernel<BN254Config> = compile__features_features_0_Add_macro().unwrap();
			let kernel__features_features_2_relu_Cast: Kernel<BN254Config> = compile__features_features_2_relu_Cast_macro().unwrap();
			let kernel__features_features_2_relu_Cast_1: Kernel<BN254Config> = compile__features_features_2_relu_Cast_1_macro().unwrap();
			let kernel__features_features_2_relu_rangeproof_cast0: Kernel<BN254Config> = compile__features_features_2_relu_rangeproof_cast0_macro().unwrap();

			let kernel__features_features_3_Mul: Kernel<BN254Config> = compile__features_features_3_Mul_macro().unwrap();
			let kernel__features_features_3_Div: Kernel<BN254Config> = compile__features_features_3_Div_macro().unwrap();
			let kernel__features_features_3_Cast: Kernel<BN254Config> = compile__features_features_3_Cast_macro().unwrap();
			let kernel__features_features_3_Cast_1: Kernel<BN254Config> = compile__features_features_3_Cast_1_macro().unwrap();
			let kernel__features_features_3_Add: Kernel<BN254Config> = compile__features_features_3_Add_macro().unwrap();
			let kernel__features_features_5_relu_Cast: Kernel<BN254Config> = compile__features_features_5_relu_Cast_macro().unwrap();
			let kernel__features_features_5_relu_Cast_1: Kernel<BN254Config> = compile__features_features_5_relu_Cast_1_macro().unwrap();
			let kernel__features_features_7_Mul: Kernel<BN254Config> = compile__features_features_7_Mul_macro().unwrap();
			let kernel__features_features_7_Div: Kernel<BN254Config> = compile__features_features_7_Div_macro().unwrap();
			let kernel__features_features_7_Cast: Kernel<BN254Config> = compile__features_features_7_Cast_macro().unwrap();
			let kernel__features_features_7_Cast_1: Kernel<BN254Config> = compile__features_features_7_Cast_1_macro().unwrap();
			let kernel__features_features_7_Add: Kernel<BN254Config> = compile__features_features_7_Add_macro().unwrap();
			let kernel__features_features_9_relu_Cast: Kernel<BN254Config> = compile__features_features_9_relu_Cast_macro().unwrap();
			let kernel__features_features_9_relu_Cast_1: Kernel<BN254Config> = compile__features_features_9_relu_Cast_1_macro().unwrap();
			let kernel__features_features_10_Mul: Kernel<BN254Config> = compile__features_features_10_Mul_macro().unwrap();
			let kernel__features_features_10_Div: Kernel<BN254Config> = compile__features_features_10_Div_macro().unwrap();
			let kernel__features_features_10_Cast: Kernel<BN254Config> = compile__features_features_10_Cast_macro().unwrap();
			let kernel__features_features_10_Cast_1: Kernel<BN254Config> = compile__features_features_10_Cast_1_macro().unwrap();
			let kernel__features_features_10_Add: Kernel<BN254Config> = compile__features_features_10_Add_macro().unwrap();
			let kernel__features_features_12_relu_Cast: Kernel<BN254Config> = compile__features_features_12_relu_Cast_macro().unwrap();
			let kernel__features_features_12_relu_Cast_1: Kernel<BN254Config> = compile__features_features_12_relu_Cast_1_macro().unwrap();
			let kernel__features_features_14_Mul: Kernel<BN254Config> = compile__features_features_14_Mul_macro().unwrap();
			let kernel__features_features_14_Div: Kernel<BN254Config> = compile__features_features_14_Div_macro().unwrap();
			let kernel__features_features_14_Cast: Kernel<BN254Config> = compile__features_features_14_Cast_macro().unwrap();
			let kernel__features_features_14_Cast_1: Kernel<BN254Config> = compile__features_features_14_Cast_1_macro().unwrap();
			let kernel__features_features_14_Add: Kernel<BN254Config> = compile__features_features_14_Add_macro().unwrap();
			let kernel__features_features_16_relu_Cast: Kernel<BN254Config> = compile__features_features_16_relu_Cast_macro().unwrap();
			let kernel__features_features_16_relu_Cast_1: Kernel<BN254Config> = compile__features_features_16_relu_Cast_1_macro().unwrap();
			let kernel__features_features_17_Mul: Kernel<BN254Config> = compile__features_features_17_Mul_macro().unwrap();
			let kernel__features_features_17_Div: Kernel<BN254Config> = compile__features_features_17_Div_macro().unwrap();
			let kernel__features_features_17_Cast: Kernel<BN254Config> = compile__features_features_17_Cast_macro().unwrap();
			let kernel__features_features_17_Cast_1: Kernel<BN254Config> = compile__features_features_17_Cast_1_macro().unwrap();
			let kernel__features_features_17_Add: Kernel<BN254Config> = compile__features_features_17_Add_macro().unwrap();
			let kernel__features_features_19_relu_Cast: Kernel<BN254Config> = compile__features_features_19_relu_Cast_macro().unwrap();
			let kernel__features_features_19_relu_Cast_1: Kernel<BN254Config> = compile__features_features_19_relu_Cast_1_macro().unwrap();
			let kernel__features_features_20_Mul: Kernel<BN254Config> = compile__features_features_20_Mul_macro().unwrap();
			let kernel__features_features_20_Div: Kernel<BN254Config> = compile__features_features_20_Div_macro().unwrap();
			let kernel__features_features_20_Cast: Kernel<BN254Config> = compile__features_features_20_Cast_macro().unwrap();
			let kernel__features_features_20_Cast_1: Kernel<BN254Config> = compile__features_features_20_Cast_1_macro().unwrap();
			let kernel__features_features_20_Add: Kernel<BN254Config> = compile__features_features_20_Add_macro().unwrap();
			let kernel__features_features_22_relu_Cast: Kernel<BN254Config> = compile__features_features_22_relu_Cast_macro().unwrap();
			let kernel__features_features_22_relu_Cast_1: Kernel<BN254Config> = compile__features_features_22_relu_Cast_1_macro().unwrap();
			let kernel__features_features_24_Mul: Kernel<BN254Config> = compile__features_features_24_Mul_macro().unwrap();
			let kernel__features_features_24_Div: Kernel<BN254Config> = compile__features_features_24_Div_macro().unwrap();
			let kernel__features_features_24_Cast: Kernel<BN254Config> = compile__features_features_24_Cast_macro().unwrap();
			let kernel__features_features_24_Cast_1: Kernel<BN254Config> = compile__features_features_24_Cast_1_macro().unwrap();
			let kernel__features_features_24_Add: Kernel<BN254Config> = compile__features_features_24_Add_macro().unwrap();
			let kernel__features_features_26_relu_Cast: Kernel<BN254Config> = compile__features_features_26_relu_Cast_macro().unwrap();
			let kernel__features_features_26_relu_Cast_1: Kernel<BN254Config> = compile__features_features_26_relu_Cast_1_macro().unwrap();
			let kernel__features_features_27_Mul: Kernel<BN254Config> = compile__features_features_27_Mul_macro().unwrap();
			let kernel__features_features_27_Div: Kernel<BN254Config> = compile__features_features_27_Div_macro().unwrap();
			let kernel__features_features_27_Cast: Kernel<BN254Config> = compile__features_features_27_Cast_macro().unwrap();
			let kernel__features_features_27_Cast_1: Kernel<BN254Config> = compile__features_features_27_Cast_1_macro().unwrap();
			let kernel__features_features_27_Add: Kernel<BN254Config> = compile__features_features_27_Add_macro().unwrap();
			let kernel__features_features_29_relu_Cast: Kernel<BN254Config> = compile__features_features_29_relu_Cast_macro().unwrap();
			let kernel__features_features_29_relu_Cast_1: Kernel<BN254Config> = compile__features_features_29_relu_Cast_1_macro().unwrap();
			let kernel__features_features_30_Mul: Kernel<BN254Config> = compile__features_features_30_Mul_macro().unwrap();
			let kernel__features_features_30_Div: Kernel<BN254Config> = compile__features_features_30_Div_macro().unwrap();
			let kernel__features_features_30_Cast: Kernel<BN254Config> = compile__features_features_30_Cast_macro().unwrap();
			let kernel__features_features_30_Cast_1: Kernel<BN254Config> = compile__features_features_30_Cast_1_macro().unwrap();
			let kernel__features_features_30_Add: Kernel<BN254Config> = compile__features_features_30_Add_macro().unwrap();
			let kernel__features_features_32_relu_Cast: Kernel<BN254Config> = compile__features_features_32_relu_Cast_macro().unwrap();
			let kernel__features_features_32_relu_Cast_1: Kernel<BN254Config> = compile__features_features_32_relu_Cast_1_macro().unwrap();
			let kernel__features_features_34_Mul: Kernel<BN254Config> = compile__features_features_34_Mul_macro().unwrap();
			let kernel__features_features_34_Div: Kernel<BN254Config> = compile__features_features_34_Div_macro().unwrap();
			let kernel__features_features_34_Cast: Kernel<BN254Config> = compile__features_features_34_Cast_macro().unwrap();
			let kernel__features_features_34_Cast_1: Kernel<BN254Config> = compile__features_features_34_Cast_1_macro().unwrap();
			let kernel__features_features_34_Add: Kernel<BN254Config> = compile__features_features_34_Add_macro().unwrap();
			let kernel__features_features_36_relu_Cast: Kernel<BN254Config> = compile__features_features_36_relu_Cast_macro().unwrap();
			let kernel__features_features_36_relu_Cast_1: Kernel<BN254Config> = compile__features_features_36_relu_Cast_1_macro().unwrap();
			let kernel__features_features_37_Mul: Kernel<BN254Config> = compile__features_features_37_Mul_macro().unwrap();
			let kernel__features_features_37_Div: Kernel<BN254Config> = compile__features_features_37_Div_macro().unwrap();
			let kernel__features_features_37_Cast: Kernel<BN254Config> = compile__features_features_37_Cast_macro().unwrap();
			let kernel__features_features_37_Cast_1: Kernel<BN254Config> = compile__features_features_37_Cast_1_macro().unwrap();
			let kernel__features_features_37_Add: Kernel<BN254Config> = compile__features_features_37_Add_macro().unwrap();
			let kernel__features_features_39_relu_Cast: Kernel<BN254Config> = compile__features_features_39_relu_Cast_macro().unwrap();
			let kernel__features_features_39_relu_Cast_1: Kernel<BN254Config> = compile__features_features_39_relu_Cast_1_macro().unwrap();
			let kernel__features_features_40_Mul: Kernel<BN254Config> = compile__features_features_40_Mul_macro().unwrap();
			let kernel__features_features_40_Div: Kernel<BN254Config> = compile__features_features_40_Div_macro().unwrap();
			let kernel__features_features_40_Cast: Kernel<BN254Config> = compile__features_features_40_Cast_macro().unwrap();
			let kernel__features_features_40_Cast_1: Kernel<BN254Config> = compile__features_features_40_Cast_1_macro().unwrap();
			let kernel__features_features_40_Add: Kernel<BN254Config> = compile__features_features_40_Add_macro().unwrap();
			let kernel__features_features_42_relu_Cast: Kernel<BN254Config> = compile__features_features_42_relu_Cast_macro().unwrap();
			let kernel__features_features_42_relu_Cast_1: Kernel<BN254Config> = compile__features_features_42_relu_Cast_1_macro().unwrap();
			let kernel__classifier_classifier_0_Mul: Kernel<BN254Config> = compile__classifier_classifier_0_Mul_macro().unwrap();
			let kernel__classifier_classifier_0_Div: Kernel<BN254Config> = compile__classifier_classifier_0_Div_macro().unwrap();
			let kernel__classifier_classifier_0_Cast: Kernel<BN254Config> = compile__classifier_classifier_0_Cast_macro().unwrap();
			let kernel__classifier_classifier_0_Cast_1: Kernel<BN254Config> = compile__classifier_classifier_0_Cast_1_macro().unwrap();
			let kernel__classifier_classifier_0_Add: Kernel<BN254Config> = compile__classifier_classifier_0_Add_macro().unwrap();
			let kernel__classifier_classifier_1_relu_Cast: Kernel<BN254Config> = compile__classifier_classifier_1_relu_Cast_macro().unwrap();
			let kernel__classifier_classifier_1_relu_Cast_1: Kernel<BN254Config> = compile__classifier_classifier_1_relu_Cast_1_macro().unwrap();
			let kernel__classifier_classifier_3_Mul: Kernel<BN254Config> = compile__classifier_classifier_3_Mul_macro().unwrap();
			let kernel__classifier_classifier_3_Div: Kernel<BN254Config> = compile__classifier_classifier_3_Div_macro().unwrap();
			let kernel__classifier_classifier_3_Cast: Kernel<BN254Config> = compile__classifier_classifier_3_Cast_macro().unwrap();
			let kernel__classifier_classifier_3_Cast_1: Kernel<BN254Config> = compile__classifier_classifier_3_Cast_1_macro().unwrap();
			let kernel__classifier_classifier_3_Add: Kernel<BN254Config> = compile__classifier_classifier_3_Add_macro().unwrap();
			let kernel__classifier_classifier_4_relu_Cast: Kernel<BN254Config> = compile__classifier_classifier_4_relu_Cast_macro().unwrap();
			let kernel__classifier_classifier_4_relu_Cast_1: Kernel<BN254Config> = compile__classifier_classifier_4_relu_Cast_1_macro().unwrap();
			let kernel__classifier_classifier_6_Mul: Kernel<BN254Config> = compile__classifier_classifier_6_Mul_macro().unwrap();
			let kernel__classifier_classifier_6_Div: Kernel<BN254Config> = compile__classifier_classifier_6_Div_macro().unwrap();
			let kernel__classifier_classifier_6_Cast: Kernel<BN254Config> = compile__classifier_classifier_6_Cast_macro().unwrap();
			let kernel__classifier_classifier_6_Cast_1: Kernel<BN254Config> = compile__classifier_classifier_6_Cast_1_macro().unwrap();
			let kernel__classifier_classifier_6_Add: Kernel<BN254Config> = compile__classifier_classifier_6_Add_macro().unwrap();
			
			// hint registry for
			let mut hint_registry = HintRegistry::<BN254Fr>::new();
			hint_registry.register("myhint.querycounthint", query_count_hint);
			hint_registry.register("myhint.rangeproofhint", rangeproof_hint);
			let mut ctx: Context<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>, HintRegistry<BN254Fr>> = Context::new(hint_registry);
			//let mut ctx: Context<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2KZG>> = Context::new(EmptyHintCaller::new());
			//let mut ctx: Context<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>> = Context::new(EmptyHintCaller::new());
			//let mut ctx: Context<M31Config, ParallelizedExpanderGKRProvingSystem<M31Config>, HintRegistry<M31>> = Context::new(hint_registry);


			let input_str = fs::read_to_string("input.json").unwrap();
			let input: Circuit_Input = serde_json::from_str(&input_str).unwrap();
			let mut assignment = input_copy(&input);

			// mul operation
			let _features_features_0_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_0_conv_Conv_output_0, false);
			let _features_features_0_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_0_Constant_output_0, true);
			let mut _features_features_0_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_Mul, _features_features_0_conv_Conv_output_0, _features_features_0_Constant_output_0, mut _features_features_0_Mul_output_0);
			// div operation
			let _features_features_0_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_0_Constant_1_output_0, true);
			let _features_features_0_Div_output_0 = ctx.copy_to_device(&assignment._features_features_0_Div_output_0, false);
			let _features_features_0_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_0_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_0_Div, _features_features_0_Mul_output_0, _features_features_0_Constant_1_output_0, _features_features_0_Div_output_0, _features_features_0_Div_output_0_r);
			// cast operation
			let _features_features_0_Div_output_0 = ctx.copy_to_device(&assignment._features_features_0_Div_output_0, false);
			let mut _features_features_0_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_Cast, _features_features_0_Div_output_0, mut _features_features_0_Cast_output_0);
			// cast operation
			let mut _features_features_0_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_Cast_1, _features_features_0_Cast_output_0, mut _features_features_0_Cast_1_output_0);
			// add operation
			let _features_features_0_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_0_Constant_2_output_0, true);
			let mut _features_features_0_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_Add, _features_features_0_Cast_1_output_0, _features_features_0_Constant_2_output_0, mut _features_features_0_Add_output_0);
			// cast operation
			let mut _features_features_2_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_2_relu_Cast, _features_features_0_Add_output_0, mut _features_features_2_relu_Cast_output_0);
			// cast operation
			let _features_features_2_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_2_relu_Relu_output_0, false);
			let _features_features_2_relu_Relu_output_0_copy = _features_features_2_relu_Relu_output_0.clone();
			let mut _features_features_2_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_2_relu_Cast_1, _features_features_2_relu_Relu_output_0, mut _features_features_2_relu_Cast_1_output_0);
			
			// mul operation
			let _features_features_3_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_3_conv_Conv_output_0, false);
			let _features_features_3_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_3_Constant_output_0, true);
			let mut _features_features_3_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_3_Mul, _features_features_3_conv_Conv_output_0, _features_features_3_Constant_output_0, mut _features_features_3_Mul_output_0);
			// div operation
			let _features_features_3_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_3_Constant_1_output_0, true);
			let _features_features_3_Div_output_0 = ctx.copy_to_device(&assignment._features_features_3_Div_output_0, false);
			let _features_features_3_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_3_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_3_Div, _features_features_3_Mul_output_0, _features_features_3_Constant_1_output_0, _features_features_3_Div_output_0, _features_features_3_Div_output_0_r);
			// cast operation
			let _features_features_3_Div_output_0 = ctx.copy_to_device(&assignment._features_features_3_Div_output_0, false);
			let mut _features_features_3_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_3_Cast, _features_features_3_Div_output_0, mut _features_features_3_Cast_output_0);
			// cast operation
			let mut _features_features_3_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_3_Cast_1, _features_features_3_Cast_output_0, mut _features_features_3_Cast_1_output_0);
			// add operation
			let _features_features_3_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_3_Constant_2_output_0, true);
			let mut _features_features_3_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_3_Add, _features_features_3_Cast_1_output_0, _features_features_3_Constant_2_output_0, mut _features_features_3_Add_output_0);
			// cast operation
			let mut _features_features_5_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_5_relu_Cast, _features_features_3_Add_output_0, mut _features_features_5_relu_Cast_output_0);
			// cast operation
			let _features_features_5_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_5_relu_Relu_output_0, false);
			let _features_features_5_relu_Relu_output_0_copy = _features_features_5_relu_Relu_output_0.clone();
			let mut _features_features_5_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_5_relu_Cast_1, _features_features_5_relu_Relu_output_0, mut _features_features_5_relu_Cast_1_output_0);
			// mul operation
			let _features_features_7_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_7_conv_Conv_output_0, false);
			let _features_features_7_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_7_Constant_output_0, true);
			let mut _features_features_7_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_7_Mul, _features_features_7_conv_Conv_output_0, _features_features_7_Constant_output_0, mut _features_features_7_Mul_output_0);
			// div operation
			let _features_features_7_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_7_Constant_1_output_0, true);
			let _features_features_7_Div_output_0 = ctx.copy_to_device(&assignment._features_features_7_Div_output_0, false);
			let _features_features_7_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_7_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_7_Div, _features_features_7_Mul_output_0, _features_features_7_Constant_1_output_0, _features_features_7_Div_output_0, _features_features_7_Div_output_0_r);
			// cast operation
			let _features_features_7_Div_output_0 = ctx.copy_to_device(&assignment._features_features_7_Div_output_0, false);
			let mut _features_features_7_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_7_Cast, _features_features_7_Div_output_0, mut _features_features_7_Cast_output_0);
			// cast operation
			let mut _features_features_7_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_7_Cast_1, _features_features_7_Cast_output_0, mut _features_features_7_Cast_1_output_0);
			// add operation
			let _features_features_7_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_7_Constant_2_output_0, true);
			let mut _features_features_7_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_7_Add, _features_features_7_Cast_1_output_0, _features_features_7_Constant_2_output_0, mut _features_features_7_Add_output_0);
			// cast operation
			let mut _features_features_9_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_9_relu_Cast, _features_features_7_Add_output_0, mut _features_features_9_relu_Cast_output_0);
			// cast operation
			let _features_features_9_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_9_relu_Relu_output_0, false);
			let _features_features_9_relu_Relu_output_0_copy = _features_features_9_relu_Relu_output_0.clone();
			let mut _features_features_9_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_9_relu_Cast_1, _features_features_9_relu_Relu_output_0, mut _features_features_9_relu_Cast_1_output_0);
			// mul operation
			let _features_features_10_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_10_conv_Conv_output_0, false);
			let _features_features_10_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_10_Constant_output_0, true);
			let mut _features_features_10_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_10_Mul, _features_features_10_conv_Conv_output_0, _features_features_10_Constant_output_0, mut _features_features_10_Mul_output_0);
			// div operation
			let _features_features_10_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_10_Constant_1_output_0, true);
			let _features_features_10_Div_output_0 = ctx.copy_to_device(&assignment._features_features_10_Div_output_0, false);
			let _features_features_10_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_10_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_10_Div, _features_features_10_Mul_output_0, _features_features_10_Constant_1_output_0, _features_features_10_Div_output_0, _features_features_10_Div_output_0_r);
			// cast operation
			let _features_features_10_Div_output_0 = ctx.copy_to_device(&assignment._features_features_10_Div_output_0, false);
			let mut _features_features_10_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_10_Cast, _features_features_10_Div_output_0, mut _features_features_10_Cast_output_0);
			// cast operation
			let mut _features_features_10_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_10_Cast_1, _features_features_10_Cast_output_0, mut _features_features_10_Cast_1_output_0);
			// add operation
			let _features_features_10_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_10_Constant_2_output_0, true);
			let mut _features_features_10_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_10_Add, _features_features_10_Cast_1_output_0, _features_features_10_Constant_2_output_0, mut _features_features_10_Add_output_0);
			// cast operation
			let mut _features_features_12_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_12_relu_Cast, _features_features_10_Add_output_0, mut _features_features_12_relu_Cast_output_0);
			// cast operation
			let _features_features_12_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_12_relu_Relu_output_0, false);
			let _features_features_12_relu_Relu_output_0_copy = _features_features_12_relu_Relu_output_0.clone();
			let mut _features_features_12_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_12_relu_Cast_1, _features_features_12_relu_Relu_output_0, mut _features_features_12_relu_Cast_1_output_0);
			// mul operation
			let _features_features_14_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_14_conv_Conv_output_0, false);
			let _features_features_14_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_14_Constant_output_0, true);
			let mut _features_features_14_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_14_Mul, _features_features_14_conv_Conv_output_0, _features_features_14_Constant_output_0, mut _features_features_14_Mul_output_0);
			// div operation
			let _features_features_14_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_14_Constant_1_output_0, true);
			let _features_features_14_Div_output_0 = ctx.copy_to_device(&assignment._features_features_14_Div_output_0, false);
			let _features_features_14_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_14_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_14_Div, _features_features_14_Mul_output_0, _features_features_14_Constant_1_output_0, _features_features_14_Div_output_0, _features_features_14_Div_output_0_r);
			// cast operation
			let _features_features_14_Div_output_0 = ctx.copy_to_device(&assignment._features_features_14_Div_output_0, false);
			let mut _features_features_14_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_14_Cast, _features_features_14_Div_output_0, mut _features_features_14_Cast_output_0);
			// cast operation
			let mut _features_features_14_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_14_Cast_1, _features_features_14_Cast_output_0, mut _features_features_14_Cast_1_output_0);
			// add operation
			let _features_features_14_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_14_Constant_2_output_0, true);
			let mut _features_features_14_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_14_Add, _features_features_14_Cast_1_output_0, _features_features_14_Constant_2_output_0, mut _features_features_14_Add_output_0);
			// cast operation
			let mut _features_features_16_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_16_relu_Cast, _features_features_14_Add_output_0, mut _features_features_16_relu_Cast_output_0);
			// cast operation
			let _features_features_16_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_16_relu_Relu_output_0, false);
			let _features_features_16_relu_Relu_output_0_copy = _features_features_16_relu_Relu_output_0.clone();
			let mut _features_features_16_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_16_relu_Cast_1, _features_features_16_relu_Relu_output_0, mut _features_features_16_relu_Cast_1_output_0);
			// mul operation
			let _features_features_17_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_17_conv_Conv_output_0, false);
			let _features_features_17_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_17_Constant_output_0, true);
			let mut _features_features_17_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_17_Mul, _features_features_17_conv_Conv_output_0, _features_features_17_Constant_output_0, mut _features_features_17_Mul_output_0);
			// div operation
			let _features_features_17_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_17_Constant_1_output_0, true);
			let _features_features_17_Div_output_0 = ctx.copy_to_device(&assignment._features_features_17_Div_output_0, false);
			let _features_features_17_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_17_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_17_Div, _features_features_17_Mul_output_0, _features_features_17_Constant_1_output_0, _features_features_17_Div_output_0, _features_features_17_Div_output_0_r);
			// cast operation
			let _features_features_17_Div_output_0 = ctx.copy_to_device(&assignment._features_features_17_Div_output_0, false);
			let mut _features_features_17_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_17_Cast, _features_features_17_Div_output_0, mut _features_features_17_Cast_output_0);
			// cast operation
			let mut _features_features_17_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_17_Cast_1, _features_features_17_Cast_output_0, mut _features_features_17_Cast_1_output_0);
			// add operation
			let _features_features_17_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_17_Constant_2_output_0, true);
			let mut _features_features_17_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_17_Add, _features_features_17_Cast_1_output_0, _features_features_17_Constant_2_output_0, mut _features_features_17_Add_output_0);
			// cast operation
			let mut _features_features_19_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_19_relu_Cast, _features_features_17_Add_output_0, mut _features_features_19_relu_Cast_output_0);
			// cast operation
			let _features_features_19_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_19_relu_Relu_output_0, false);
			let _features_features_19_relu_Relu_output_0_copy = _features_features_19_relu_Relu_output_0.clone();
			let mut _features_features_19_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_19_relu_Cast_1, _features_features_19_relu_Relu_output_0, mut _features_features_19_relu_Cast_1_output_0);
			// mul operation
			let _features_features_20_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_20_conv_Conv_output_0, false);
			let _features_features_20_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_20_Constant_output_0, true);
			let mut _features_features_20_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_20_Mul, _features_features_20_conv_Conv_output_0, _features_features_20_Constant_output_0, mut _features_features_20_Mul_output_0);
			// div operation
			let _features_features_20_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_20_Constant_1_output_0, true);
			let _features_features_20_Div_output_0 = ctx.copy_to_device(&assignment._features_features_20_Div_output_0, false);
			let _features_features_20_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_20_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_20_Div, _features_features_20_Mul_output_0, _features_features_20_Constant_1_output_0, _features_features_20_Div_output_0, _features_features_20_Div_output_0_r);
			// cast operation
			let _features_features_20_Div_output_0 = ctx.copy_to_device(&assignment._features_features_20_Div_output_0, false);
			let mut _features_features_20_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_20_Cast, _features_features_20_Div_output_0, mut _features_features_20_Cast_output_0);
			// cast operation
			let mut _features_features_20_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_20_Cast_1, _features_features_20_Cast_output_0, mut _features_features_20_Cast_1_output_0);
			// add operation
			let _features_features_20_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_20_Constant_2_output_0, true);
			let mut _features_features_20_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_20_Add, _features_features_20_Cast_1_output_0, _features_features_20_Constant_2_output_0, mut _features_features_20_Add_output_0);
			// cast operation
			let mut _features_features_22_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_22_relu_Cast, _features_features_20_Add_output_0, mut _features_features_22_relu_Cast_output_0);
			// cast operation
			let _features_features_22_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_22_relu_Relu_output_0, false);
			let _features_features_22_relu_Relu_output_0_copy = _features_features_22_relu_Relu_output_0.clone();
			let mut _features_features_22_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_22_relu_Cast_1, _features_features_22_relu_Relu_output_0, mut _features_features_22_relu_Cast_1_output_0);
			// mul operation
			let _features_features_24_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_24_conv_Conv_output_0, false);
			let _features_features_24_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_24_Constant_output_0, true);
			let mut _features_features_24_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_24_Mul, _features_features_24_conv_Conv_output_0, _features_features_24_Constant_output_0, mut _features_features_24_Mul_output_0);
			// div operation
			let _features_features_24_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_24_Constant_1_output_0, true);
			let _features_features_24_Div_output_0 = ctx.copy_to_device(&assignment._features_features_24_Div_output_0, false);
			let _features_features_24_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_24_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_24_Div, _features_features_24_Mul_output_0, _features_features_24_Constant_1_output_0, _features_features_24_Div_output_0, _features_features_24_Div_output_0_r);
			// cast operation
			let _features_features_24_Div_output_0 = ctx.copy_to_device(&assignment._features_features_24_Div_output_0, false);
			let mut _features_features_24_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_24_Cast, _features_features_24_Div_output_0, mut _features_features_24_Cast_output_0);
			// cast operation
			let mut _features_features_24_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_24_Cast_1, _features_features_24_Cast_output_0, mut _features_features_24_Cast_1_output_0);
			// add operation
			let _features_features_24_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_24_Constant_2_output_0, true);
			let mut _features_features_24_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_24_Add, _features_features_24_Cast_1_output_0, _features_features_24_Constant_2_output_0, mut _features_features_24_Add_output_0);
			// cast operation
			let mut _features_features_26_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_26_relu_Cast, _features_features_24_Add_output_0, mut _features_features_26_relu_Cast_output_0);
			// cast operation
			let _features_features_26_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_26_relu_Relu_output_0, false);
			let _features_features_26_relu_Relu_output_0_copy = _features_features_26_relu_Relu_output_0.clone();
			let mut _features_features_26_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_26_relu_Cast_1, _features_features_26_relu_Relu_output_0, mut _features_features_26_relu_Cast_1_output_0);
			// mul operation
			let _features_features_27_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_27_conv_Conv_output_0, false);
			let _features_features_27_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_27_Constant_output_0, true);
			let mut _features_features_27_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_27_Mul, _features_features_27_conv_Conv_output_0, _features_features_27_Constant_output_0, mut _features_features_27_Mul_output_0);
			// div operation
			let _features_features_27_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_27_Constant_1_output_0, true);
			let _features_features_27_Div_output_0 = ctx.copy_to_device(&assignment._features_features_27_Div_output_0, false);
			let _features_features_27_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_27_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_27_Div, _features_features_27_Mul_output_0, _features_features_27_Constant_1_output_0, _features_features_27_Div_output_0, _features_features_27_Div_output_0_r);
			// cast operation
			let _features_features_27_Div_output_0 = ctx.copy_to_device(&assignment._features_features_27_Div_output_0, false);
			let mut _features_features_27_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_27_Cast, _features_features_27_Div_output_0, mut _features_features_27_Cast_output_0);
			// cast operation
			let mut _features_features_27_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_27_Cast_1, _features_features_27_Cast_output_0, mut _features_features_27_Cast_1_output_0);
			// add operation
			let _features_features_27_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_27_Constant_2_output_0, true);
			let mut _features_features_27_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_27_Add, _features_features_27_Cast_1_output_0, _features_features_27_Constant_2_output_0, mut _features_features_27_Add_output_0);
			// cast operation
			let mut _features_features_29_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_29_relu_Cast, _features_features_27_Add_output_0, mut _features_features_29_relu_Cast_output_0);
			// cast operation
			let _features_features_29_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_29_relu_Relu_output_0, false);
			let _features_features_29_relu_Relu_output_0_copy = _features_features_29_relu_Relu_output_0.clone();
			let mut _features_features_29_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_29_relu_Cast_1, _features_features_29_relu_Relu_output_0, mut _features_features_29_relu_Cast_1_output_0);
			// mul operation
			let _features_features_30_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_30_conv_Conv_output_0, false);
			let _features_features_30_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_30_Constant_output_0, true);
			let mut _features_features_30_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_30_Mul, _features_features_30_conv_Conv_output_0, _features_features_30_Constant_output_0, mut _features_features_30_Mul_output_0);
			// div operation
			let _features_features_30_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_30_Constant_1_output_0, true);
			let _features_features_30_Div_output_0 = ctx.copy_to_device(&assignment._features_features_30_Div_output_0, false);
			let _features_features_30_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_30_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_30_Div, _features_features_30_Mul_output_0, _features_features_30_Constant_1_output_0, _features_features_30_Div_output_0, _features_features_30_Div_output_0_r);
			// cast operation
			let _features_features_30_Div_output_0 = ctx.copy_to_device(&assignment._features_features_30_Div_output_0, false);
			let mut _features_features_30_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_30_Cast, _features_features_30_Div_output_0, mut _features_features_30_Cast_output_0);
			// cast operation
			let mut _features_features_30_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_30_Cast_1, _features_features_30_Cast_output_0, mut _features_features_30_Cast_1_output_0);
			// add operation
			let _features_features_30_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_30_Constant_2_output_0, true);
			let mut _features_features_30_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_30_Add, _features_features_30_Cast_1_output_0, _features_features_30_Constant_2_output_0, mut _features_features_30_Add_output_0);
			// cast operation
			let mut _features_features_32_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_32_relu_Cast, _features_features_30_Add_output_0, mut _features_features_32_relu_Cast_output_0);
			// cast operation
			let _features_features_32_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_32_relu_Relu_output_0, false);
			let _features_features_32_relu_Relu_output_0_copy = _features_features_32_relu_Relu_output_0.clone();
			let mut _features_features_32_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_32_relu_Cast_1, _features_features_32_relu_Relu_output_0, mut _features_features_32_relu_Cast_1_output_0);
			// mul operation
			let _features_features_34_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_34_conv_Conv_output_0, false);
			let _features_features_34_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_34_Constant_output_0, true);
			let mut _features_features_34_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_34_Mul, _features_features_34_conv_Conv_output_0, _features_features_34_Constant_output_0, mut _features_features_34_Mul_output_0);
			// div operation
			let _features_features_34_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_34_Constant_1_output_0, true);
			let _features_features_34_Div_output_0 = ctx.copy_to_device(&assignment._features_features_34_Div_output_0, false);
			let _features_features_34_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_34_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_34_Div, _features_features_34_Mul_output_0, _features_features_34_Constant_1_output_0, _features_features_34_Div_output_0, _features_features_34_Div_output_0_r);
			// cast operation
			let _features_features_34_Div_output_0 = ctx.copy_to_device(&assignment._features_features_34_Div_output_0, false);
			let mut _features_features_34_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_34_Cast, _features_features_34_Div_output_0, mut _features_features_34_Cast_output_0);
			// cast operation
			let mut _features_features_34_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_34_Cast_1, _features_features_34_Cast_output_0, mut _features_features_34_Cast_1_output_0);
			// add operation
			let _features_features_34_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_34_Constant_2_output_0, true);
			let mut _features_features_34_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_34_Add, _features_features_34_Cast_1_output_0, _features_features_34_Constant_2_output_0, mut _features_features_34_Add_output_0);
			// cast operation
			let mut _features_features_36_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_36_relu_Cast, _features_features_34_Add_output_0, mut _features_features_36_relu_Cast_output_0);
			// cast operation
			let _features_features_36_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_36_relu_Relu_output_0, false);
			let _features_features_36_relu_Relu_output_0_copy = _features_features_36_relu_Relu_output_0.clone();
			let mut _features_features_36_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_36_relu_Cast_1, _features_features_36_relu_Relu_output_0, mut _features_features_36_relu_Cast_1_output_0);
			// mul operation
			let _features_features_37_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_37_conv_Conv_output_0, false);
			let _features_features_37_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_37_Constant_output_0, true);
			let mut _features_features_37_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_37_Mul, _features_features_37_conv_Conv_output_0, _features_features_37_Constant_output_0, mut _features_features_37_Mul_output_0);
			// div operation
			let _features_features_37_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_37_Constant_1_output_0, true);
			let _features_features_37_Div_output_0 = ctx.copy_to_device(&assignment._features_features_37_Div_output_0, false);
			let _features_features_37_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_37_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_37_Div, _features_features_37_Mul_output_0, _features_features_37_Constant_1_output_0, _features_features_37_Div_output_0, _features_features_37_Div_output_0_r);
			// cast operation
			let _features_features_37_Div_output_0 = ctx.copy_to_device(&assignment._features_features_37_Div_output_0, false);
			let mut _features_features_37_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_37_Cast, _features_features_37_Div_output_0, mut _features_features_37_Cast_output_0);
			// cast operation
			let mut _features_features_37_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_37_Cast_1, _features_features_37_Cast_output_0, mut _features_features_37_Cast_1_output_0);
			// add operation
			let _features_features_37_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_37_Constant_2_output_0, true);
			let mut _features_features_37_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_37_Add, _features_features_37_Cast_1_output_0, _features_features_37_Constant_2_output_0, mut _features_features_37_Add_output_0);
			// cast operation
			let mut _features_features_39_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_39_relu_Cast, _features_features_37_Add_output_0, mut _features_features_39_relu_Cast_output_0);
			// cast operation
			let _features_features_39_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_39_relu_Relu_output_0, false);
			let _features_features_39_relu_Relu_output_0_copy = _features_features_39_relu_Relu_output_0.clone();
			let mut _features_features_39_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_39_relu_Cast_1, _features_features_39_relu_Relu_output_0, mut _features_features_39_relu_Cast_1_output_0);
			// mul operation
			let _features_features_40_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_40_conv_Conv_output_0, false);
			let _features_features_40_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_40_Constant_output_0, true);
			let mut _features_features_40_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_40_Mul, _features_features_40_conv_Conv_output_0, _features_features_40_Constant_output_0, mut _features_features_40_Mul_output_0);
			// div operation
			let _features_features_40_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_40_Constant_1_output_0, true);
			let _features_features_40_Div_output_0 = ctx.copy_to_device(&assignment._features_features_40_Div_output_0, false);
			let _features_features_40_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_40_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_40_Div, _features_features_40_Mul_output_0, _features_features_40_Constant_1_output_0, _features_features_40_Div_output_0, _features_features_40_Div_output_0_r);
			// cast operation
			let _features_features_40_Div_output_0 = ctx.copy_to_device(&assignment._features_features_40_Div_output_0, false);
			let mut _features_features_40_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_40_Cast, _features_features_40_Div_output_0, mut _features_features_40_Cast_output_0);
			// cast operation
			let mut _features_features_40_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_40_Cast_1, _features_features_40_Cast_output_0, mut _features_features_40_Cast_1_output_0);
			// add operation
			let _features_features_40_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_40_Constant_2_output_0, true);
			let mut _features_features_40_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_40_Add, _features_features_40_Cast_1_output_0, _features_features_40_Constant_2_output_0, mut _features_features_40_Add_output_0);
			// cast operation
			let mut _features_features_42_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_42_relu_Cast, _features_features_40_Add_output_0, mut _features_features_42_relu_Cast_output_0);
			// cast operation
			let _features_features_42_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_42_relu_Relu_output_0, false);
			let _features_features_42_relu_Relu_output_0_copy = _features_features_42_relu_Relu_output_0.clone();
			let mut _features_features_42_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_42_relu_Cast_1, _features_features_42_relu_Relu_output_0, mut _features_features_42_relu_Cast_1_output_0);
			// mul operation
			let _classifier_classifier_0_linear_MatMul_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_linear_MatMul_output_0, false);
			let _classifier_classifier_0_Constant_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_Constant_output_0, true);
			let mut _classifier_classifier_0_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_0_Mul, _classifier_classifier_0_linear_MatMul_output_0, _classifier_classifier_0_Constant_output_0, mut _classifier_classifier_0_Mul_output_0);
			// div operation
			let _classifier_classifier_0_Constant_1_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_Constant_1_output_0, true);
			let _classifier_classifier_0_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_Div_output_0, false);
			let _classifier_classifier_0_Div_output_0_r = ctx.copy_to_device(&assignment._classifier_classifier_0_Div_output_0_r, false);
			call_kernel!(ctx, kernel__classifier_classifier_0_Div, _classifier_classifier_0_Mul_output_0, _classifier_classifier_0_Constant_1_output_0, _classifier_classifier_0_Div_output_0, _classifier_classifier_0_Div_output_0_r);
			// cast operation
			let _classifier_classifier_0_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_Div_output_0, false);
			let mut _classifier_classifier_0_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_0_Cast, _classifier_classifier_0_Div_output_0, mut _classifier_classifier_0_Cast_output_0);
			// cast operation
			let mut _classifier_classifier_0_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_0_Cast_1, _classifier_classifier_0_Cast_output_0, mut _classifier_classifier_0_Cast_1_output_0);
			// add operation
			let _classifier_classifier_0_Constant_2_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_Constant_2_output_0, true);
			let mut _classifier_classifier_0_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_0_Add, _classifier_classifier_0_Cast_1_output_0, _classifier_classifier_0_Constant_2_output_0, mut _classifier_classifier_0_Add_output_0);
			// cast operation
			let mut _classifier_classifier_1_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_1_relu_Cast, _classifier_classifier_0_Add_output_0, mut _classifier_classifier_1_relu_Cast_output_0);
			// cast operation
			let _classifier_classifier_1_relu_Relu_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_1_relu_Relu_output_0, false);
			let _classifier_classifier_1_relu_Relu_output_0_copy = _classifier_classifier_1_relu_Relu_output_0.clone();
			let mut _classifier_classifier_1_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_1_relu_Cast_1, _classifier_classifier_1_relu_Relu_output_0, mut _classifier_classifier_1_relu_Cast_1_output_0);
			// mul operation
			let _classifier_classifier_3_linear_MatMul_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_linear_MatMul_output_0, false);
			let _classifier_classifier_3_Constant_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_Constant_output_0, true);
			let mut _classifier_classifier_3_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_3_Mul, _classifier_classifier_3_linear_MatMul_output_0, _classifier_classifier_3_Constant_output_0, mut _classifier_classifier_3_Mul_output_0);
			// div operation
			let _classifier_classifier_3_Constant_1_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_Constant_1_output_0, true);
			let _classifier_classifier_3_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_Div_output_0, false);
			let _classifier_classifier_3_Div_output_0_r = ctx.copy_to_device(&assignment._classifier_classifier_3_Div_output_0_r, false);
			call_kernel!(ctx, kernel__classifier_classifier_3_Div, _classifier_classifier_3_Mul_output_0, _classifier_classifier_3_Constant_1_output_0, _classifier_classifier_3_Div_output_0, _classifier_classifier_3_Div_output_0_r);
			// cast operation
			let _classifier_classifier_3_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_Div_output_0, false);
			let mut _classifier_classifier_3_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_3_Cast, _classifier_classifier_3_Div_output_0, mut _classifier_classifier_3_Cast_output_0);
			// cast operation
			let mut _classifier_classifier_3_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_3_Cast_1, _classifier_classifier_3_Cast_output_0, mut _classifier_classifier_3_Cast_1_output_0);
			// add operation
			let _classifier_classifier_3_Constant_2_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_Constant_2_output_0, true);
			let mut _classifier_classifier_3_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_3_Add, _classifier_classifier_3_Cast_1_output_0, _classifier_classifier_3_Constant_2_output_0, mut _classifier_classifier_3_Add_output_0);
			// cast operation
			let mut _classifier_classifier_4_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_4_relu_Cast, _classifier_classifier_3_Add_output_0, mut _classifier_classifier_4_relu_Cast_output_0);
			// cast operation
			let _classifier_classifier_4_relu_Relu_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_4_relu_Relu_output_0, false);
			let _classifier_classifier_4_relu_Relu_output_0_copy = _classifier_classifier_4_relu_Relu_output_0.clone();
			let mut _classifier_classifier_4_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_4_relu_Cast_1, _classifier_classifier_4_relu_Relu_output_0, mut _classifier_classifier_4_relu_Cast_1_output_0);
			// mul operation
			let _classifier_classifier_6_linear_MatMul_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_linear_MatMul_output_0, false);
			let _classifier_classifier_6_Constant_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_Constant_output_0, true);
			let mut _classifier_classifier_6_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_6_Mul, _classifier_classifier_6_linear_MatMul_output_0, _classifier_classifier_6_Constant_output_0, mut _classifier_classifier_6_Mul_output_0);
			// div operation
			let _classifier_classifier_6_Constant_1_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_Constant_1_output_0, true);
			let _classifier_classifier_6_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_Div_output_0, false);
			let _classifier_classifier_6_Div_output_0_r = ctx.copy_to_device(&assignment._classifier_classifier_6_Div_output_0_r, false);
			call_kernel!(ctx, kernel__classifier_classifier_6_Div, _classifier_classifier_6_Mul_output_0, _classifier_classifier_6_Constant_1_output_0, _classifier_classifier_6_Div_output_0, _classifier_classifier_6_Div_output_0_r);
			// cast operation
			let _classifier_classifier_6_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_Div_output_0, false);
			let mut _classifier_classifier_6_Cast_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_6_Cast, _classifier_classifier_6_Div_output_0, mut _classifier_classifier_6_Cast_output_0);
			// cast operation
			let mut _classifier_classifier_6_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_6_Cast_1, _classifier_classifier_6_Cast_output_0, mut _classifier_classifier_6_Cast_1_output_0);
			// add operation
			let _classifier_classifier_6_Constant_2_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_Constant_2_output_0, true);
			
			let mut output: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_6_Add, _classifier_classifier_6_Cast_1_output_0, _classifier_classifier_6_Constant_2_output_0, mut output);
				
			// relu rangeproof cast
			call_kernel!(ctx, kernel__features_features_2_relu_rangeproof_cast0, 
				_features_features_2_relu_Relu_output_0_copy, 
				_features_features_2_relu_Cast_output_0,
				_features_features_5_relu_Relu_output_0_copy,
				_features_features_5_relu_Cast_output_0,
				_features_features_9_relu_Relu_output_0_copy,
				_features_features_9_relu_Cast_output_0 ,
				_features_features_12_relu_Relu_output_0_copy,
				_features_features_12_relu_Cast_output_0,
				_features_features_16_relu_Relu_output_0_copy,
				_features_features_16_relu_Cast_output_0,
				_features_features_19_relu_Relu_output_0_copy,
				_features_features_19_relu_Cast_output_0,
				_features_features_22_relu_Relu_output_0_copy,
				_features_features_22_relu_Cast_output_0,
				_features_features_26_relu_Relu_output_0_copy,
				_features_features_26_relu_Cast_output_0,
				_features_features_29_relu_Relu_output_0_copy,
				_features_features_29_relu_Cast_output_0,
				_features_features_32_relu_Relu_output_0_copy,
				_features_features_32_relu_Cast_output_0,
				_features_features_36_relu_Relu_output_0_copy,
				_features_features_36_relu_Cast_output_0,
				_features_features_39_relu_Relu_output_0_copy,
				_features_features_39_relu_Cast_output_0,
				_features_features_42_relu_Relu_output_0_copy,
				_features_features_42_relu_Cast_output_0,
				_classifier_classifier_1_relu_Relu_output_0_copy,
				_classifier_classifier_1_relu_Cast_output_0,
				_classifier_classifier_4_relu_Relu_output_0_copy,
				_classifier_classifier_4_relu_Cast_output_0,
	);
			


            let computation_graph = ctx.to_computation_graph();
            let (prover_setup, verifier_setup) = ctx.proving_system_setup(&computation_graph);
            let proof = ctx.to_proof(&prover_setup);
            assert!(computation_graph.verify(&proof, &verifier_setup));

            // let start = Instant::now();
			// let proof = ctx.to_proof();
			// let duration = start.elapsed();
			// println!("Time taken: {:?}", duration);
			// assert!(proof.verify());
		}
	);
	Ok(())
}
