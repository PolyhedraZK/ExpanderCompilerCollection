use expander_compiler::frontend::*;
use expander_compiler::zkcuda::{context::*, kernel::*};
use expander_compiler::zkcuda::shape::{Reshape, Transpose};
use serdes::ExpSerde;

#[allow(dead_code)]
struct Circuit {
	output: Vec<Vec<BN254Fr>>, 
	input: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_0_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_2_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_3_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_5_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_6_maxpool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_7_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_7_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_7_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_7_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_9_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_10_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_10_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_10_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_10_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_12_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_13_maxpool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_14_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_14_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_14_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_14_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_16_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_17_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_17_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_17_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_17_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_19_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_20_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_20_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_20_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_20_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_22_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_23_maxpool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_24_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_24_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_24_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_24_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_26_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_27_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_27_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_27_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_27_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_29_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_30_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_30_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_30_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_30_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_32_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_33_maxpool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_34_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_34_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_34_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_34_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_36_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_37_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_37_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_37_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_37_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_39_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_40_conv_output_0_conv: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_40_conv_output_0_div: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_40_conv_output_0_rem: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_40_conv_output_0_floor: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_42_relu_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_features_features_43_maxpool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_avgpool_GlobalAveragePool_output_0: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	_classifier_classifier_0_gemm_output_0_matmul: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_0_gemm_output_0_div: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_0_gemm_output_0_rem: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_0_gemm_output_0_floor: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_1_relu_output_0: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_3_gemm_output_0_matmul: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_3_gemm_output_0_div: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_3_gemm_output_0_rem: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_3_gemm_output_0_floor: Vec<Vec<BN254Fr>>, 
	_classifier_classifier_4_relu_output_0: Vec<Vec<BN254Fr>>, 
	output_matmul: Vec<Vec<BN254Fr>>, 
	output_div: Vec<Vec<BN254Fr>>, 
	output_rem: Vec<Vec<BN254Fr>>, 
	output_floor: Vec<Vec<BN254Fr>>, 
	onnx_conv_150: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_151: Vec<BN254Fr>, 
	onnx_conv_151_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_150_nscale: BN254Fr, 
	onnx_conv_150_dscale: BN254Fr, 
	onnx_conv_153: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_154: Vec<BN254Fr>, 
	onnx_conv_154_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_153_nscale: BN254Fr, 
	onnx_conv_153_dscale: BN254Fr, 
	onnx_conv_156: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_157: Vec<BN254Fr>, 
	onnx_conv_157_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_156_nscale: BN254Fr, 
	onnx_conv_156_dscale: BN254Fr, 
	onnx_conv_159: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_160: Vec<BN254Fr>, 
	onnx_conv_160_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_159_nscale: BN254Fr, 
	onnx_conv_159_dscale: BN254Fr, 
	onnx_conv_162: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_163: Vec<BN254Fr>, 
	onnx_conv_163_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_162_nscale: BN254Fr, 
	onnx_conv_162_dscale: BN254Fr, 
	onnx_conv_165: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_166: Vec<BN254Fr>, 
	onnx_conv_166_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_165_nscale: BN254Fr, 
	onnx_conv_165_dscale: BN254Fr, 
	onnx_conv_168: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_169: Vec<BN254Fr>, 
	onnx_conv_169_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_168_nscale: BN254Fr, 
	onnx_conv_168_dscale: BN254Fr, 
	onnx_conv_171: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_172: Vec<BN254Fr>, 
	onnx_conv_172_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_171_nscale: BN254Fr, 
	onnx_conv_171_dscale: BN254Fr, 
	onnx_conv_174: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_175: Vec<BN254Fr>, 
	onnx_conv_175_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_174_nscale: BN254Fr, 
	onnx_conv_174_dscale: BN254Fr, 
	onnx_conv_177: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_178: Vec<BN254Fr>, 
	onnx_conv_178_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_177_nscale: BN254Fr, 
	onnx_conv_177_dscale: BN254Fr, 
	onnx_conv_180: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_181: Vec<BN254Fr>, 
	onnx_conv_181_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_180_nscale: BN254Fr, 
	onnx_conv_180_dscale: BN254Fr, 
	onnx_conv_183: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_184: Vec<BN254Fr>, 
	onnx_conv_184_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_183_nscale: BN254Fr, 
	onnx_conv_183_dscale: BN254Fr, 
	onnx_conv_186: Vec<Vec<Vec<Vec<BN254Fr>>>>, 
	onnx_conv_187: Vec<BN254Fr>, 
	onnx_conv_187_q: Vec<Vec<Vec<BN254Fr>>>, 
	onnx_conv_186_nscale: BN254Fr, 
	onnx_conv_186_dscale: BN254Fr, 
	classifier_0_weight: Vec<Vec<BN254Fr>>, 
	classifier_0_bias_q: Vec<BN254Fr>, 
	classifier_0_weight_nscale: BN254Fr, 
	classifier_0_weight_dscale: BN254Fr, 
	classifier_3_weight: Vec<Vec<BN254Fr>>, 
	classifier_3_bias_q: Vec<BN254Fr>, 
	classifier_3_weight_nscale: BN254Fr, 
	classifier_3_weight_dscale: BN254Fr, 
	classifier_6_weight: Vec<Vec<BN254Fr>>, 
	classifier_6_bias_q: Vec<BN254Fr>, 
	classifier_6_weight_nscale: BN254Fr, 
	classifier_6_weight_dscale: BN254Fr, 
	input_mat_ru: Vec<BN254Fr>, 
	onnx_conv_150_mat_rv: Vec<BN254Fr>, 
	_features_features_2_relu_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_153_mat_rv: Vec<BN254Fr>, 
	_features_features_6_maxpool_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_156_mat_rv: Vec<BN254Fr>, 
	_features_features_9_relu_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_159_mat_rv: Vec<BN254Fr>, 
	_features_features_13_maxpool_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_162_mat_rv: Vec<BN254Fr>, 
	_features_features_16_relu_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_165_mat_rv: Vec<BN254Fr>, 
	_features_features_19_relu_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_168_mat_rv: Vec<BN254Fr>, 
	_features_features_23_maxpool_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_171_mat_rv: Vec<BN254Fr>, 
	_features_features_26_relu_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_174_mat_rv: Vec<BN254Fr>, 
	_features_features_29_relu_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_177_mat_rv: Vec<BN254Fr>, 
	_features_features_33_maxpool_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_180_mat_rv: Vec<BN254Fr>, 
	_features_features_36_relu_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_183_mat_rv: Vec<BN254Fr>, 
	_features_features_39_relu_output_0_mat_ru: Vec<BN254Fr>, 
	onnx_conv_186_mat_rv: Vec<BN254Fr>, 
	_Flatten_output_0_mat_ru: Vec<BN254Fr>, 
	classifier_0_weight_mat_rv: Vec<BN254Fr>, 
	_classifier_classifier_1_relu_output_0_mat_ru: Vec<BN254Fr>, 
	classifier_3_weight_mat_rv: Vec<BN254Fr>, 
	_classifier_classifier_4_relu_output_0_mat_ru: Vec<BN254Fr>, 
	classifier_6_weight_mat_rv: Vec<BN254Fr>, 
}

fn default_variable() -> Circuit{
	let output = vec![vec![BN254Fr::default();10];1]; 
	let input = vec![vec![vec![vec![BN254Fr::default();32];32];3];1]; 
	let _features_features_0_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();32];32];64];1]; 
	let _features_features_0_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();32];32];64];1]; 
	let _features_features_0_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();32];32];64];1]; 
	let _features_features_0_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();32];32];64];1]; 
	let _features_features_2_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();32];32];64];1]; 
	let _features_features_3_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();32];32];64];1]; 
	let _features_features_3_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();32];32];64];1]; 
	let _features_features_3_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();32];32];64];1]; 
	let _features_features_3_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();32];32];64];1]; 
	let _features_features_5_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();32];32];64];1]; 
	let _features_features_6_maxpool_output_0 = vec![vec![vec![vec![BN254Fr::default();16];16];64];1]; 
	let _features_features_7_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();16];16];128];1]; 
	let _features_features_7_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();16];16];128];1]; 
	let _features_features_7_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();16];16];128];1]; 
	let _features_features_7_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();16];16];128];1]; 
	let _features_features_9_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();16];16];128];1]; 
	let _features_features_10_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();16];16];128];1]; 
	let _features_features_10_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();16];16];128];1]; 
	let _features_features_10_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();16];16];128];1]; 
	let _features_features_10_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();16];16];128];1]; 
	let _features_features_12_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();16];16];128];1]; 
	let _features_features_13_maxpool_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];128];1]; 
	let _features_features_14_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_14_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_14_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_14_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_16_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_17_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_17_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_17_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_17_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_19_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_20_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_20_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_20_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_20_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_22_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();8];8];256];1]; 
	let _features_features_23_maxpool_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];256];1]; 
	let _features_features_24_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_24_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_24_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_24_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_26_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_27_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_27_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_27_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_27_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_29_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_30_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_30_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_30_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_30_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_32_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();4];4];512];1]; 
	let _features_features_33_maxpool_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_34_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_34_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_34_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_34_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_36_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_37_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_37_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_37_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_37_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_39_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_40_conv_output_0_conv = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_40_conv_output_0_div = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_40_conv_output_0_rem = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_40_conv_output_0_floor = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_42_relu_output_0 = vec![vec![vec![vec![BN254Fr::default();2];2];512];1]; 
	let _features_features_43_maxpool_output_0 = vec![vec![vec![vec![BN254Fr::default();1];1];512];1]; 
	let _avgpool_GlobalAveragePool_output_0 = vec![vec![vec![vec![BN254Fr::default();1];1];512];1]; 
	let _classifier_classifier_0_gemm_output_0_matmul = vec![vec![BN254Fr::default();512];1]; 
	let _classifier_classifier_0_gemm_output_0_div = vec![vec![BN254Fr::default();512];1]; 
	let _classifier_classifier_0_gemm_output_0_rem = vec![vec![BN254Fr::default();512];1]; 
	let _classifier_classifier_0_gemm_output_0_floor = vec![vec![BN254Fr::default();512];1]; 
	let _classifier_classifier_1_relu_output_0 = vec![vec![BN254Fr::default();512];1]; 
	let _classifier_classifier_3_gemm_output_0_matmul = vec![vec![BN254Fr::default();512];1]; 
	let _classifier_classifier_3_gemm_output_0_div = vec![vec![BN254Fr::default();512];1]; 
	let _classifier_classifier_3_gemm_output_0_rem = vec![vec![BN254Fr::default();512];1]; 
	let _classifier_classifier_3_gemm_output_0_floor = vec![vec![BN254Fr::default();512];1]; 
	let _classifier_classifier_4_relu_output_0 = vec![vec![BN254Fr::default();512];1]; 
	let output_matmul = vec![vec![BN254Fr::default();10];1]; 
	let output_div = vec![vec![BN254Fr::default();10];1]; 
	let output_rem = vec![vec![BN254Fr::default();10];1]; 
	let output_floor = vec![vec![BN254Fr::default();10];1]; 
	let onnx_conv_150 = vec![vec![vec![vec![BN254Fr::default();3];3];3];64]; 
	let onnx_conv_151 = vec![BN254Fr::default();64]; 
	let onnx_conv_151_q = vec![vec![vec![BN254Fr::default();1];1];64]; 
	let onnx_conv_150_nscale = BN254Fr::default(); 
	let onnx_conv_150_dscale = BN254Fr::default(); 
	let onnx_conv_153 = vec![vec![vec![vec![BN254Fr::default();3];3];64];64]; 
	let onnx_conv_154 = vec![BN254Fr::default();64]; 
	let onnx_conv_154_q = vec![vec![vec![BN254Fr::default();1];1];64]; 
	let onnx_conv_153_nscale = BN254Fr::default(); 
	let onnx_conv_153_dscale = BN254Fr::default(); 
	let onnx_conv_156 = vec![vec![vec![vec![BN254Fr::default();3];3];64];128]; 
	let onnx_conv_157 = vec![BN254Fr::default();128]; 
	let onnx_conv_157_q = vec![vec![vec![BN254Fr::default();1];1];128]; 
	let onnx_conv_156_nscale = BN254Fr::default(); 
	let onnx_conv_156_dscale = BN254Fr::default(); 
	let onnx_conv_159 = vec![vec![vec![vec![BN254Fr::default();3];3];128];128]; 
	let onnx_conv_160 = vec![BN254Fr::default();128]; 
	let onnx_conv_160_q = vec![vec![vec![BN254Fr::default();1];1];128]; 
	let onnx_conv_159_nscale = BN254Fr::default(); 
	let onnx_conv_159_dscale = BN254Fr::default(); 
	let onnx_conv_162 = vec![vec![vec![vec![BN254Fr::default();3];3];128];256]; 
	let onnx_conv_163 = vec![BN254Fr::default();256]; 
	let onnx_conv_163_q = vec![vec![vec![BN254Fr::default();1];1];256]; 
	let onnx_conv_162_nscale = BN254Fr::default(); 
	let onnx_conv_162_dscale = BN254Fr::default(); 
	let onnx_conv_165 = vec![vec![vec![vec![BN254Fr::default();3];3];256];256]; 
	let onnx_conv_166 = vec![BN254Fr::default();256]; 
	let onnx_conv_166_q = vec![vec![vec![BN254Fr::default();1];1];256]; 
	let onnx_conv_165_nscale = BN254Fr::default(); 
	let onnx_conv_165_dscale = BN254Fr::default(); 
	let onnx_conv_168 = vec![vec![vec![vec![BN254Fr::default();3];3];256];256]; 
	let onnx_conv_169 = vec![BN254Fr::default();256]; 
	let onnx_conv_169_q = vec![vec![vec![BN254Fr::default();1];1];256]; 
	let onnx_conv_168_nscale = BN254Fr::default(); 
	let onnx_conv_168_dscale = BN254Fr::default(); 
	let onnx_conv_171 = vec![vec![vec![vec![BN254Fr::default();3];3];256];512]; 
	let onnx_conv_172 = vec![BN254Fr::default();512]; 
	let onnx_conv_172_q = vec![vec![vec![BN254Fr::default();1];1];512]; 
	let onnx_conv_171_nscale = BN254Fr::default(); 
	let onnx_conv_171_dscale = BN254Fr::default(); 
	let onnx_conv_174 = vec![vec![vec![vec![BN254Fr::default();3];3];512];512]; 
	let onnx_conv_175 = vec![BN254Fr::default();512]; 
	let onnx_conv_175_q = vec![vec![vec![BN254Fr::default();1];1];512]; 
	let onnx_conv_174_nscale = BN254Fr::default(); 
	let onnx_conv_174_dscale = BN254Fr::default(); 
	let onnx_conv_177 = vec![vec![vec![vec![BN254Fr::default();3];3];512];512]; 
	let onnx_conv_178 = vec![BN254Fr::default();512]; 
	let onnx_conv_178_q = vec![vec![vec![BN254Fr::default();1];1];512]; 
	let onnx_conv_177_nscale = BN254Fr::default(); 
	let onnx_conv_177_dscale = BN254Fr::default(); 
	let onnx_conv_180 = vec![vec![vec![vec![BN254Fr::default();3];3];512];512]; 
	let onnx_conv_181 = vec![BN254Fr::default();512]; 
	let onnx_conv_181_q = vec![vec![vec![BN254Fr::default();1];1];512]; 
	let onnx_conv_180_nscale = BN254Fr::default(); 
	let onnx_conv_180_dscale = BN254Fr::default(); 
	let onnx_conv_183 = vec![vec![vec![vec![BN254Fr::default();3];3];512];512]; 
	let onnx_conv_184 = vec![BN254Fr::default();512]; 
	let onnx_conv_184_q = vec![vec![vec![BN254Fr::default();1];1];512]; 
	let onnx_conv_183_nscale = BN254Fr::default(); 
	let onnx_conv_183_dscale = BN254Fr::default(); 
	let onnx_conv_186 = vec![vec![vec![vec![BN254Fr::default();3];3];512];512]; 
	let onnx_conv_187 = vec![BN254Fr::default();512]; 
	let onnx_conv_187_q = vec![vec![vec![BN254Fr::default();1];1];512]; 
	let onnx_conv_186_nscale = BN254Fr::default(); 
	let onnx_conv_186_dscale = BN254Fr::default(); 
	let classifier_0_weight = vec![vec![BN254Fr::default();512];512]; 
	let classifier_0_bias_q = vec![BN254Fr::default();512]; 
	let classifier_0_weight_nscale = BN254Fr::default(); 
	let classifier_0_weight_dscale = BN254Fr::default(); 
	let classifier_3_weight = vec![vec![BN254Fr::default();512];512]; 
	let classifier_3_bias_q = vec![BN254Fr::default();512]; 
	let classifier_3_weight_nscale = BN254Fr::default(); 
	let classifier_3_weight_dscale = BN254Fr::default(); 
	let classifier_6_weight = vec![vec![BN254Fr::default();10];512]; 
	let classifier_6_bias_q = vec![BN254Fr::default();10]; 
	let classifier_6_weight_nscale = BN254Fr::default(); 
	let classifier_6_weight_dscale = BN254Fr::default(); 
	let input_mat_ru = vec![BN254Fr::default();1024]; 
	let onnx_conv_150_mat_rv = vec![BN254Fr::default();64]; 
	let _features_features_2_relu_output_0_mat_ru = vec![BN254Fr::default();1024]; 
	let onnx_conv_153_mat_rv = vec![BN254Fr::default();64]; 
	let _features_features_6_maxpool_output_0_mat_ru = vec![BN254Fr::default();256]; 
	let onnx_conv_156_mat_rv = vec![BN254Fr::default();128]; 
	let _features_features_9_relu_output_0_mat_ru = vec![BN254Fr::default();256]; 
	let onnx_conv_159_mat_rv = vec![BN254Fr::default();128]; 
	let _features_features_13_maxpool_output_0_mat_ru = vec![BN254Fr::default();64]; 
	let onnx_conv_162_mat_rv = vec![BN254Fr::default();256]; 
	let _features_features_16_relu_output_0_mat_ru = vec![BN254Fr::default();64]; 
	let onnx_conv_165_mat_rv = vec![BN254Fr::default();256]; 
	let _features_features_19_relu_output_0_mat_ru = vec![BN254Fr::default();64]; 
	let onnx_conv_168_mat_rv = vec![BN254Fr::default();256]; 
	let _features_features_23_maxpool_output_0_mat_ru = vec![BN254Fr::default();16]; 
	let onnx_conv_171_mat_rv = vec![BN254Fr::default();512]; 
	let _features_features_26_relu_output_0_mat_ru = vec![BN254Fr::default();16]; 
	let onnx_conv_174_mat_rv = vec![BN254Fr::default();512]; 
	let _features_features_29_relu_output_0_mat_ru = vec![BN254Fr::default();16]; 
	let onnx_conv_177_mat_rv = vec![BN254Fr::default();512]; 
	let _features_features_33_maxpool_output_0_mat_ru = vec![BN254Fr::default();4]; 
	let onnx_conv_180_mat_rv = vec![BN254Fr::default();512]; 
	let _features_features_36_relu_output_0_mat_ru = vec![BN254Fr::default();4]; 
	let onnx_conv_183_mat_rv = vec![BN254Fr::default();512]; 
	let _features_features_39_relu_output_0_mat_ru = vec![BN254Fr::default();4]; 
	let onnx_conv_186_mat_rv = vec![BN254Fr::default();512]; 
	let _Flatten_output_0_mat_ru = vec![BN254Fr::default();1]; 
	let classifier_0_weight_mat_rv = vec![BN254Fr::default();512]; 
	let _classifier_classifier_1_relu_output_0_mat_ru = vec![BN254Fr::default();1]; 
	let classifier_3_weight_mat_rv = vec![BN254Fr::default();512]; 
	let _classifier_classifier_4_relu_output_0_mat_ru = vec![BN254Fr::default();1]; 
	let classifier_6_weight_mat_rv = vec![BN254Fr::default();10]; 
	let ass = Circuit{output,input,_features_features_0_conv_output_0_conv,_features_features_0_conv_output_0_div,_features_features_0_conv_output_0_rem,_features_features_0_conv_output_0_floor,_features_features_2_relu_output_0,_features_features_3_conv_output_0_conv,_features_features_3_conv_output_0_div,_features_features_3_conv_output_0_rem,_features_features_3_conv_output_0_floor,_features_features_5_relu_output_0,_features_features_6_maxpool_output_0,_features_features_7_conv_output_0_conv,_features_features_7_conv_output_0_div,_features_features_7_conv_output_0_rem,_features_features_7_conv_output_0_floor,_features_features_9_relu_output_0,_features_features_10_conv_output_0_conv,_features_features_10_conv_output_0_div,_features_features_10_conv_output_0_rem,_features_features_10_conv_output_0_floor,_features_features_12_relu_output_0,_features_features_13_maxpool_output_0,_features_features_14_conv_output_0_conv,_features_features_14_conv_output_0_div,_features_features_14_conv_output_0_rem,_features_features_14_conv_output_0_floor,_features_features_16_relu_output_0,_features_features_17_conv_output_0_conv,_features_features_17_conv_output_0_div,_features_features_17_conv_output_0_rem,_features_features_17_conv_output_0_floor,_features_features_19_relu_output_0,_features_features_20_conv_output_0_conv,_features_features_20_conv_output_0_div,_features_features_20_conv_output_0_rem,_features_features_20_conv_output_0_floor,_features_features_22_relu_output_0,_features_features_23_maxpool_output_0,_features_features_24_conv_output_0_conv,_features_features_24_conv_output_0_div,_features_features_24_conv_output_0_rem,_features_features_24_conv_output_0_floor,_features_features_26_relu_output_0,_features_features_27_conv_output_0_conv,_features_features_27_conv_output_0_div,_features_features_27_conv_output_0_rem,_features_features_27_conv_output_0_floor,_features_features_29_relu_output_0,_features_features_30_conv_output_0_conv,_features_features_30_conv_output_0_div,_features_features_30_conv_output_0_rem,_features_features_30_conv_output_0_floor,_features_features_32_relu_output_0,_features_features_33_maxpool_output_0,_features_features_34_conv_output_0_conv,_features_features_34_conv_output_0_div,_features_features_34_conv_output_0_rem,_features_features_34_conv_output_0_floor,_features_features_36_relu_output_0,_features_features_37_conv_output_0_conv,_features_features_37_conv_output_0_div,_features_features_37_conv_output_0_rem,_features_features_37_conv_output_0_floor,_features_features_39_relu_output_0,_features_features_40_conv_output_0_conv,_features_features_40_conv_output_0_div,_features_features_40_conv_output_0_rem,_features_features_40_conv_output_0_floor,_features_features_42_relu_output_0,_features_features_43_maxpool_output_0,_avgpool_GlobalAveragePool_output_0,_classifier_classifier_0_gemm_output_0_matmul,_classifier_classifier_0_gemm_output_0_div,_classifier_classifier_0_gemm_output_0_rem,_classifier_classifier_0_gemm_output_0_floor,_classifier_classifier_1_relu_output_0,_classifier_classifier_3_gemm_output_0_matmul,_classifier_classifier_3_gemm_output_0_div,_classifier_classifier_3_gemm_output_0_rem,_classifier_classifier_3_gemm_output_0_floor,_classifier_classifier_4_relu_output_0,output_matmul,output_div,output_rem,output_floor,onnx_conv_150,onnx_conv_151,onnx_conv_151_q,onnx_conv_150_nscale,onnx_conv_150_dscale,onnx_conv_153,onnx_conv_154,onnx_conv_154_q,onnx_conv_153_nscale,onnx_conv_153_dscale,onnx_conv_156,onnx_conv_157,onnx_conv_157_q,onnx_conv_156_nscale,onnx_conv_156_dscale,onnx_conv_159,onnx_conv_160,onnx_conv_160_q,onnx_conv_159_nscale,onnx_conv_159_dscale,onnx_conv_162,onnx_conv_163,onnx_conv_163_q,onnx_conv_162_nscale,onnx_conv_162_dscale,onnx_conv_165,onnx_conv_166,onnx_conv_166_q,onnx_conv_165_nscale,onnx_conv_165_dscale,onnx_conv_168,onnx_conv_169,onnx_conv_169_q,onnx_conv_168_nscale,onnx_conv_168_dscale,onnx_conv_171,onnx_conv_172,onnx_conv_172_q,onnx_conv_171_nscale,onnx_conv_171_dscale,onnx_conv_174,onnx_conv_175,onnx_conv_175_q,onnx_conv_174_nscale,onnx_conv_174_dscale,onnx_conv_177,onnx_conv_178,onnx_conv_178_q,onnx_conv_177_nscale,onnx_conv_177_dscale,onnx_conv_180,onnx_conv_181,onnx_conv_181_q,onnx_conv_180_nscale,onnx_conv_180_dscale,onnx_conv_183,onnx_conv_184,onnx_conv_184_q,onnx_conv_183_nscale,onnx_conv_183_dscale,onnx_conv_186,onnx_conv_187,onnx_conv_187_q,onnx_conv_186_nscale,onnx_conv_186_dscale,classifier_0_weight,classifier_0_bias_q,classifier_0_weight_nscale,classifier_0_weight_dscale,classifier_3_weight,classifier_3_bias_q,classifier_3_weight_nscale,classifier_3_weight_dscale,classifier_6_weight,classifier_6_bias_q,classifier_6_weight_nscale,classifier_6_weight_dscale,input_mat_ru,onnx_conv_150_mat_rv,_features_features_2_relu_output_0_mat_ru,onnx_conv_153_mat_rv,_features_features_6_maxpool_output_0_mat_ru,onnx_conv_156_mat_rv,_features_features_9_relu_output_0_mat_ru,onnx_conv_159_mat_rv,_features_features_13_maxpool_output_0_mat_ru,onnx_conv_162_mat_rv,_features_features_16_relu_output_0_mat_ru,onnx_conv_165_mat_rv,_features_features_19_relu_output_0_mat_ru,onnx_conv_168_mat_rv,_features_features_23_maxpool_output_0_mat_ru,onnx_conv_171_mat_rv,_features_features_26_relu_output_0_mat_ru,onnx_conv_174_mat_rv,_features_features_29_relu_output_0_mat_ru,onnx_conv_177_mat_rv,_features_features_33_maxpool_output_0_mat_ru,onnx_conv_180_mat_rv,_features_features_36_relu_output_0_mat_ru,onnx_conv_183_mat_rv,_features_features_39_relu_output_0_mat_ru,onnx_conv_186_mat_rv,_Flatten_output_0_mat_ru,classifier_0_weight_mat_rv,_classifier_classifier_1_relu_output_0_mat_ru,classifier_3_weight_mat_rv,_classifier_classifier_4_relu_output_0_mat_ru,classifier_6_weight_mat_rv};
	ass
}

#[kernel]
fn _features_features_0_conv_conv_copy_macro<C: Config>(
	api: &mut API<C>,
	onnx_conv_150: &[[[[InputVariable;3];3];3];64],
	_features_features_0_conv_output_0_conv: &[[[[InputVariable;32];32];64];1],
	input: &[[[[InputVariable;32];32];3];1],

	onnx_conv_150_mat: &mut [[OutputVariable;64];27],
	_features_features_0_conv_output_0_conv_mat: &mut [[OutputVariable;1024];64],
	input_mat: &mut [[OutputVariable;1024];27],
) {
	// for i in 0..64 {
	// 	for j in 0..3 {
	// 		for k in 0..3 {
	// 			for l in 0..3 {
	// 				onnx_conv_150_mat[((j)*3 + k)*3 + l][i] = onnx_conv_150[i][j][k][l];
	// 			}
	// 		}
	// 	}
	// }
	// for i in 0..1 {
	// 	for j in 0..64 {
	// 		for k in 0..32 {
	// 			for l in 0..32 {
	// 				_features_features_0_conv_output_0_conv_mat[j][((i)*32 + k)*32 + l] = _features_features_0_conv_output_0_conv[i][j][k][l];
	// 			}
	// 		}
	// 	}
	// }
		for i in (0..(1 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(3 + 0 + 0 - 3 + 1)).step_by(3) {
				for k in (0..(32 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(32 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..3 {
								for o in 0..3 {
									for p in 0..3 {
										if true && (i+m-0) >= 0 && (i+m-0) < 1 && (j+n-0) >= 0 && (j+n-0) < 3 && (k+o-1) >= 0 && (k+o-1) < 32 && (l+p-1) >= 0 && (l+p-1) < 32 { input_mat[((n)*3 + o)*3 + p][((i)*32 + k)*32 + l] = input[i+m-0][j+n-0][k+o-1][l+p-1]}
										else { input_mat[((n)*3 + o)*3 + p][((i)*32 + k)*32 + l] = api.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
}

#[kernel]
fn _features_features_0_conv_conv_ab_matrix_macro<C: Config>(
	api: &mut API<C>,
	input_mat: & [InputVariable;1024],
	onnx_conv_150_mat: & [InputVariable;64],
	input_mat_ru: & [InputVariable;1024],
	onnx_conv_150_mat_rv: & [InputVariable;64],
	_features_features_0_conv_conv_ab_matrix_rx: &mut OutputVariable,
	_features_features_0_conv_conv_ab_matrix_ry: &mut OutputVariable,
) {
	*_features_features_0_conv_conv_ab_matrix_rx = api.constant(0);
	for i in 0..1024 {
		let tmp = api.mul(input_mat_ru[i], input_mat[i]);
		*_features_features_0_conv_conv_ab_matrix_rx = api.add(tmp, *_features_features_0_conv_conv_ab_matrix_rx);
	}
	*_features_features_0_conv_conv_ab_matrix_ry = api.constant(0);
	for i in 0..64 {
		let tmp = api.mul(onnx_conv_150_mat_rv[i], onnx_conv_150_mat[i]);
		*_features_features_0_conv_conv_ab_matrix_ry = api.add(tmp, *_features_features_0_conv_conv_ab_matrix_ry);
	}
}
#[kernel]
fn _features_features_0_conv_conv_c_matrix_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_conv_output_0_conv_mat: & [InputVariable;1024],
	input_mat_ru: & [InputVariable;1024],
	_features_features_0_conv_conv_c_matrix_rz: &mut OutputVariable,
) {
	*_features_features_0_conv_conv_c_matrix_rz = api.constant(0);
	for i in 0..1024 {
		let tmp = api.mul(input_mat_ru[i], _features_features_0_conv_output_0_conv_mat[i]);
		*_features_features_0_conv_conv_c_matrix_rz = api.add(tmp, *_features_features_0_conv_conv_c_matrix_rz);
	}
}

#[kernel]		// multiply operation
fn _features_features_0_conv_mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_conv_output_0_conv: &[[InputVariable;32];32],
	onnx_conv_150_nscale: &InputVariable,
	_features_features_0_conv_output_0_mul: &mut [[OutputVariable;32];32],
) {
	for i in 0..32 {
		for j in 0..32 {
			_features_features_0_conv_output_0_mul[i][j] = api.mul(_features_features_0_conv_output_0_conv[i][j], onnx_conv_150_nscale);
		}
	}
}

#[kernel]		// divide operation
fn _features_features_0_conv_div_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_conv_output_0_mul: &[[InputVariable;32];32],
	onnx_conv_150_dscale: &InputVariable,
	_features_features_0_conv_output_0_floor: &[[InputVariable;32];32],
	_features_features_0_conv_output_0_rem: &[[InputVariable;32];32],
) {
	for i in 0..32 {
		for j in 0..32 {
			let tmp1 = api.mul(_features_features_0_conv_output_0_floor[i][j], onnx_conv_150_dscale);
			let tmp2 = api.sub(_features_features_0_conv_output_0_mul[i][j], _features_features_0_conv_output_0_rem[i][j]);
			api.assert_is_equal(tmp1, tmp2);
		}
	}
}

#[test]
fn expander_circuit() -> std::io::Result<()>{ 
	let compile_result = stacker::grow(32 * 1024 * 1024 * 1024, ||
		{
			let mut ctx = Context::<BN254Config>::default();
			let mut assignment = default_variable();

			let onnx_conv_150_mat = ctx.copy_to_device(&assignment.onnx_conv_150);  // [64, 3, 3, 3]
			let onnx_conv_150_mat = onnx_conv_150_mat.reshape(&[64, 27]);       // [64, 27]
			let onnx_conv_150_mat = onnx_conv_150_mat.transpose(&[1, 0]);       // [27, 64]

			let kernel__features_features_0_conv_conv_ab_matrix: KernelPrimitive<BN254Config> = compile__features_features_0_conv_conv_ab_matrix_macro().unwrap();
			let input_mat = ctx.copy_to_device(&vec![vec![BN254Fr::default();1024];27]);
			let input_mat_ru = ctx.copy_to_device(&assignment.input_mat_ru);
			let onnx_conv_150_mat_rv = ctx.copy_to_device(&assignment.onnx_conv_150_mat_rv);
			let mut _features_features_0_conv_conv_rx = None;
			let mut _features_features_0_conv_conv_ry = None;
			let mut input_mat_clone = input_mat.clone();
			let mut onnx_conv_150_mat_clone = onnx_conv_150_mat.clone();
			let mut input_mat_ru_clone = input_mat_ru.clone();
			let mut onnx_conv_150_mat_rv_clone = onnx_conv_150_mat_rv.clone();
			call_kernel!(ctx, kernel__features_features_0_conv_conv_ab_matrix, 27, input_mat_clone, onnx_conv_150_mat_clone, input_mat_ru_clone, onnx_conv_150_mat_rv_clone, mut _features_features_0_conv_conv_rx, mut _features_features_0_conv_conv_ry).unwrap();
			
			let _features_features_0_conv_output_0_conv = ctx.copy_to_device(&assignment._features_features_0_conv_output_0_conv);  // [1, 64, 32, 32]
			let _features_features_0_conv_output_0_conv_mat = _features_features_0_conv_output_0_conv.transpose(&[1, 0, 2, 3]);   // [64, 1, 32, 32]
			let _features_features_0_conv_output_0_conv_mat = _features_features_0_conv_output_0_conv_mat.reshape(&[64, 1024]);   // [64, 1024]

			let kernel__features_features_0_conv_conv_c_matrix: KernelPrimitive<BN254Config> = compile__features_features_0_conv_conv_c_matrix_macro().unwrap();
			// let _features_features_0_conv_output_0_conv_mat = ctx.copy_to_device(&vec![vec![BN254Fr::default();1024];64]);
			let mut _features_features_0_conv_conv_rz = None;
			let _features_features_0_conv_output_0_conv_mat_clone = _features_features_0_conv_output_0_conv_mat.clone();
			let input_mat_ru_clone = input_mat_ru.clone();
			call_kernel!(ctx, kernel__features_features_0_conv_conv_c_matrix, 64, _features_features_0_conv_output_0_conv_mat_clone, input_mat_ru_clone, mut _features_features_0_conv_conv_rz).unwrap();

			let computation_graph = ctx.compile_computation_graph().unwrap();
			let file = std::fs::File::create("graph.txt").unwrap();
			let writer = std::io::BufWriter::new(file);
			computation_graph.serialize_into(writer);
		}
	);
	Ok(())
}
