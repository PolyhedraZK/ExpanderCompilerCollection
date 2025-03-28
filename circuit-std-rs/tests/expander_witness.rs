use expander_compiler::frontend::*;
use expander_compiler::frontend::extra::debug_eval;
use expander_compiler::circuit::layered::{NormalInputType, CrossLayerInputType};
use expander_compiler::Proof;
use gkr::BN254ConfigSha2Raw;
use extra::Serde;
use serde::{Serialize, Deserialize};
use arith::FieldSerde;
use stacker;
use std::fs;
use std::time::Instant;
use circuit_std_rs::{
    logup::{query_count_hint, rangeproof_hint, LogUpRangeProofTable},
    LogUpCircuit, LogUpParams,
};
declare_circuit!(Circuit {
	output: [[Variable]], 
	input: [[[[Variable]]]], 
	_features_features_0_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_0_Constant_output_0: Variable, 
	_features_features_0_Constant_1_output_0: Variable, 
	_features_features_0_Div_output_0_r: [[[[Variable]]]], 
	_features_features_0_Div_output_0: [[[[Variable]]]], 
	_features_features_0_Constant_2_output_0: [[[Variable]]], 
	_features_features_2_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_3_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_3_Constant_output_0: Variable, 
	_features_features_3_Constant_1_output_0: Variable, 
	_features_features_3_Div_output_0_r: [[[[Variable]]]], 
	_features_features_3_Div_output_0: [[[[Variable]]]], 
	_features_features_3_Constant_2_output_0: [[[Variable]]], 
	_features_features_5_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_6_maxpool_MaxPool_output_0: [[[[Variable]]]], 
	_features_features_7_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_7_Constant_output_0: Variable, 
	_features_features_7_Constant_1_output_0: Variable, 
	_features_features_7_Div_output_0_r: [[[[Variable]]]], 
	_features_features_7_Div_output_0: [[[[Variable]]]], 
	_features_features_7_Constant_2_output_0: [[[Variable]]], 
	_features_features_9_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_10_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_10_Constant_output_0: Variable, 
	_features_features_10_Constant_1_output_0: Variable, 
	_features_features_10_Div_output_0_r: [[[[Variable]]]], 
	_features_features_10_Div_output_0: [[[[Variable]]]], 
	_features_features_10_Constant_2_output_0: [[[Variable]]], 
	_features_features_12_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_13_maxpool_MaxPool_output_0: [[[[Variable]]]], 
	_features_features_14_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_14_Constant_output_0: Variable, 
	_features_features_14_Constant_1_output_0: Variable, 
	_features_features_14_Div_output_0_r: [[[[Variable]]]], 
	_features_features_14_Div_output_0: [[[[Variable]]]], 
	_features_features_14_Constant_2_output_0: [[[Variable]]], 
	_features_features_16_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_17_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_17_Constant_output_0: Variable, 
	_features_features_17_Constant_1_output_0: Variable, 
	_features_features_17_Div_output_0_r: [[[[Variable]]]], 
	_features_features_17_Div_output_0: [[[[Variable]]]], 
	_features_features_17_Constant_2_output_0: [[[Variable]]], 
	_features_features_19_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_20_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_20_Constant_output_0: Variable, 
	_features_features_20_Constant_1_output_0: Variable, 
	_features_features_20_Div_output_0_r: [[[[Variable]]]], 
	_features_features_20_Div_output_0: [[[[Variable]]]], 
	_features_features_20_Constant_2_output_0: [[[Variable]]], 
	_features_features_22_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_23_maxpool_MaxPool_output_0: [[[[Variable]]]], 
	_features_features_24_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_24_Constant_output_0: Variable, 
	_features_features_24_Constant_1_output_0: Variable, 
	_features_features_24_Div_output_0_r: [[[[Variable]]]], 
	_features_features_24_Div_output_0: [[[[Variable]]]], 
	_features_features_24_Constant_2_output_0: [[[Variable]]], 
	_features_features_26_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_27_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_27_Constant_output_0: Variable, 
	_features_features_27_Constant_1_output_0: Variable, 
	_features_features_27_Div_output_0_r: [[[[Variable]]]], 
	_features_features_27_Div_output_0: [[[[Variable]]]], 
	_features_features_27_Constant_2_output_0: [[[Variable]]], 
	_features_features_29_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_30_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_30_Constant_output_0: Variable, 
	_features_features_30_Constant_1_output_0: Variable, 
	_features_features_30_Div_output_0_r: [[[[Variable]]]], 
	_features_features_30_Div_output_0: [[[[Variable]]]], 
	_features_features_30_Constant_2_output_0: [[[Variable]]], 
	_features_features_32_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_33_maxpool_MaxPool_output_0: [[[[Variable]]]], 
	_features_features_34_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_34_Constant_output_0: Variable, 
	_features_features_34_Constant_1_output_0: Variable, 
	_features_features_34_Div_output_0_r: [[[[Variable]]]], 
	_features_features_34_Div_output_0: [[[[Variable]]]], 
	_features_features_34_Constant_2_output_0: [[[Variable]]], 
	_features_features_36_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_37_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_37_Constant_output_0: Variable, 
	_features_features_37_Constant_1_output_0: Variable, 
	_features_features_37_Div_output_0_r: [[[[Variable]]]], 
	_features_features_37_Div_output_0: [[[[Variable]]]], 
	_features_features_37_Constant_2_output_0: [[[Variable]]], 
	_features_features_39_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_40_conv_Conv_output_0: [[[[Variable]]]], 
	_features_features_40_Constant_output_0: Variable, 
	_features_features_40_Constant_1_output_0: Variable, 
	_features_features_40_Div_output_0_r: [[[[Variable]]]], 
	_features_features_40_Div_output_0: [[[[Variable]]]], 
	_features_features_40_Constant_2_output_0: [[[Variable]]], 
	_features_features_42_relu_Relu_output_0: [[[[Variable]]]], 
	_features_features_43_maxpool_MaxPool_output_0: [[[[Variable]]]], 
	_classifier_classifier_0_linear_MatMul_output_0: [[Variable]], 
	_classifier_classifier_0_Constant_output_0: Variable, 
	_classifier_classifier_0_Constant_1_output_0: Variable, 
	_classifier_classifier_0_Div_output_0_r: [[Variable]], 
	_classifier_classifier_0_Div_output_0: [[Variable]], 
	_classifier_classifier_0_Constant_2_output_0: [Variable], 
	_classifier_classifier_1_relu_Relu_output_0: [[Variable]], 
	_classifier_classifier_3_linear_MatMul_output_0: [[Variable]], 
	_classifier_classifier_3_Constant_output_0: Variable, 
	_classifier_classifier_3_Constant_1_output_0: Variable, 
	_classifier_classifier_3_Div_output_0_r: [[Variable]], 
	_classifier_classifier_3_Div_output_0: [[Variable]], 
	_classifier_classifier_3_Constant_2_output_0: [Variable], 
	_classifier_classifier_4_relu_Relu_output_0: [[Variable]], 
	_classifier_classifier_6_linear_MatMul_output_0: [[Variable]], 
	_classifier_classifier_6_Constant_output_0: Variable, 
	_classifier_classifier_6_Constant_1_output_0: Variable, 
	_classifier_classifier_6_Div_output_0_r: [[Variable]], 
	_classifier_classifier_6_Div_output_0: [[Variable]], 
	_classifier_classifier_6_Constant_2_output_0: [Variable], 
	features_0_conv_weight: [[[[Variable]]]], 
	features_3_conv_weight: [[[[Variable]]]], 
	features_7_conv_weight: [[[[Variable]]]], 
	features_10_conv_weight: [[[[Variable]]]], 
	features_14_conv_weight: [[[[Variable]]]], 
	features_17_conv_weight: [[[[Variable]]]], 
	features_20_conv_weight: [[[[Variable]]]], 
	features_24_conv_weight: [[[[Variable]]]], 
	features_27_conv_weight: [[[[Variable]]]], 
	features_30_conv_weight: [[[[Variable]]]], 
	features_34_conv_weight: [[[[Variable]]]], 
	features_37_conv_weight: [[[[Variable]]]], 
	features_40_conv_weight: [[[[Variable]]]], 
	onnx__MatMul_215: [[Variable]], 
	onnx__MatMul_216: [[Variable]], 
	onnx__MatMul_217: [[Variable]], 
	input_mat_ru: [Variable], 
	features_0_conv_weight_mat_rv: [Variable], 
	_features_features_2_relu_Cast_1_output_0_mat_ru: [Variable], 
	features_3_conv_weight_mat_rv: [Variable], 
	_features_features_6_maxpool_MaxPool_output_0_mat_ru: [Variable], 
	features_7_conv_weight_mat_rv: [Variable], 
	_features_features_9_relu_Cast_1_output_0_mat_ru: [Variable], 
	features_10_conv_weight_mat_rv: [Variable], 
	_features_features_13_maxpool_MaxPool_output_0_mat_ru: [Variable], 
	features_14_conv_weight_mat_rv: [Variable], 
	_features_features_16_relu_Cast_1_output_0_mat_ru: [Variable], 
	features_17_conv_weight_mat_rv: [Variable], 
	_features_features_19_relu_Cast_1_output_0_mat_ru: [Variable], 
	features_20_conv_weight_mat_rv: [Variable], 
	_features_features_23_maxpool_MaxPool_output_0_mat_ru: [Variable], 
	features_24_conv_weight_mat_rv: [Variable], 
	_features_features_26_relu_Cast_1_output_0_mat_ru: [Variable], 
	features_27_conv_weight_mat_rv: [Variable], 
	_features_features_29_relu_Cast_1_output_0_mat_ru: [Variable], 
	features_30_conv_weight_mat_rv: [Variable], 
	_features_features_33_maxpool_MaxPool_output_0_mat_ru: [Variable], 
	features_34_conv_weight_mat_rv: [Variable], 
	_features_features_36_relu_Cast_1_output_0_mat_ru: [Variable], 
	features_37_conv_weight_mat_rv: [Variable], 
	_features_features_39_relu_Cast_1_output_0_mat_ru: [Variable], 
	features_40_conv_weight_mat_rv: [Variable], 
	_Flatten_output_0_mat_ru: [Variable], 
	onnx__MatMul_215_mat_rv: [Variable], 
	_classifier_classifier_1_relu_Cast_1_output_0_mat_ru: [Variable], 
	onnx__MatMul_216_mat_rv: [Variable], 
	_classifier_classifier_4_relu_Cast_1_output_0_mat_ru: [Variable], 
	onnx__MatMul_217_mat_rv: [Variable], 
});

#[derive(Serialize, Deserialize, Debug)]
struct Circuit_Input {
	output: Vec<Vec<i64>>, 
	input: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_Constant_output_0: i64, 
	_features_features_0_Constant_1_output_0: i64, 
	_features_features_0_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_0_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_2_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_Constant_output_0: i64, 
	_features_features_3_Constant_1_output_0: i64, 
	_features_features_3_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_3_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_5_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_6_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_7_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_7_Constant_output_0: i64, 
	_features_features_7_Constant_1_output_0: i64, 
	_features_features_7_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_7_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_7_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_9_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_10_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_10_Constant_output_0: i64, 
	_features_features_10_Constant_1_output_0: i64, 
	_features_features_10_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_10_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_10_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_12_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_13_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_14_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_14_Constant_output_0: i64, 
	_features_features_14_Constant_1_output_0: i64, 
	_features_features_14_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_14_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_14_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_16_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_17_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_17_Constant_output_0: i64, 
	_features_features_17_Constant_1_output_0: i64, 
	_features_features_17_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_17_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_17_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_19_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_20_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_20_Constant_output_0: i64, 
	_features_features_20_Constant_1_output_0: i64, 
	_features_features_20_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_20_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_20_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_22_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_23_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_24_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_24_Constant_output_0: i64, 
	_features_features_24_Constant_1_output_0: i64, 
	_features_features_24_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_24_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_24_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_26_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_27_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_27_Constant_output_0: i64, 
	_features_features_27_Constant_1_output_0: i64, 
	_features_features_27_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_27_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_27_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_29_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_30_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_30_Constant_output_0: i64, 
	_features_features_30_Constant_1_output_0: i64, 
	_features_features_30_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_30_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_30_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_32_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_33_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_34_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_34_Constant_output_0: i64, 
	_features_features_34_Constant_1_output_0: i64, 
	_features_features_34_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_34_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_34_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_36_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_37_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_37_Constant_output_0: i64, 
	_features_features_37_Constant_1_output_0: i64, 
	_features_features_37_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_37_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_37_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_39_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_40_conv_Conv_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_40_Constant_output_0: i64, 
	_features_features_40_Constant_1_output_0: i64, 
	_features_features_40_Div_output_0_r: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_40_Div_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_40_Constant_2_output_0: Vec<Vec<Vec<i64>>>, 
	_features_features_42_relu_Relu_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_features_features_43_maxpool_MaxPool_output_0: Vec<Vec<Vec<Vec<i64>>>>, 
	_classifier_classifier_0_linear_MatMul_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_0_Constant_output_0: i64, 
	_classifier_classifier_0_Constant_1_output_0: i64, 
	_classifier_classifier_0_Div_output_0_r: Vec<Vec<i64>>, 
	_classifier_classifier_0_Div_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_0_Constant_2_output_0: Vec<i64>, 
	_classifier_classifier_1_relu_Relu_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_3_linear_MatMul_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_3_Constant_output_0: i64, 
	_classifier_classifier_3_Constant_1_output_0: i64, 
	_classifier_classifier_3_Div_output_0_r: Vec<Vec<i64>>, 
	_classifier_classifier_3_Div_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_3_Constant_2_output_0: Vec<i64>, 
	_classifier_classifier_4_relu_Relu_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_6_linear_MatMul_output_0: Vec<Vec<i64>>, 
	_classifier_classifier_6_Constant_output_0: i64, 
	_classifier_classifier_6_Constant_1_output_0: i64, 
	_classifier_classifier_6_Div_output_0_r: Vec<Vec<i64>>, 
	_classifier_classifier_6_Div_output_0: Vec<Vec<i64>>, 
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

fn input_copy(input: &Circuit_Input, assignment: &mut Circuit::<BN254>){
	assignment.input_mat_ru = vec![BN254::default();16384]; 
	assignment.features_0_conv_weight_mat_rv = vec![BN254::default();64]; 
	assignment._features_features_2_relu_Cast_1_output_0_mat_ru = vec![BN254::default();16384]; 
	assignment.features_3_conv_weight_mat_rv = vec![BN254::default();64]; 
	assignment._features_features_6_maxpool_MaxPool_output_0_mat_ru = vec![BN254::default();4096]; 
	assignment.features_7_conv_weight_mat_rv = vec![BN254::default();128]; 
	assignment._features_features_9_relu_Cast_1_output_0_mat_ru = vec![BN254::default();4096]; 
	assignment.features_10_conv_weight_mat_rv = vec![BN254::default();128]; 
	assignment._features_features_13_maxpool_MaxPool_output_0_mat_ru = vec![BN254::default();1024]; 
	assignment.features_14_conv_weight_mat_rv = vec![BN254::default();256]; 
	assignment._features_features_16_relu_Cast_1_output_0_mat_ru = vec![BN254::default();1024]; 
	assignment.features_17_conv_weight_mat_rv = vec![BN254::default();256]; 
	assignment._features_features_19_relu_Cast_1_output_0_mat_ru = vec![BN254::default();1024]; 
	assignment.features_20_conv_weight_mat_rv = vec![BN254::default();256]; 
	assignment._features_features_23_maxpool_MaxPool_output_0_mat_ru = vec![BN254::default();256]; 
	assignment.features_24_conv_weight_mat_rv = vec![BN254::default();512]; 
	assignment._features_features_26_relu_Cast_1_output_0_mat_ru = vec![BN254::default();256]; 
	assignment.features_27_conv_weight_mat_rv = vec![BN254::default();512]; 
	assignment._features_features_29_relu_Cast_1_output_0_mat_ru = vec![BN254::default();256]; 
	assignment.features_30_conv_weight_mat_rv = vec![BN254::default();512]; 
	assignment._features_features_33_maxpool_MaxPool_output_0_mat_ru = vec![BN254::default();64]; 
	assignment.features_34_conv_weight_mat_rv = vec![BN254::default();512]; 
	assignment._features_features_36_relu_Cast_1_output_0_mat_ru = vec![BN254::default();64]; 
	assignment.features_37_conv_weight_mat_rv = vec![BN254::default();512]; 
	assignment._features_features_39_relu_Cast_1_output_0_mat_ru = vec![BN254::default();64]; 
	assignment.features_40_conv_weight_mat_rv = vec![BN254::default();512]; 
	assignment._Flatten_output_0_mat_ru = vec![BN254::default();16]; 
	assignment.onnx__MatMul_215_mat_rv = vec![BN254::default();512]; 
	assignment._classifier_classifier_1_relu_Cast_1_output_0_mat_ru = vec![BN254::default();16]; 
	assignment.onnx__MatMul_216_mat_rv = vec![BN254::default();512]; 
	assignment._classifier_classifier_4_relu_Cast_1_output_0_mat_ru = vec![BN254::default();16]; 
	assignment.onnx__MatMul_217_mat_rv = vec![BN254::default();10]; 
	assignment.output = vec![vec![BN254::default();10];16]; 
	for i in 0..16 {
		for j in 0..10 {
			if input.output[i][j] >= 0{
				assignment.output[i][j] = BN254::from((input.output[i][j]) as u64); 
			} else {
				assignment.output[i][j] = -BN254::from((-input.output[i][j]) as u64); 
			} 
		}
	}
	assignment.input = vec![vec![vec![vec![BN254::default();32];32];3];16]; 
	for i in 0..16 {
		for j in 0..3 {
			for k in 0..32 {
				for l in 0..32 {
					if input.input[i][j][k][l] >= 0{
						assignment.input[i][j][k][l] = BN254::from((input.input[i][j][k][l]) as u64); 
					} else {
						assignment.input[i][j][k][l] = -BN254::from((-input.input[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_0_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_0_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_0_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_0_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_0_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_0_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_0_Constant_output_0 = BN254::default(); 
	if input._features_features_0_Constant_output_0 >= 0{
		assignment._features_features_0_Constant_output_0 = BN254::from((input._features_features_0_Constant_output_0) as u64); 
	} else {
		assignment._features_features_0_Constant_output_0 = -BN254::from((-input._features_features_0_Constant_output_0) as u64); 
	} 
	assignment._features_features_0_Constant_1_output_0 = BN254::default(); 
	if input._features_features_0_Constant_1_output_0 >= 0{
		assignment._features_features_0_Constant_1_output_0 = BN254::from((input._features_features_0_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_0_Constant_1_output_0 = -BN254::from((-input._features_features_0_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_0_Div_output_0_r = vec![vec![vec![vec![BN254::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_0_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_0_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_0_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_0_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_0_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_0_Div_output_0 = vec![vec![vec![vec![BN254::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_0_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_0_Div_output_0[i][j][k][l] = BN254::from((input._features_features_0_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_0_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_0_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_0_Constant_2_output_0 = vec![vec![vec![BN254::default();32];32];64]; 
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				if input._features_features_0_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_0_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_0_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_0_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_0_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_2_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_2_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_2_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_2_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_2_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_2_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_3_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_3_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_3_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_3_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_3_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_3_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_3_Constant_output_0 = BN254::default(); 
	if input._features_features_3_Constant_output_0 >= 0{
		assignment._features_features_3_Constant_output_0 = BN254::from((input._features_features_3_Constant_output_0) as u64); 
	} else {
		assignment._features_features_3_Constant_output_0 = -BN254::from((-input._features_features_3_Constant_output_0) as u64); 
	} 
	assignment._features_features_3_Constant_1_output_0 = BN254::default(); 
	if input._features_features_3_Constant_1_output_0 >= 0{
		assignment._features_features_3_Constant_1_output_0 = BN254::from((input._features_features_3_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_3_Constant_1_output_0 = -BN254::from((-input._features_features_3_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_3_Div_output_0_r = vec![vec![vec![vec![BN254::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_3_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_3_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_3_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_3_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_3_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_3_Div_output_0 = vec![vec![vec![vec![BN254::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_3_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_3_Div_output_0[i][j][k][l] = BN254::from((input._features_features_3_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_3_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_3_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_3_Constant_2_output_0 = vec![vec![vec![BN254::default();32];32];64]; 
	for i in 0..64 {
		for j in 0..32 {
			for k in 0..32 {
				if input._features_features_3_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_3_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_3_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_3_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_3_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_5_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();32];32];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..32 {
				for l in 0..32 {
					if input._features_features_5_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_5_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_5_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_5_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_5_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_6_maxpool_MaxPool_output_0 = vec![vec![vec![vec![BN254::default();16];16];64];16]; 
	for i in 0..16 {
		for j in 0..64 {
			for k in 0..16 {
				for l in 0..16 {
					if input._features_features_6_maxpool_MaxPool_output_0[i][j][k][l] >= 0{
						assignment._features_features_6_maxpool_MaxPool_output_0[i][j][k][l] = BN254::from((input._features_features_6_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_6_maxpool_MaxPool_output_0[i][j][k][l] = -BN254::from((-input._features_features_6_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_7_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if input._features_features_7_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_7_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_7_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_7_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_7_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_7_Constant_output_0 = BN254::default(); 
	if input._features_features_7_Constant_output_0 >= 0{
		assignment._features_features_7_Constant_output_0 = BN254::from((input._features_features_7_Constant_output_0) as u64); 
	} else {
		assignment._features_features_7_Constant_output_0 = -BN254::from((-input._features_features_7_Constant_output_0) as u64); 
	} 
	assignment._features_features_7_Constant_1_output_0 = BN254::default(); 
	if input._features_features_7_Constant_1_output_0 >= 0{
		assignment._features_features_7_Constant_1_output_0 = BN254::from((input._features_features_7_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_7_Constant_1_output_0 = -BN254::from((-input._features_features_7_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_7_Div_output_0_r = vec![vec![vec![vec![BN254::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if input._features_features_7_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_7_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_7_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_7_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_7_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_7_Div_output_0 = vec![vec![vec![vec![BN254::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if input._features_features_7_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_7_Div_output_0[i][j][k][l] = BN254::from((input._features_features_7_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_7_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_7_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_7_Constant_2_output_0 = vec![vec![vec![BN254::default();16];16];128]; 
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				if input._features_features_7_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_7_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_7_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_7_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_7_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_9_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if input._features_features_9_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_9_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_9_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_9_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_9_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_10_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if input._features_features_10_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_10_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_10_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_10_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_10_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_10_Constant_output_0 = BN254::default(); 
	if input._features_features_10_Constant_output_0 >= 0{
		assignment._features_features_10_Constant_output_0 = BN254::from((input._features_features_10_Constant_output_0) as u64); 
	} else {
		assignment._features_features_10_Constant_output_0 = -BN254::from((-input._features_features_10_Constant_output_0) as u64); 
	} 
	assignment._features_features_10_Constant_1_output_0 = BN254::default(); 
	if input._features_features_10_Constant_1_output_0 >= 0{
		assignment._features_features_10_Constant_1_output_0 = BN254::from((input._features_features_10_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_10_Constant_1_output_0 = -BN254::from((-input._features_features_10_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_10_Div_output_0_r = vec![vec![vec![vec![BN254::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if input._features_features_10_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_10_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_10_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_10_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_10_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_10_Div_output_0 = vec![vec![vec![vec![BN254::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if input._features_features_10_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_10_Div_output_0[i][j][k][l] = BN254::from((input._features_features_10_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_10_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_10_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_10_Constant_2_output_0 = vec![vec![vec![BN254::default();16];16];128]; 
	for i in 0..128 {
		for j in 0..16 {
			for k in 0..16 {
				if input._features_features_10_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_10_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_10_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_10_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_10_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_12_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();16];16];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..16 {
				for l in 0..16 {
					if input._features_features_12_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_12_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_12_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_12_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_12_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_13_maxpool_MaxPool_output_0 = vec![vec![vec![vec![BN254::default();8];8];128];16]; 
	for i in 0..16 {
		for j in 0..128 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_13_maxpool_MaxPool_output_0[i][j][k][l] >= 0{
						assignment._features_features_13_maxpool_MaxPool_output_0[i][j][k][l] = BN254::from((input._features_features_13_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_13_maxpool_MaxPool_output_0[i][j][k][l] = -BN254::from((-input._features_features_13_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_14_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_14_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_14_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_14_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_14_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_14_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_14_Constant_output_0 = BN254::default(); 
	if input._features_features_14_Constant_output_0 >= 0{
		assignment._features_features_14_Constant_output_0 = BN254::from((input._features_features_14_Constant_output_0) as u64); 
	} else {
		assignment._features_features_14_Constant_output_0 = -BN254::from((-input._features_features_14_Constant_output_0) as u64); 
	} 
	assignment._features_features_14_Constant_1_output_0 = BN254::default(); 
	if input._features_features_14_Constant_1_output_0 >= 0{
		assignment._features_features_14_Constant_1_output_0 = BN254::from((input._features_features_14_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_14_Constant_1_output_0 = -BN254::from((-input._features_features_14_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_14_Div_output_0_r = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_14_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_14_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_14_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_14_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_14_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_14_Div_output_0 = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_14_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_14_Div_output_0[i][j][k][l] = BN254::from((input._features_features_14_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_14_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_14_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_14_Constant_2_output_0 = vec![vec![vec![BN254::default();8];8];256]; 
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				if input._features_features_14_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_14_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_14_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_14_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_14_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_16_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_16_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_16_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_16_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_16_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_16_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_17_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_17_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_17_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_17_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_17_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_17_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_17_Constant_output_0 = BN254::default(); 
	if input._features_features_17_Constant_output_0 >= 0{
		assignment._features_features_17_Constant_output_0 = BN254::from((input._features_features_17_Constant_output_0) as u64); 
	} else {
		assignment._features_features_17_Constant_output_0 = -BN254::from((-input._features_features_17_Constant_output_0) as u64); 
	} 
	assignment._features_features_17_Constant_1_output_0 = BN254::default(); 
	if input._features_features_17_Constant_1_output_0 >= 0{
		assignment._features_features_17_Constant_1_output_0 = BN254::from((input._features_features_17_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_17_Constant_1_output_0 = -BN254::from((-input._features_features_17_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_17_Div_output_0_r = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_17_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_17_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_17_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_17_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_17_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_17_Div_output_0 = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_17_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_17_Div_output_0[i][j][k][l] = BN254::from((input._features_features_17_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_17_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_17_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_17_Constant_2_output_0 = vec![vec![vec![BN254::default();8];8];256]; 
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				if input._features_features_17_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_17_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_17_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_17_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_17_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_19_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_19_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_19_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_19_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_19_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_19_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_20_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_20_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_20_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_20_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_20_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_20_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_20_Constant_output_0 = BN254::default(); 
	if input._features_features_20_Constant_output_0 >= 0{
		assignment._features_features_20_Constant_output_0 = BN254::from((input._features_features_20_Constant_output_0) as u64); 
	} else {
		assignment._features_features_20_Constant_output_0 = -BN254::from((-input._features_features_20_Constant_output_0) as u64); 
	} 
	assignment._features_features_20_Constant_1_output_0 = BN254::default(); 
	if input._features_features_20_Constant_1_output_0 >= 0{
		assignment._features_features_20_Constant_1_output_0 = BN254::from((input._features_features_20_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_20_Constant_1_output_0 = -BN254::from((-input._features_features_20_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_20_Div_output_0_r = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_20_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_20_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_20_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_20_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_20_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_20_Div_output_0 = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_20_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_20_Div_output_0[i][j][k][l] = BN254::from((input._features_features_20_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_20_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_20_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_20_Constant_2_output_0 = vec![vec![vec![BN254::default();8];8];256]; 
	for i in 0..256 {
		for j in 0..8 {
			for k in 0..8 {
				if input._features_features_20_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_20_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_20_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_20_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_20_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_22_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();8];8];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..8 {
				for l in 0..8 {
					if input._features_features_22_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_22_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_22_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_22_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_22_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_23_maxpool_MaxPool_output_0 = vec![vec![vec![vec![BN254::default();4];4];256];16]; 
	for i in 0..16 {
		for j in 0..256 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_23_maxpool_MaxPool_output_0[i][j][k][l] >= 0{
						assignment._features_features_23_maxpool_MaxPool_output_0[i][j][k][l] = BN254::from((input._features_features_23_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_23_maxpool_MaxPool_output_0[i][j][k][l] = -BN254::from((-input._features_features_23_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_24_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_24_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_24_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_24_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_24_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_24_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_24_Constant_output_0 = BN254::default(); 
	if input._features_features_24_Constant_output_0 >= 0{
		assignment._features_features_24_Constant_output_0 = BN254::from((input._features_features_24_Constant_output_0) as u64); 
	} else {
		assignment._features_features_24_Constant_output_0 = -BN254::from((-input._features_features_24_Constant_output_0) as u64); 
	} 
	assignment._features_features_24_Constant_1_output_0 = BN254::default(); 
	if input._features_features_24_Constant_1_output_0 >= 0{
		assignment._features_features_24_Constant_1_output_0 = BN254::from((input._features_features_24_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_24_Constant_1_output_0 = -BN254::from((-input._features_features_24_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_24_Div_output_0_r = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_24_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_24_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_24_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_24_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_24_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_24_Div_output_0 = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_24_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_24_Div_output_0[i][j][k][l] = BN254::from((input._features_features_24_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_24_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_24_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_24_Constant_2_output_0 = vec![vec![vec![BN254::default();4];4];512]; 
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				if input._features_features_24_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_24_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_24_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_24_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_24_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_26_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_26_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_26_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_26_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_26_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_26_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_27_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_27_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_27_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_27_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_27_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_27_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_27_Constant_output_0 = BN254::default(); 
	if input._features_features_27_Constant_output_0 >= 0{
		assignment._features_features_27_Constant_output_0 = BN254::from((input._features_features_27_Constant_output_0) as u64); 
	} else {
		assignment._features_features_27_Constant_output_0 = -BN254::from((-input._features_features_27_Constant_output_0) as u64); 
	} 
	assignment._features_features_27_Constant_1_output_0 = BN254::default(); 
	if input._features_features_27_Constant_1_output_0 >= 0{
		assignment._features_features_27_Constant_1_output_0 = BN254::from((input._features_features_27_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_27_Constant_1_output_0 = -BN254::from((-input._features_features_27_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_27_Div_output_0_r = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_27_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_27_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_27_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_27_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_27_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_27_Div_output_0 = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_27_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_27_Div_output_0[i][j][k][l] = BN254::from((input._features_features_27_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_27_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_27_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_27_Constant_2_output_0 = vec![vec![vec![BN254::default();4];4];512]; 
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				if input._features_features_27_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_27_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_27_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_27_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_27_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_29_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_29_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_29_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_29_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_29_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_29_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_30_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_30_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_30_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_30_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_30_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_30_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_30_Constant_output_0 = BN254::default(); 
	if input._features_features_30_Constant_output_0 >= 0{
		assignment._features_features_30_Constant_output_0 = BN254::from((input._features_features_30_Constant_output_0) as u64); 
	} else {
		assignment._features_features_30_Constant_output_0 = -BN254::from((-input._features_features_30_Constant_output_0) as u64); 
	} 
	assignment._features_features_30_Constant_1_output_0 = BN254::default(); 
	if input._features_features_30_Constant_1_output_0 >= 0{
		assignment._features_features_30_Constant_1_output_0 = BN254::from((input._features_features_30_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_30_Constant_1_output_0 = -BN254::from((-input._features_features_30_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_30_Div_output_0_r = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_30_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_30_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_30_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_30_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_30_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_30_Div_output_0 = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_30_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_30_Div_output_0[i][j][k][l] = BN254::from((input._features_features_30_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_30_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_30_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_30_Constant_2_output_0 = vec![vec![vec![BN254::default();4];4];512]; 
	for i in 0..512 {
		for j in 0..4 {
			for k in 0..4 {
				if input._features_features_30_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_30_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_30_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_30_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_30_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_32_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();4];4];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..4 {
				for l in 0..4 {
					if input._features_features_32_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_32_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_32_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_32_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_32_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_33_maxpool_MaxPool_output_0 = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_33_maxpool_MaxPool_output_0[i][j][k][l] >= 0{
						assignment._features_features_33_maxpool_MaxPool_output_0[i][j][k][l] = BN254::from((input._features_features_33_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_33_maxpool_MaxPool_output_0[i][j][k][l] = -BN254::from((-input._features_features_33_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_34_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_34_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_34_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_34_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_34_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_34_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_34_Constant_output_0 = BN254::default(); 
	if input._features_features_34_Constant_output_0 >= 0{
		assignment._features_features_34_Constant_output_0 = BN254::from((input._features_features_34_Constant_output_0) as u64); 
	} else {
		assignment._features_features_34_Constant_output_0 = -BN254::from((-input._features_features_34_Constant_output_0) as u64); 
	} 
	assignment._features_features_34_Constant_1_output_0 = BN254::default(); 
	if input._features_features_34_Constant_1_output_0 >= 0{
		assignment._features_features_34_Constant_1_output_0 = BN254::from((input._features_features_34_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_34_Constant_1_output_0 = -BN254::from((-input._features_features_34_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_34_Div_output_0_r = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_34_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_34_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_34_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_34_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_34_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_34_Div_output_0 = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_34_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_34_Div_output_0[i][j][k][l] = BN254::from((input._features_features_34_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_34_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_34_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_34_Constant_2_output_0 = vec![vec![vec![BN254::default();2];2];512]; 
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				if input._features_features_34_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_34_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_34_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_34_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_34_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_36_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_36_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_36_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_36_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_36_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_36_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_37_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_37_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_37_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_37_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_37_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_37_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_37_Constant_output_0 = BN254::default(); 
	if input._features_features_37_Constant_output_0 >= 0{
		assignment._features_features_37_Constant_output_0 = BN254::from((input._features_features_37_Constant_output_0) as u64); 
	} else {
		assignment._features_features_37_Constant_output_0 = -BN254::from((-input._features_features_37_Constant_output_0) as u64); 
	} 
	assignment._features_features_37_Constant_1_output_0 = BN254::default(); 
	if input._features_features_37_Constant_1_output_0 >= 0{
		assignment._features_features_37_Constant_1_output_0 = BN254::from((input._features_features_37_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_37_Constant_1_output_0 = -BN254::from((-input._features_features_37_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_37_Div_output_0_r = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_37_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_37_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_37_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_37_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_37_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_37_Div_output_0 = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_37_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_37_Div_output_0[i][j][k][l] = BN254::from((input._features_features_37_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_37_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_37_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_37_Constant_2_output_0 = vec![vec![vec![BN254::default();2];2];512]; 
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				if input._features_features_37_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_37_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_37_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_37_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_37_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_39_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_39_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_39_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_39_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_39_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_39_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_40_conv_Conv_output_0 = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_40_conv_Conv_output_0[i][j][k][l] >= 0{
						assignment._features_features_40_conv_Conv_output_0[i][j][k][l] = BN254::from((input._features_features_40_conv_Conv_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_40_conv_Conv_output_0[i][j][k][l] = -BN254::from((-input._features_features_40_conv_Conv_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_40_Constant_output_0 = BN254::default(); 
	if input._features_features_40_Constant_output_0 >= 0{
		assignment._features_features_40_Constant_output_0 = BN254::from((input._features_features_40_Constant_output_0) as u64); 
	} else {
		assignment._features_features_40_Constant_output_0 = -BN254::from((-input._features_features_40_Constant_output_0) as u64); 
	} 
	assignment._features_features_40_Constant_1_output_0 = BN254::default(); 
	if input._features_features_40_Constant_1_output_0 >= 0{
		assignment._features_features_40_Constant_1_output_0 = BN254::from((input._features_features_40_Constant_1_output_0) as u64); 
	} else {
		assignment._features_features_40_Constant_1_output_0 = -BN254::from((-input._features_features_40_Constant_1_output_0) as u64); 
	} 
	assignment._features_features_40_Div_output_0_r = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_40_Div_output_0_r[i][j][k][l] >= 0{
						assignment._features_features_40_Div_output_0_r[i][j][k][l] = BN254::from((input._features_features_40_Div_output_0_r[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_40_Div_output_0_r[i][j][k][l] = -BN254::from((-input._features_features_40_Div_output_0_r[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_40_Div_output_0 = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_40_Div_output_0[i][j][k][l] >= 0{
						assignment._features_features_40_Div_output_0[i][j][k][l] = BN254::from((input._features_features_40_Div_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_40_Div_output_0[i][j][k][l] = -BN254::from((-input._features_features_40_Div_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_40_Constant_2_output_0 = vec![vec![vec![BN254::default();2];2];512]; 
	for i in 0..512 {
		for j in 0..2 {
			for k in 0..2 {
				if input._features_features_40_Constant_2_output_0[i][j][k] >= 0{
					assignment._features_features_40_Constant_2_output_0[i][j][k] = BN254::from((input._features_features_40_Constant_2_output_0[i][j][k]) as u64); 
				} else {
					assignment._features_features_40_Constant_2_output_0[i][j][k] = -BN254::from((-input._features_features_40_Constant_2_output_0[i][j][k]) as u64); 
				} 
			}
		}
	}
	assignment._features_features_42_relu_Relu_output_0 = vec![vec![vec![vec![BN254::default();2];2];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..2 {
				for l in 0..2 {
					if input._features_features_42_relu_Relu_output_0[i][j][k][l] >= 0{
						assignment._features_features_42_relu_Relu_output_0[i][j][k][l] = BN254::from((input._features_features_42_relu_Relu_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_42_relu_Relu_output_0[i][j][k][l] = -BN254::from((-input._features_features_42_relu_Relu_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._features_features_43_maxpool_MaxPool_output_0 = vec![vec![vec![vec![BN254::default();1];1];512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			for k in 0..1 {
				for l in 0..1 {
					if input._features_features_43_maxpool_MaxPool_output_0[i][j][k][l] >= 0{
						assignment._features_features_43_maxpool_MaxPool_output_0[i][j][k][l] = BN254::from((input._features_features_43_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} else {
						assignment._features_features_43_maxpool_MaxPool_output_0[i][j][k][l] = -BN254::from((-input._features_features_43_maxpool_MaxPool_output_0[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment._classifier_classifier_0_linear_MatMul_output_0 = vec![vec![BN254::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if input._classifier_classifier_0_linear_MatMul_output_0[i][j] >= 0{
				assignment._classifier_classifier_0_linear_MatMul_output_0[i][j] = BN254::from((input._classifier_classifier_0_linear_MatMul_output_0[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_0_linear_MatMul_output_0[i][j] = -BN254::from((-input._classifier_classifier_0_linear_MatMul_output_0[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_0_Constant_output_0 = BN254::default(); 
	if input._classifier_classifier_0_Constant_output_0 >= 0{
		assignment._classifier_classifier_0_Constant_output_0 = BN254::from((input._classifier_classifier_0_Constant_output_0) as u64); 
	} else {
		assignment._classifier_classifier_0_Constant_output_0 = -BN254::from((-input._classifier_classifier_0_Constant_output_0) as u64); 
	} 
	assignment._classifier_classifier_0_Constant_1_output_0 = BN254::default(); 
	if input._classifier_classifier_0_Constant_1_output_0 >= 0{
		assignment._classifier_classifier_0_Constant_1_output_0 = BN254::from((input._classifier_classifier_0_Constant_1_output_0) as u64); 
	} else {
		assignment._classifier_classifier_0_Constant_1_output_0 = -BN254::from((-input._classifier_classifier_0_Constant_1_output_0) as u64); 
	} 
	assignment._classifier_classifier_0_Div_output_0_r = vec![vec![BN254::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if input._classifier_classifier_0_Div_output_0_r[i][j] >= 0{
				assignment._classifier_classifier_0_Div_output_0_r[i][j] = BN254::from((input._classifier_classifier_0_Div_output_0_r[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_0_Div_output_0_r[i][j] = -BN254::from((-input._classifier_classifier_0_Div_output_0_r[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_0_Div_output_0 = vec![vec![BN254::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if input._classifier_classifier_0_Div_output_0[i][j] >= 0{
				assignment._classifier_classifier_0_Div_output_0[i][j] = BN254::from((input._classifier_classifier_0_Div_output_0[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_0_Div_output_0[i][j] = -BN254::from((-input._classifier_classifier_0_Div_output_0[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_0_Constant_2_output_0 = vec![BN254::default();512]; 
	for i in 0..512 {
		if input._classifier_classifier_0_Constant_2_output_0[i] >= 0{
			assignment._classifier_classifier_0_Constant_2_output_0[i] = BN254::from((input._classifier_classifier_0_Constant_2_output_0[i]) as u64); 
		} else {
			assignment._classifier_classifier_0_Constant_2_output_0[i] = -BN254::from((-input._classifier_classifier_0_Constant_2_output_0[i]) as u64); 
		} 
	}
	assignment._classifier_classifier_1_relu_Relu_output_0 = vec![vec![BN254::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if input._classifier_classifier_1_relu_Relu_output_0[i][j] >= 0{
				assignment._classifier_classifier_1_relu_Relu_output_0[i][j] = BN254::from((input._classifier_classifier_1_relu_Relu_output_0[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_1_relu_Relu_output_0[i][j] = -BN254::from((-input._classifier_classifier_1_relu_Relu_output_0[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_3_linear_MatMul_output_0 = vec![vec![BN254::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if input._classifier_classifier_3_linear_MatMul_output_0[i][j] >= 0{
				assignment._classifier_classifier_3_linear_MatMul_output_0[i][j] = BN254::from((input._classifier_classifier_3_linear_MatMul_output_0[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_3_linear_MatMul_output_0[i][j] = -BN254::from((-input._classifier_classifier_3_linear_MatMul_output_0[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_3_Constant_output_0 = BN254::default(); 
	if input._classifier_classifier_3_Constant_output_0 >= 0{
		assignment._classifier_classifier_3_Constant_output_0 = BN254::from((input._classifier_classifier_3_Constant_output_0) as u64); 
	} else {
		assignment._classifier_classifier_3_Constant_output_0 = -BN254::from((-input._classifier_classifier_3_Constant_output_0) as u64); 
	} 
	assignment._classifier_classifier_3_Constant_1_output_0 = BN254::default(); 
	if input._classifier_classifier_3_Constant_1_output_0 >= 0{
		assignment._classifier_classifier_3_Constant_1_output_0 = BN254::from((input._classifier_classifier_3_Constant_1_output_0) as u64); 
	} else {
		assignment._classifier_classifier_3_Constant_1_output_0 = -BN254::from((-input._classifier_classifier_3_Constant_1_output_0) as u64); 
	} 
	assignment._classifier_classifier_3_Div_output_0_r = vec![vec![BN254::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if input._classifier_classifier_3_Div_output_0_r[i][j] >= 0{
				assignment._classifier_classifier_3_Div_output_0_r[i][j] = BN254::from((input._classifier_classifier_3_Div_output_0_r[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_3_Div_output_0_r[i][j] = -BN254::from((-input._classifier_classifier_3_Div_output_0_r[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_3_Div_output_0 = vec![vec![BN254::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if input._classifier_classifier_3_Div_output_0[i][j] >= 0{
				assignment._classifier_classifier_3_Div_output_0[i][j] = BN254::from((input._classifier_classifier_3_Div_output_0[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_3_Div_output_0[i][j] = -BN254::from((-input._classifier_classifier_3_Div_output_0[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_3_Constant_2_output_0 = vec![BN254::default();512]; 
	for i in 0..512 {
		if input._classifier_classifier_3_Constant_2_output_0[i] >= 0{
			assignment._classifier_classifier_3_Constant_2_output_0[i] = BN254::from((input._classifier_classifier_3_Constant_2_output_0[i]) as u64); 
		} else {
			assignment._classifier_classifier_3_Constant_2_output_0[i] = -BN254::from((-input._classifier_classifier_3_Constant_2_output_0[i]) as u64); 
		} 
	}
	assignment._classifier_classifier_4_relu_Relu_output_0 = vec![vec![BN254::default();512];16]; 
	for i in 0..16 {
		for j in 0..512 {
			if input._classifier_classifier_4_relu_Relu_output_0[i][j] >= 0{
				assignment._classifier_classifier_4_relu_Relu_output_0[i][j] = BN254::from((input._classifier_classifier_4_relu_Relu_output_0[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_4_relu_Relu_output_0[i][j] = -BN254::from((-input._classifier_classifier_4_relu_Relu_output_0[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_6_linear_MatMul_output_0 = vec![vec![BN254::default();10];16]; 
	for i in 0..16 {
		for j in 0..10 {
			if input._classifier_classifier_6_linear_MatMul_output_0[i][j] >= 0{
				assignment._classifier_classifier_6_linear_MatMul_output_0[i][j] = BN254::from((input._classifier_classifier_6_linear_MatMul_output_0[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_6_linear_MatMul_output_0[i][j] = -BN254::from((-input._classifier_classifier_6_linear_MatMul_output_0[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_6_Constant_output_0 = BN254::default(); 
	if input._classifier_classifier_6_Constant_output_0 >= 0{
		assignment._classifier_classifier_6_Constant_output_0 = BN254::from((input._classifier_classifier_6_Constant_output_0) as u64); 
	} else {
		assignment._classifier_classifier_6_Constant_output_0 = -BN254::from((-input._classifier_classifier_6_Constant_output_0) as u64); 
	} 
	assignment._classifier_classifier_6_Constant_1_output_0 = BN254::default(); 
	if input._classifier_classifier_6_Constant_1_output_0 >= 0{
		assignment._classifier_classifier_6_Constant_1_output_0 = BN254::from((input._classifier_classifier_6_Constant_1_output_0) as u64); 
	} else {
		assignment._classifier_classifier_6_Constant_1_output_0 = -BN254::from((-input._classifier_classifier_6_Constant_1_output_0) as u64); 
	} 
	assignment._classifier_classifier_6_Div_output_0_r = vec![vec![BN254::default();10];16]; 
	for i in 0..16 {
		for j in 0..10 {
			if input._classifier_classifier_6_Div_output_0_r[i][j] >= 0{
				assignment._classifier_classifier_6_Div_output_0_r[i][j] = BN254::from((input._classifier_classifier_6_Div_output_0_r[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_6_Div_output_0_r[i][j] = -BN254::from((-input._classifier_classifier_6_Div_output_0_r[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_6_Div_output_0 = vec![vec![BN254::default();10];16]; 
	for i in 0..16 {
		for j in 0..10 {
			if input._classifier_classifier_6_Div_output_0[i][j] >= 0{
				assignment._classifier_classifier_6_Div_output_0[i][j] = BN254::from((input._classifier_classifier_6_Div_output_0[i][j]) as u64); 
			} else {
				assignment._classifier_classifier_6_Div_output_0[i][j] = -BN254::from((-input._classifier_classifier_6_Div_output_0[i][j]) as u64); 
			} 
		}
	}
	assignment._classifier_classifier_6_Constant_2_output_0 = vec![BN254::default();10]; 
	for i in 0..10 {
		if input._classifier_classifier_6_Constant_2_output_0[i] >= 0{
			assignment._classifier_classifier_6_Constant_2_output_0[i] = BN254::from((input._classifier_classifier_6_Constant_2_output_0[i]) as u64); 
		} else {
			assignment._classifier_classifier_6_Constant_2_output_0[i] = -BN254::from((-input._classifier_classifier_6_Constant_2_output_0[i]) as u64); 
		} 
	}
	assignment.features_0_conv_weight = vec![vec![vec![vec![BN254::default();3];3];3];64]; 
	for i in 0..64 {
		for j in 0..3 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_0_conv_weight[i][j][k][l] >= 0{
						assignment.features_0_conv_weight[i][j][k][l] = BN254::from((input.features_0_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_0_conv_weight[i][j][k][l] = -BN254::from((-input.features_0_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_3_conv_weight = vec![vec![vec![vec![BN254::default();3];3];64];64]; 
	for i in 0..64 {
		for j in 0..64 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_3_conv_weight[i][j][k][l] >= 0{
						assignment.features_3_conv_weight[i][j][k][l] = BN254::from((input.features_3_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_3_conv_weight[i][j][k][l] = -BN254::from((-input.features_3_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_7_conv_weight = vec![vec![vec![vec![BN254::default();3];3];64];128]; 
	for i in 0..128 {
		for j in 0..64 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_7_conv_weight[i][j][k][l] >= 0{
						assignment.features_7_conv_weight[i][j][k][l] = BN254::from((input.features_7_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_7_conv_weight[i][j][k][l] = -BN254::from((-input.features_7_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_10_conv_weight = vec![vec![vec![vec![BN254::default();3];3];128];128]; 
	for i in 0..128 {
		for j in 0..128 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_10_conv_weight[i][j][k][l] >= 0{
						assignment.features_10_conv_weight[i][j][k][l] = BN254::from((input.features_10_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_10_conv_weight[i][j][k][l] = -BN254::from((-input.features_10_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_14_conv_weight = vec![vec![vec![vec![BN254::default();3];3];128];256]; 
	for i in 0..256 {
		for j in 0..128 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_14_conv_weight[i][j][k][l] >= 0{
						assignment.features_14_conv_weight[i][j][k][l] = BN254::from((input.features_14_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_14_conv_weight[i][j][k][l] = -BN254::from((-input.features_14_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_17_conv_weight = vec![vec![vec![vec![BN254::default();3];3];256];256]; 
	for i in 0..256 {
		for j in 0..256 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_17_conv_weight[i][j][k][l] >= 0{
						assignment.features_17_conv_weight[i][j][k][l] = BN254::from((input.features_17_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_17_conv_weight[i][j][k][l] = -BN254::from((-input.features_17_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_20_conv_weight = vec![vec![vec![vec![BN254::default();3];3];256];256]; 
	for i in 0..256 {
		for j in 0..256 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_20_conv_weight[i][j][k][l] >= 0{
						assignment.features_20_conv_weight[i][j][k][l] = BN254::from((input.features_20_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_20_conv_weight[i][j][k][l] = -BN254::from((-input.features_20_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_24_conv_weight = vec![vec![vec![vec![BN254::default();3];3];256];512]; 
	for i in 0..512 {
		for j in 0..256 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_24_conv_weight[i][j][k][l] >= 0{
						assignment.features_24_conv_weight[i][j][k][l] = BN254::from((input.features_24_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_24_conv_weight[i][j][k][l] = -BN254::from((-input.features_24_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_27_conv_weight = vec![vec![vec![vec![BN254::default();3];3];512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_27_conv_weight[i][j][k][l] >= 0{
						assignment.features_27_conv_weight[i][j][k][l] = BN254::from((input.features_27_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_27_conv_weight[i][j][k][l] = -BN254::from((-input.features_27_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_30_conv_weight = vec![vec![vec![vec![BN254::default();3];3];512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_30_conv_weight[i][j][k][l] >= 0{
						assignment.features_30_conv_weight[i][j][k][l] = BN254::from((input.features_30_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_30_conv_weight[i][j][k][l] = -BN254::from((-input.features_30_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_34_conv_weight = vec![vec![vec![vec![BN254::default();3];3];512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_34_conv_weight[i][j][k][l] >= 0{
						assignment.features_34_conv_weight[i][j][k][l] = BN254::from((input.features_34_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_34_conv_weight[i][j][k][l] = -BN254::from((-input.features_34_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_37_conv_weight = vec![vec![vec![vec![BN254::default();3];3];512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_37_conv_weight[i][j][k][l] >= 0{
						assignment.features_37_conv_weight[i][j][k][l] = BN254::from((input.features_37_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_37_conv_weight[i][j][k][l] = -BN254::from((-input.features_37_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.features_40_conv_weight = vec![vec![vec![vec![BN254::default();3];3];512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			for k in 0..3 {
				for l in 0..3 {
					if input.features_40_conv_weight[i][j][k][l] >= 0{
						assignment.features_40_conv_weight[i][j][k][l] = BN254::from((input.features_40_conv_weight[i][j][k][l]) as u64); 
					} else {
						assignment.features_40_conv_weight[i][j][k][l] = -BN254::from((-input.features_40_conv_weight[i][j][k][l]) as u64); 
					} 
				}
			}
		}
	}
	assignment.onnx__MatMul_215 = vec![vec![BN254::default();512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			if input.onnx__MatMul_215[i][j] >= 0{
				assignment.onnx__MatMul_215[i][j] = BN254::from((input.onnx__MatMul_215[i][j]) as u64); 
			} else {
				assignment.onnx__MatMul_215[i][j] = -BN254::from((-input.onnx__MatMul_215[i][j]) as u64); 
			} 
		}
	}
	assignment.onnx__MatMul_216 = vec![vec![BN254::default();512];512]; 
	for i in 0..512 {
		for j in 0..512 {
			if input.onnx__MatMul_216[i][j] >= 0{
				assignment.onnx__MatMul_216[i][j] = BN254::from((input.onnx__MatMul_216[i][j]) as u64); 
			} else {
				assignment.onnx__MatMul_216[i][j] = -BN254::from((-input.onnx__MatMul_216[i][j]) as u64); 
			} 
		}
	}
	assignment.onnx__MatMul_217 = vec![vec![BN254::default();10];512]; 
	for i in 0..512 {
		for j in 0..10 {
			if input.onnx__MatMul_217[i][j] >= 0{
				assignment.onnx__MatMul_217[i][j] = BN254::from((input.onnx__MatMul_217[i][j]) as u64); 
			} else {
				assignment.onnx__MatMul_217[i][j] = -BN254::from((-input.onnx__MatMul_217[i][j]) as u64); 
			} 
		}
	}
}

#[test]
fn expander_witness() -> std::io::Result<()>{ 
	let compile_result = stacker::grow(12 * 1024 * 1024 * 1024, ||
		{
			let mut hint_registry = HintRegistry::<BN254>::new();
			hint_registry.register("myhint.querycounthint", query_count_hint);
			hint_registry.register("myhint.rangeproofhint", rangeproof_hint);
			// let file = std::fs::File::open("circuit.txt").unwrap();
			// let reader = std::io::BufReader::new(file);
			// let layered_circuit = expander_compiler::circuit::layered::Circuit::<BN254Config, NormalInputType>::deserialize_from(reader).unwrap();
			// let file = std::fs::File::open("witness_solver.txt").unwrap();
			// let reader = std::io::BufReader::new(file);
			// let witness_solver = expander_compiler::circuit::ir::hint_normalized::witness_solver::WitnessSolver::<BN254Config>::deserialize_from(reader).unwrap();
			let input_str = fs::read_to_string("input.json").unwrap();
			let input: Circuit_Input = serde_json::from_str(&input_str).unwrap();
			let mut assignment = Circuit::<BN254>::default();
			input_copy(&input, &mut assignment);
			//let witness = witness_solver.solve_witness_with_hints(&assignment, &mut hint_registry).unwrap();
			debug_eval(&Circuit::<BN254>::default(), &assignment, hint_registry);
			// println!("Check result:");
			// let res = layered_circuit.run(&witness);
			// println!("{:?}", res);
			// let file = std::fs::File::create("witness.txt").unwrap();
			// let writer = std::io::BufWriter::new(file);
			// witness.serialize_into(writer).unwrap();
		}
	);
	Ok(())
}
