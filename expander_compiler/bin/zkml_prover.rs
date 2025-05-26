use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proof::ComputationGraph;
use expander_compiler::zkcuda::proving_system::{ExpanderGKRProvingSystem, ExpanderGKRProverSetup, ExpanderGKRVerifierSetup, ParallelizedExpanderGKRProvingSystem, ProvingSystem,};
use expander_compiler::zkcuda::{context::*, kernel::*};
use gkr::BN254ConfigSha2Hyrax;
use gkr_engine::{FieldEngine, GKREngine};
use serdes::ExpSerde;
use poly_commit::HyraxPCS;
use serde::{Deserialize, Serialize};
use std::fs;
use warp::http::StatusCode;
use warp::test::request;
use warp::Filter;
use warp::Rejection;
use warp::Reply;
use once_cell::sync::Lazy;
use std::convert::Infallible;
use warp::multipart::FormData;
use warp::multipart::Part;
use futures::TryStreamExt;
use bytes::Buf;
use futures::StreamExt;
use halo2curves::bn256::G1Affine;
use std::sync::Arc;
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




static KERNELS: Lazy<Vec<Kernel<BN254Config>>> = Lazy::new(|| {
    println!("Loading Kernel...");
	let start_time = std::time::Instant::now();
    let file = std::fs::File::open("circuit.txt").unwrap();
	let reader = std::io::BufReader::new(file);
	let res = Vec::<Kernel<BN254Config>>::deserialize_from(reader).unwrap();
	println!("Kernel loaded in {:?}", start_time.elapsed());
	res
});
static GRAPH: Lazy<ComputationGraph<BN254Config>> = Lazy::new(|| {
	println!("Loading Graph...");
	let start_time = std::time::Instant::now();
	let file = std::fs::File::open("graph.txt").unwrap();
	let reader = std::io::BufReader::new(file);
	let res = ComputationGraph::<BN254Config>::deserialize_from(reader).unwrap();
	println!("Graph loaded in {:?}", start_time.elapsed());
	res
});

static SETUPS: Lazy<(Arc<ExpanderGKRProverSetup<<BN254Config as GKREngine>::FieldConfig, HyraxPCS<G1Affine>>>, Arc<ExpanderGKRVerifierSetup<<BN254Config as GKREngine>::FieldConfig, HyraxPCS<G1Affine>>>)> = Lazy::new(|| {
    println!("Running proving_system_setup...");
    let mut ctx: Context<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>> =
        Context::default();
    let (prover_setup, verifier_setup) = ctx.proving_system_setup(&*GRAPH);
	(Arc::new(prover_setup), Arc::new(verifier_setup))
});
pub fn run_prover(assignment: Circuit, kernels: &[Kernel<BN254Config>]){
	stacker::grow(32 * 1024 * 1024 * 1024, ||
		{
			let mut ctx: Context<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>> = Context::default();
			let kernel__features_features_0_Mul = &kernels[0];
			let _features_features_0_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_0_conv_Conv_output_0, false);
			let _features_features_0_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_0_Constant_output_0, true);
			let mut _features_features_0_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_Mul, _features_features_0_conv_Conv_output_0, _features_features_0_Constant_output_0, mut _features_features_0_Mul_output_0);
			// div operation
			let kernel__features_features_0_Div = &kernels[1];
			let _features_features_0_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_0_Constant_1_output_0, true);
			let _features_features_0_Div_output_0 = ctx.copy_to_device(&assignment._features_features_0_Div_output_0, false);
			let _features_features_0_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_0_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_0_Div, _features_features_0_Mul_output_0, _features_features_0_Constant_1_output_0, _features_features_0_Div_output_0, _features_features_0_Div_output_0_r);
			// cast operation
			let _features_features_0_Div_output_0 = ctx.copy_to_device(&assignment._features_features_0_Div_output_0, false);
			let mut _features_features_0_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_0_Div_output_0;
			// cast operation
			let mut _features_features_0_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_0_Cast_output_0;
			// add operation
			let kernel__features_features_0_Add = &kernels[2];
			let _features_features_0_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_0_Constant_2_output_0, true);
			let mut _features_features_0_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_0_Add, _features_features_0_Cast_1_output_0, _features_features_0_Constant_2_output_0, mut _features_features_0_Add_output_0);
			// cast operation
			let mut _features_features_2_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_0_Add_output_0;
			// relu operation
			let kernel__features_features_2_relu_Relu = &kernels[3];
			let _features_features_2_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_2_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_2_relu_Relu, _features_features_2_relu_Cast_output_0, _features_features_2_relu_Relu_output_0);
			// cast operation
			let _features_features_2_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_2_relu_Relu_output_0, false);
			let mut _features_features_2_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_2_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_3_Mul = &kernels[4];
			let _features_features_3_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_3_conv_Conv_output_0, false);
			let _features_features_3_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_3_Constant_output_0, true);
			let mut _features_features_3_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_3_Mul, _features_features_3_conv_Conv_output_0, _features_features_3_Constant_output_0, mut _features_features_3_Mul_output_0);
			// div operation
			let kernel__features_features_3_Div = &kernels[5];
			let _features_features_3_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_3_Constant_1_output_0, true);
			let _features_features_3_Div_output_0 = ctx.copy_to_device(&assignment._features_features_3_Div_output_0, false);
			let _features_features_3_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_3_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_3_Div, _features_features_3_Mul_output_0, _features_features_3_Constant_1_output_0, _features_features_3_Div_output_0, _features_features_3_Div_output_0_r);
			// cast operation
			let _features_features_3_Div_output_0 = ctx.copy_to_device(&assignment._features_features_3_Div_output_0, false);
			let mut _features_features_3_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_3_Div_output_0;
			// cast operation
			let mut _features_features_3_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_3_Cast_output_0;
			// add operation
			let kernel__features_features_3_Add = &kernels[6];
			let _features_features_3_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_3_Constant_2_output_0, true);
			let mut _features_features_3_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_3_Add, _features_features_3_Cast_1_output_0, _features_features_3_Constant_2_output_0, mut _features_features_3_Add_output_0);
			// cast operation
			let mut _features_features_5_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_3_Add_output_0;
			// relu operation
			let kernel__features_features_5_relu_Relu = &kernels[7];
			let _features_features_5_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_5_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_5_relu_Relu, _features_features_5_relu_Cast_output_0, _features_features_5_relu_Relu_output_0);
			// cast operation
			let _features_features_5_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_5_relu_Relu_output_0, false);
			let mut _features_features_5_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_5_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_7_Mul = &kernels[8];
			let _features_features_7_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_7_conv_Conv_output_0, false);
			let _features_features_7_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_7_Constant_output_0, true);
			let mut _features_features_7_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_7_Mul, _features_features_7_conv_Conv_output_0, _features_features_7_Constant_output_0, mut _features_features_7_Mul_output_0);
			// div operation
			let kernel__features_features_7_Div = &kernels[9];
			let _features_features_7_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_7_Constant_1_output_0, true);
			let _features_features_7_Div_output_0 = ctx.copy_to_device(&assignment._features_features_7_Div_output_0, false);
			let _features_features_7_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_7_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_7_Div, _features_features_7_Mul_output_0, _features_features_7_Constant_1_output_0, _features_features_7_Div_output_0, _features_features_7_Div_output_0_r);
			// cast operation
			let _features_features_7_Div_output_0 = ctx.copy_to_device(&assignment._features_features_7_Div_output_0, false);
			let mut _features_features_7_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_7_Div_output_0;
			// cast operation
			let mut _features_features_7_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_7_Cast_output_0;
			// add operation
			let kernel__features_features_7_Add = &kernels[10];
			let _features_features_7_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_7_Constant_2_output_0, true);
			let mut _features_features_7_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_7_Add, _features_features_7_Cast_1_output_0, _features_features_7_Constant_2_output_0, mut _features_features_7_Add_output_0);
			// cast operation
			let mut _features_features_9_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_7_Add_output_0;
			// relu operation
			let kernel__features_features_9_relu_Relu = &kernels[11];
			let _features_features_9_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_9_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_9_relu_Relu, _features_features_9_relu_Cast_output_0, _features_features_9_relu_Relu_output_0);
			// cast operation
			let _features_features_9_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_9_relu_Relu_output_0, false);
			let mut _features_features_9_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_9_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_10_Mul = &kernels[12];
			let _features_features_10_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_10_conv_Conv_output_0, false);
			let _features_features_10_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_10_Constant_output_0, true);
			let mut _features_features_10_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_10_Mul, _features_features_10_conv_Conv_output_0, _features_features_10_Constant_output_0, mut _features_features_10_Mul_output_0);
			// div operation
			let kernel__features_features_10_Div = &kernels[13];
			let _features_features_10_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_10_Constant_1_output_0, true);
			let _features_features_10_Div_output_0 = ctx.copy_to_device(&assignment._features_features_10_Div_output_0, false);
			let _features_features_10_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_10_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_10_Div, _features_features_10_Mul_output_0, _features_features_10_Constant_1_output_0, _features_features_10_Div_output_0, _features_features_10_Div_output_0_r);
			// cast operation
			let _features_features_10_Div_output_0 = ctx.copy_to_device(&assignment._features_features_10_Div_output_0, false);
			let mut _features_features_10_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_10_Div_output_0;
			// cast operation
			let mut _features_features_10_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_10_Cast_output_0;
			// add operation
			let kernel__features_features_10_Add = &kernels[14];
			let _features_features_10_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_10_Constant_2_output_0, true);
			let mut _features_features_10_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_10_Add, _features_features_10_Cast_1_output_0, _features_features_10_Constant_2_output_0, mut _features_features_10_Add_output_0);
			// cast operation
			let mut _features_features_12_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_10_Add_output_0;
			// relu operation
			let kernel__features_features_12_relu_Relu = &kernels[15];
			let _features_features_12_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_12_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_12_relu_Relu, _features_features_12_relu_Cast_output_0, _features_features_12_relu_Relu_output_0);
			// cast operation
			let _features_features_12_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_12_relu_Relu_output_0, false);
			let mut _features_features_12_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_12_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_14_Mul = &kernels[16];
			let _features_features_14_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_14_conv_Conv_output_0, false);
			let _features_features_14_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_14_Constant_output_0, true);
			let mut _features_features_14_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_14_Mul, _features_features_14_conv_Conv_output_0, _features_features_14_Constant_output_0, mut _features_features_14_Mul_output_0);
			// div operation
			let kernel__features_features_14_Div = &kernels[17];
			let _features_features_14_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_14_Constant_1_output_0, true);
			let _features_features_14_Div_output_0 = ctx.copy_to_device(&assignment._features_features_14_Div_output_0, false);
			let _features_features_14_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_14_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_14_Div, _features_features_14_Mul_output_0, _features_features_14_Constant_1_output_0, _features_features_14_Div_output_0, _features_features_14_Div_output_0_r);
			// cast operation
			let _features_features_14_Div_output_0 = ctx.copy_to_device(&assignment._features_features_14_Div_output_0, false);
			let mut _features_features_14_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_14_Div_output_0;
			// cast operation
			let mut _features_features_14_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_14_Cast_output_0;
			// add operation
			let kernel__features_features_14_Add = &kernels[18];
			let _features_features_14_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_14_Constant_2_output_0, true);
			let mut _features_features_14_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_14_Add, _features_features_14_Cast_1_output_0, _features_features_14_Constant_2_output_0, mut _features_features_14_Add_output_0);
			// cast operation
			let mut _features_features_16_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_14_Add_output_0;
			// relu operation
			let kernel__features_features_16_relu_Relu = &kernels[19];
			let _features_features_16_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_16_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_16_relu_Relu, _features_features_16_relu_Cast_output_0, _features_features_16_relu_Relu_output_0);
			// cast operation
			let _features_features_16_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_16_relu_Relu_output_0, false);
			let mut _features_features_16_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_16_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_17_Mul = &kernels[20];
			let _features_features_17_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_17_conv_Conv_output_0, false);
			let _features_features_17_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_17_Constant_output_0, true);
			let mut _features_features_17_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_17_Mul, _features_features_17_conv_Conv_output_0, _features_features_17_Constant_output_0, mut _features_features_17_Mul_output_0);
			// div operation
			let kernel__features_features_17_Div = &kernels[21];
			let _features_features_17_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_17_Constant_1_output_0, true);
			let _features_features_17_Div_output_0 = ctx.copy_to_device(&assignment._features_features_17_Div_output_0, false);
			let _features_features_17_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_17_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_17_Div, _features_features_17_Mul_output_0, _features_features_17_Constant_1_output_0, _features_features_17_Div_output_0, _features_features_17_Div_output_0_r);
			// cast operation
			let _features_features_17_Div_output_0 = ctx.copy_to_device(&assignment._features_features_17_Div_output_0, false);
			let mut _features_features_17_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_17_Div_output_0;
			// cast operation
			let mut _features_features_17_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_17_Cast_output_0;
			// add operation
			let kernel__features_features_17_Add = &kernels[22];
			let _features_features_17_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_17_Constant_2_output_0, true);
			let mut _features_features_17_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_17_Add, _features_features_17_Cast_1_output_0, _features_features_17_Constant_2_output_0, mut _features_features_17_Add_output_0);
			// cast operation
			let mut _features_features_19_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_17_Add_output_0;
			// relu operation
			let kernel__features_features_19_relu_Relu = &kernels[23];
			let _features_features_19_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_19_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_19_relu_Relu, _features_features_19_relu_Cast_output_0, _features_features_19_relu_Relu_output_0);
			// cast operation
			let _features_features_19_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_19_relu_Relu_output_0, false);
			let mut _features_features_19_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_19_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_20_Mul = &kernels[24];
			let _features_features_20_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_20_conv_Conv_output_0, false);
			let _features_features_20_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_20_Constant_output_0, true);
			let mut _features_features_20_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_20_Mul, _features_features_20_conv_Conv_output_0, _features_features_20_Constant_output_0, mut _features_features_20_Mul_output_0);
			// div operation
			let kernel__features_features_20_Div = &kernels[25];
			let _features_features_20_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_20_Constant_1_output_0, true);
			let _features_features_20_Div_output_0 = ctx.copy_to_device(&assignment._features_features_20_Div_output_0, false);
			let _features_features_20_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_20_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_20_Div, _features_features_20_Mul_output_0, _features_features_20_Constant_1_output_0, _features_features_20_Div_output_0, _features_features_20_Div_output_0_r);
			// cast operation
			let _features_features_20_Div_output_0 = ctx.copy_to_device(&assignment._features_features_20_Div_output_0, false);
			let mut _features_features_20_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_20_Div_output_0;
			// cast operation
			let mut _features_features_20_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_20_Cast_output_0;
			// add operation
			let kernel__features_features_20_Add = &kernels[26];
			let _features_features_20_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_20_Constant_2_output_0, true);
			let mut _features_features_20_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_20_Add, _features_features_20_Cast_1_output_0, _features_features_20_Constant_2_output_0, mut _features_features_20_Add_output_0);
			// cast operation
			let mut _features_features_22_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_20_Add_output_0;
			// relu operation
			let kernel__features_features_22_relu_Relu = &kernels[27];
			let _features_features_22_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_22_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_22_relu_Relu, _features_features_22_relu_Cast_output_0, _features_features_22_relu_Relu_output_0);
			// cast operation
			let _features_features_22_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_22_relu_Relu_output_0, false);
			let mut _features_features_22_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_22_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_24_Mul = &kernels[28];
			let _features_features_24_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_24_conv_Conv_output_0, false);
			let _features_features_24_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_24_Constant_output_0, true);
			let mut _features_features_24_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_24_Mul, _features_features_24_conv_Conv_output_0, _features_features_24_Constant_output_0, mut _features_features_24_Mul_output_0);
			// div operation
			let kernel__features_features_24_Div = &kernels[29];
			let _features_features_24_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_24_Constant_1_output_0, true);
			let _features_features_24_Div_output_0 = ctx.copy_to_device(&assignment._features_features_24_Div_output_0, false);
			let _features_features_24_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_24_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_24_Div, _features_features_24_Mul_output_0, _features_features_24_Constant_1_output_0, _features_features_24_Div_output_0, _features_features_24_Div_output_0_r);
			// cast operation
			let _features_features_24_Div_output_0 = ctx.copy_to_device(&assignment._features_features_24_Div_output_0, false);
			let mut _features_features_24_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_24_Div_output_0;
			// cast operation
			let mut _features_features_24_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_24_Cast_output_0;
			// add operation
			let kernel__features_features_24_Add = &kernels[30];
			let _features_features_24_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_24_Constant_2_output_0, true);
			let mut _features_features_24_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_24_Add, _features_features_24_Cast_1_output_0, _features_features_24_Constant_2_output_0, mut _features_features_24_Add_output_0);
			// cast operation
			let mut _features_features_26_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_24_Add_output_0;
			// relu operation
			let kernel__features_features_26_relu_Relu = &kernels[31];
			let _features_features_26_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_26_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_26_relu_Relu, _features_features_26_relu_Cast_output_0, _features_features_26_relu_Relu_output_0);
			// cast operation
			let _features_features_26_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_26_relu_Relu_output_0, false);
			let mut _features_features_26_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_26_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_27_Mul = &kernels[32];
			let _features_features_27_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_27_conv_Conv_output_0, false);
			let _features_features_27_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_27_Constant_output_0, true);
			let mut _features_features_27_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_27_Mul, _features_features_27_conv_Conv_output_0, _features_features_27_Constant_output_0, mut _features_features_27_Mul_output_0);
			// div operation
			let kernel__features_features_27_Div = &kernels[33];
			let _features_features_27_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_27_Constant_1_output_0, true);
			let _features_features_27_Div_output_0 = ctx.copy_to_device(&assignment._features_features_27_Div_output_0, false);
			let _features_features_27_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_27_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_27_Div, _features_features_27_Mul_output_0, _features_features_27_Constant_1_output_0, _features_features_27_Div_output_0, _features_features_27_Div_output_0_r);
			// cast operation
			let _features_features_27_Div_output_0 = ctx.copy_to_device(&assignment._features_features_27_Div_output_0, false);
			let mut _features_features_27_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_27_Div_output_0;
			// cast operation
			let mut _features_features_27_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_27_Cast_output_0;
			// add operation
			let kernel__features_features_27_Add = &kernels[34];
			let _features_features_27_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_27_Constant_2_output_0, true);
			let mut _features_features_27_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_27_Add, _features_features_27_Cast_1_output_0, _features_features_27_Constant_2_output_0, mut _features_features_27_Add_output_0);
			// cast operation
			let mut _features_features_29_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_27_Add_output_0;
			// relu operation
			let kernel__features_features_29_relu_Relu = &kernels[35];
			let _features_features_29_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_29_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_29_relu_Relu, _features_features_29_relu_Cast_output_0, _features_features_29_relu_Relu_output_0);
			// cast operation
			let _features_features_29_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_29_relu_Relu_output_0, false);
			let mut _features_features_29_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_29_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_30_Mul = &kernels[36];
			let _features_features_30_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_30_conv_Conv_output_0, false);
			let _features_features_30_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_30_Constant_output_0, true);
			let mut _features_features_30_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_30_Mul, _features_features_30_conv_Conv_output_0, _features_features_30_Constant_output_0, mut _features_features_30_Mul_output_0);
			// div operation
			let kernel__features_features_30_Div = &kernels[37];
			let _features_features_30_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_30_Constant_1_output_0, true);
			let _features_features_30_Div_output_0 = ctx.copy_to_device(&assignment._features_features_30_Div_output_0, false);
			let _features_features_30_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_30_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_30_Div, _features_features_30_Mul_output_0, _features_features_30_Constant_1_output_0, _features_features_30_Div_output_0, _features_features_30_Div_output_0_r);
			// cast operation
			let _features_features_30_Div_output_0 = ctx.copy_to_device(&assignment._features_features_30_Div_output_0, false);
			let mut _features_features_30_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_30_Div_output_0;
			// cast operation
			let mut _features_features_30_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_30_Cast_output_0;
			// add operation
			let kernel__features_features_30_Add = &kernels[38];
			let _features_features_30_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_30_Constant_2_output_0, true);
			let mut _features_features_30_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_30_Add, _features_features_30_Cast_1_output_0, _features_features_30_Constant_2_output_0, mut _features_features_30_Add_output_0);
			// cast operation
			let mut _features_features_32_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_30_Add_output_0;
			// relu operation
			let kernel__features_features_32_relu_Relu = &kernels[39];
			let _features_features_32_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_32_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_32_relu_Relu, _features_features_32_relu_Cast_output_0, _features_features_32_relu_Relu_output_0);
			// cast operation
			let _features_features_32_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_32_relu_Relu_output_0, false);
			let mut _features_features_32_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_32_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_34_Mul = &kernels[40];
			let _features_features_34_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_34_conv_Conv_output_0, false);
			let _features_features_34_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_34_Constant_output_0, true);
			let mut _features_features_34_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_34_Mul, _features_features_34_conv_Conv_output_0, _features_features_34_Constant_output_0, mut _features_features_34_Mul_output_0);
			// div operation
			let kernel__features_features_34_Div = &kernels[41];
			let _features_features_34_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_34_Constant_1_output_0, true);
			let _features_features_34_Div_output_0 = ctx.copy_to_device(&assignment._features_features_34_Div_output_0, false);
			let _features_features_34_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_34_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_34_Div, _features_features_34_Mul_output_0, _features_features_34_Constant_1_output_0, _features_features_34_Div_output_0, _features_features_34_Div_output_0_r);
			// cast operation
			let _features_features_34_Div_output_0 = ctx.copy_to_device(&assignment._features_features_34_Div_output_0, false);
			let mut _features_features_34_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_34_Div_output_0;
			// cast operation
			let mut _features_features_34_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_34_Cast_output_0;
			// add operation
			let kernel__features_features_34_Add = &kernels[42];
			let _features_features_34_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_34_Constant_2_output_0, true);
			let mut _features_features_34_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_34_Add, _features_features_34_Cast_1_output_0, _features_features_34_Constant_2_output_0, mut _features_features_34_Add_output_0);
			// cast operation
			let mut _features_features_36_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_34_Add_output_0;
			// relu operation
			let kernel__features_features_36_relu_Relu = &kernels[43];
			let _features_features_36_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_36_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_36_relu_Relu, _features_features_36_relu_Cast_output_0, _features_features_36_relu_Relu_output_0);
			// cast operation
			let _features_features_36_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_36_relu_Relu_output_0, false);
			let mut _features_features_36_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_36_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_37_Mul = &kernels[44];
			let _features_features_37_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_37_conv_Conv_output_0, false);
			let _features_features_37_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_37_Constant_output_0, true);
			let mut _features_features_37_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_37_Mul, _features_features_37_conv_Conv_output_0, _features_features_37_Constant_output_0, mut _features_features_37_Mul_output_0);
			// div operation
			let kernel__features_features_37_Div = &kernels[45];
			let _features_features_37_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_37_Constant_1_output_0, true);
			let _features_features_37_Div_output_0 = ctx.copy_to_device(&assignment._features_features_37_Div_output_0, false);
			let _features_features_37_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_37_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_37_Div, _features_features_37_Mul_output_0, _features_features_37_Constant_1_output_0, _features_features_37_Div_output_0, _features_features_37_Div_output_0_r);
			// cast operation
			let _features_features_37_Div_output_0 = ctx.copy_to_device(&assignment._features_features_37_Div_output_0, false);
			let mut _features_features_37_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_37_Div_output_0;
			// cast operation
			let mut _features_features_37_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_37_Cast_output_0;
			// add operation
			let kernel__features_features_37_Add = &kernels[46];
			let _features_features_37_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_37_Constant_2_output_0, true);
			let mut _features_features_37_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_37_Add, _features_features_37_Cast_1_output_0, _features_features_37_Constant_2_output_0, mut _features_features_37_Add_output_0);
			// cast operation
			let mut _features_features_39_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_37_Add_output_0;
			// relu operation
			let kernel__features_features_39_relu_Relu = &kernels[47];
			let _features_features_39_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_39_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_39_relu_Relu, _features_features_39_relu_Cast_output_0, _features_features_39_relu_Relu_output_0);
			// cast operation
			let _features_features_39_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_39_relu_Relu_output_0, false);
			let mut _features_features_39_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_39_relu_Relu_output_0;
			// mul operation
			let kernel__features_features_40_Mul = &kernels[48];
			let _features_features_40_conv_Conv_output_0 = ctx.copy_to_device(&assignment._features_features_40_conv_Conv_output_0, false);
			let _features_features_40_Constant_output_0 = ctx.copy_to_device(&assignment._features_features_40_Constant_output_0, true);
			let mut _features_features_40_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_40_Mul, _features_features_40_conv_Conv_output_0, _features_features_40_Constant_output_0, mut _features_features_40_Mul_output_0);
			// div operation
			let kernel__features_features_40_Div = &kernels[49];
			let _features_features_40_Constant_1_output_0 = ctx.copy_to_device(&assignment._features_features_40_Constant_1_output_0, true);
			let _features_features_40_Div_output_0 = ctx.copy_to_device(&assignment._features_features_40_Div_output_0, false);
			let _features_features_40_Div_output_0_r = ctx.copy_to_device(&assignment._features_features_40_Div_output_0_r, false);
			call_kernel!(ctx, kernel__features_features_40_Div, _features_features_40_Mul_output_0, _features_features_40_Constant_1_output_0, _features_features_40_Div_output_0, _features_features_40_Div_output_0_r);
			// cast operation
			let _features_features_40_Div_output_0 = ctx.copy_to_device(&assignment._features_features_40_Div_output_0, false);
			let mut _features_features_40_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_40_Div_output_0;
			// cast operation
			let mut _features_features_40_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_40_Cast_output_0;
			// add operation
			let kernel__features_features_40_Add = &kernels[50];
			let _features_features_40_Constant_2_output_0 = ctx.copy_to_device(&assignment._features_features_40_Constant_2_output_0, true);
			let mut _features_features_40_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__features_features_40_Add, _features_features_40_Cast_1_output_0, _features_features_40_Constant_2_output_0, mut _features_features_40_Add_output_0);
			// cast operation
			let mut _features_features_42_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _features_features_40_Add_output_0;
			// relu operation
			let kernel__features_features_42_relu_Relu = &kernels[51];
			let _features_features_42_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_42_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__features_features_42_relu_Relu, _features_features_42_relu_Cast_output_0, _features_features_42_relu_Relu_output_0);
			// cast operation
			let _features_features_42_relu_Relu_output_0 = ctx.copy_to_device(&assignment._features_features_42_relu_Relu_output_0, false);
			let mut _features_features_42_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _features_features_42_relu_Relu_output_0;
			// mul operation
			let kernel__classifier_classifier_0_Mul = &kernels[52];
			let _classifier_classifier_0_linear_MatMul_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_linear_MatMul_output_0, false);
			let _classifier_classifier_0_Constant_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_Constant_output_0, true);
			let mut _classifier_classifier_0_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_0_Mul, _classifier_classifier_0_linear_MatMul_output_0, _classifier_classifier_0_Constant_output_0, mut _classifier_classifier_0_Mul_output_0);
			// div operation
			let kernel__classifier_classifier_0_Div = &kernels[53];
			let _classifier_classifier_0_Constant_1_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_Constant_1_output_0, true);
			let _classifier_classifier_0_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_Div_output_0, false);
			let _classifier_classifier_0_Div_output_0_r = ctx.copy_to_device(&assignment._classifier_classifier_0_Div_output_0_r, false);
			call_kernel!(ctx, kernel__classifier_classifier_0_Div, _classifier_classifier_0_Mul_output_0, _classifier_classifier_0_Constant_1_output_0, _classifier_classifier_0_Div_output_0, _classifier_classifier_0_Div_output_0_r);
			// cast operation
			let _classifier_classifier_0_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_Div_output_0, false);
			let mut _classifier_classifier_0_Cast_output_0: Option<DeviceMemoryHandleRaw> = _classifier_classifier_0_Div_output_0;
			// cast operation
			let mut _classifier_classifier_0_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _classifier_classifier_0_Cast_output_0;
			// add operation
			let kernel__classifier_classifier_0_Add = &kernels[54];
			let _classifier_classifier_0_Constant_2_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_0_Constant_2_output_0, true);
			let mut _classifier_classifier_0_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_0_Add, _classifier_classifier_0_Cast_1_output_0, _classifier_classifier_0_Constant_2_output_0, mut _classifier_classifier_0_Add_output_0);
			// cast operation
			let mut _classifier_classifier_1_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _classifier_classifier_0_Add_output_0;
			// relu operation
			let kernel__classifier_classifier_1_relu_Relu = &kernels[55];
			let _classifier_classifier_1_relu_Relu_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_1_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__classifier_classifier_1_relu_Relu, _classifier_classifier_1_relu_Cast_output_0, _classifier_classifier_1_relu_Relu_output_0);
			// cast operation
			let _classifier_classifier_1_relu_Relu_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_1_relu_Relu_output_0, false);
			let mut _classifier_classifier_1_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _classifier_classifier_1_relu_Relu_output_0;
			// mul operation
			let kernel__classifier_classifier_3_Mul = &kernels[56];
			let _classifier_classifier_3_linear_MatMul_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_linear_MatMul_output_0, false);
			let _classifier_classifier_3_Constant_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_Constant_output_0, true);
			let mut _classifier_classifier_3_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_3_Mul, _classifier_classifier_3_linear_MatMul_output_0, _classifier_classifier_3_Constant_output_0, mut _classifier_classifier_3_Mul_output_0);
			// div operation
			let kernel__classifier_classifier_3_Div = &kernels[57];
			let _classifier_classifier_3_Constant_1_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_Constant_1_output_0, true);
			let _classifier_classifier_3_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_Div_output_0, false);
			let _classifier_classifier_3_Div_output_0_r = ctx.copy_to_device(&assignment._classifier_classifier_3_Div_output_0_r, false);
			call_kernel!(ctx, kernel__classifier_classifier_3_Div, _classifier_classifier_3_Mul_output_0, _classifier_classifier_3_Constant_1_output_0, _classifier_classifier_3_Div_output_0, _classifier_classifier_3_Div_output_0_r);
			// cast operation
			let _classifier_classifier_3_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_Div_output_0, false);
			let mut _classifier_classifier_3_Cast_output_0: Option<DeviceMemoryHandleRaw> = _classifier_classifier_3_Div_output_0;
			// cast operation
			let mut _classifier_classifier_3_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _classifier_classifier_3_Cast_output_0;
			// add operation
			let kernel__classifier_classifier_3_Add = &kernels[58];
			let _classifier_classifier_3_Constant_2_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_3_Constant_2_output_0, true);
			let mut _classifier_classifier_3_Add_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_3_Add, _classifier_classifier_3_Cast_1_output_0, _classifier_classifier_3_Constant_2_output_0, mut _classifier_classifier_3_Add_output_0);
			// cast operation
			let mut _classifier_classifier_4_relu_Cast_output_0: Option<DeviceMemoryHandleRaw> = _classifier_classifier_3_Add_output_0;
			// relu operation
			let kernel__classifier_classifier_4_relu_Relu = &kernels[59];
			let _classifier_classifier_4_relu_Relu_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_4_relu_Relu_output_0, false);
			call_kernel!(ctx, kernel__classifier_classifier_4_relu_Relu, _classifier_classifier_4_relu_Cast_output_0, _classifier_classifier_4_relu_Relu_output_0);
			// cast operation
			let _classifier_classifier_4_relu_Relu_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_4_relu_Relu_output_0, false);
			let mut _classifier_classifier_4_relu_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _classifier_classifier_4_relu_Relu_output_0;
			// mul operation
			let kernel__classifier_classifier_6_Mul = &kernels[60];
			let _classifier_classifier_6_linear_MatMul_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_linear_MatMul_output_0, false);
			let _classifier_classifier_6_Constant_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_Constant_output_0, true);
			let mut _classifier_classifier_6_Mul_output_0: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_6_Mul, _classifier_classifier_6_linear_MatMul_output_0, _classifier_classifier_6_Constant_output_0, mut _classifier_classifier_6_Mul_output_0);
			// div operation
			let kernel__classifier_classifier_6_Div = &kernels[61];
			let _classifier_classifier_6_Constant_1_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_Constant_1_output_0, true);
			let _classifier_classifier_6_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_Div_output_0, false);
			let _classifier_classifier_6_Div_output_0_r = ctx.copy_to_device(&assignment._classifier_classifier_6_Div_output_0_r, false);
			call_kernel!(ctx, kernel__classifier_classifier_6_Div, _classifier_classifier_6_Mul_output_0, _classifier_classifier_6_Constant_1_output_0, _classifier_classifier_6_Div_output_0, _classifier_classifier_6_Div_output_0_r);
			// cast operation
			let _classifier_classifier_6_Div_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_Div_output_0, false);
			let mut _classifier_classifier_6_Cast_output_0: Option<DeviceMemoryHandleRaw> = _classifier_classifier_6_Div_output_0;
			// cast operation
			let mut _classifier_classifier_6_Cast_1_output_0: Option<DeviceMemoryHandleRaw> = _classifier_classifier_6_Cast_output_0;
			// add operation
			let kernel__classifier_classifier_6_Add = &kernels[62];
			let _classifier_classifier_6_Constant_2_output_0 = ctx.copy_to_device(&assignment._classifier_classifier_6_Constant_2_output_0, true);
			let mut output: Option<DeviceMemoryHandleRaw> = None;
			call_kernel!(ctx, kernel__classifier_classifier_6_Add, _classifier_classifier_6_Cast_1_output_0, _classifier_classifier_6_Constant_2_output_0, mut output);
			// let computation_graph = ctx.to_computation_graph();
			// let file = std::fs::File::create("graph.txt").unwrap();
			// let writer = std::io::BufWriter::new(file);
			// computation_graph.serialize_into(writer);
			// let (prover_setup, _) = ctx.proving_system_setup(&computation_graph);
			let prover_setup = &*SETUPS.0;
			let proof = ctx.to_proof(&prover_setup);
			let file = std::fs::File::create("proof.txt").unwrap();
			let writer = std::io::BufWriter::new(file);
			proof.serialize_into(writer);
			println!("prove done!");
		}
	);
}

pub fn run_verifier(){
	let compile_result = stacker::grow(32 * 1024 * 1024 * 1024, ||
		{
			let start_time = std::time::Instant::now();
			let computation_graph = &*GRAPH;
			println!("refer graph time: {:?}", start_time.elapsed());
			let verifier_setup = &*SETUPS.1;
			println!("setup time: {:?}", start_time.elapsed());
			let file = std::fs::File::open("proof.txt").unwrap();
			let reader = std::io::BufReader::new(file);
			let proof = CombinedProof::<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>>::deserialize_from(reader).unwrap();
			println!("read proof time: {:?}", start_time.elapsed());
			assert!(computation_graph.verify(&proof, &verifier_setup));
		}
	);
}
#[tokio::main]
async fn main() {
	let kernels = &*KERNELS;
	let graph = &*GRAPH;
	let prover_setup = &*SETUPS.0;
	let verifier_setup = &*SETUPS.1;
    let route = warp::post()
        .and(warp::path::end())
        .and(warp::multipart::form().max_length(1_000_000_000))  // max: 1GB
        .and_then(handler);

    println!("Server running at http://127.0.0.1:3030");
    warp::serve(route)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
/// handle request
async fn handler(form: FormData) -> Result<impl warp::Reply, Infallible> {
	// tokio::fs::write("input_save.json", &body).await.ok();
	let mut req_type: i32 = 0;
	let mut input_bytes: Vec<u8> = Vec::new();
	let mut parts = form.into_stream();

	while let Some(part_res) = parts.next().await {
		let mut part = match part_res {
			Ok(p) => p,
			Err(_) => continue, 
		};
		match part.name() {
			"type" => {
				let data = part
					.stream()
					.try_fold(Vec::new(), |mut acc, chunk| async move {
						acc.extend_from_slice(chunk.chunk());
						Ok(acc)
					})
					.await
					.unwrap_or_default();
				req_type = String::from_utf8(data).unwrap().parse().unwrap();
			}
			"file" => {
				let data = part
					.stream()
					.try_fold(Vec::new(), |mut acc, chunk| async move {
						acc.extend_from_slice(chunk.chunk());
						Ok(acc)
					})
					.await
					.unwrap_or_default();
				input_bytes = data;
			}
			_ => {}
		}
    }
	match req_type {
        1 => {
			let start_time = std::time::Instant::now();
			let kernels: &[Kernel<BN254Config>] = &KERNELS;
			println!("read kernels time: {:?}", start_time.elapsed());
			let input_str = String::from_utf8(input_bytes.clone()).unwrap();
			// let input_str = fs::read_to_string("input_save.json").unwrap();
			let input: Circuit_Input = serde_json::from_str(&input_str).unwrap();
			let mut assignment = input_copy(&input);
			println!("start_proving");
            run_prover(assignment, kernels);
        }
        2 => {
			println!("start_verifying");
            run_verifier();
			println!("verify done!");
        }
        _ => {
			;
        }
    }
	// run_prover(assignment, kernels);
	
    match tokio::fs::read("proof.txt").await {
        Ok(bytes) => {
            let resp = warp::http::Response::builder()
                .header("Content-Type", "application/octet-stream")
                .header(
                    "Content-Disposition",
                    "attachment; filename=\"proof.txt\"",
                )
                .body(bytes)
                .unwrap();
            Ok(resp)
        }
        Err(err) => {
            let msg = format!("Failed to read proof.txt: {}", err);
            let resp = warp::http::Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "text/plain")
                .body(msg.into_bytes())
                .unwrap();
            Ok(resp)
        }
    }
}


// #[cfg(test)]
// mod tests {
// 	use super::*;
//     #[tokio::test]
//     async fn test_prover() {
//         let reply = handler().await.unwrap();
//         let resp = reply.into_response();
//         assert_eq!(resp.status(), StatusCode::OK);
//         let body_bytes = warp::hyper::body::to_bytes(resp.into_body()).await.unwrap();
//         let body = std::str::from_utf8(&body_bytes).unwrap();
//         assert!(body.contains("Prove done"));
//     }
// }
