use expander_compiler::frontend::*;
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

impl Define<BN254Config> for Circuit<Variable> {
	// fn define(&self, builder: &mut API<BN254Config>) {
	fn define<Builder: RootAPI<BN254Config>>(&self, builder: &mut Builder) {
		let mut table = LogUpRangeProofTable::new(24);
		table.initial(builder);
		// conv operation
		let mut features_0_conv_weight_mat: [[Variable;64];27] = [[Default::default();64];27];
		for i in 0..64 {
			for j in 0..3 {
				for k in 0..3 {
					for l in 0..3 {
						features_0_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_0_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_0_conv_Conv_output_0_mat: [[Variable;64];16384] = [[Default::default();64];16384];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_0_conv_Conv_output_0_mat[((i)*32 + k)*32 + l][j] = self._features_features_0_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut input_mat: [[Variable;27];16384] = [[Default::default();27];16384];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(3 + 0 + 0 - 3 + 1)).step_by(3) {
				for k in (0..(32 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(32 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..3 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 3 && (k+o-1) >= 0 && (k+o-1) < 32 && (l+p-1) >= 0 && (l+p-1) < 32 { input_mat[((i)*32 + k)*32 + l][((n)*3 + o)*3 + p] = self.input[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { input_mat[((i)*32 + k)*32 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_0_conv_Conv_rx: [Variable;27] = [Default::default();27];
		let mut _features_features_0_conv_Conv_ry: [Variable;27] = [Default::default();27];
		for i in 0..27 {
			let mut _features_features_0_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..16384 {
				let tmp = builder.mul(self.input_mat_ru[j], input_mat[j][i]);
				_features_features_0_conv_Conv_rx_tmp = builder.add(tmp, _features_features_0_conv_Conv_rx_tmp);
			}
			_features_features_0_conv_Conv_rx[i] = _features_features_0_conv_Conv_rx_tmp;
		}
		for i in 0..27 {
			let mut _features_features_0_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..64 {
				let tmp = builder.mul(self.features_0_conv_weight_mat_rv[j], features_0_conv_weight_mat[i][j]);
				_features_features_0_conv_Conv_ry_tmp = builder.add(tmp, _features_features_0_conv_Conv_ry_tmp);
			}
			_features_features_0_conv_Conv_ry[i] = _features_features_0_conv_Conv_ry_tmp;
		}
		let mut _features_features_0_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_0_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..27 {
			let tmp = builder.mul(_features_features_0_conv_Conv_ry[i], _features_features_0_conv_Conv_rx[i]);
			_features_features_0_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_0_conv_Conv_rxy_tmp);
		}
		_features_features_0_conv_Conv_rxy = _features_features_0_conv_Conv_rxy_tmp;
		let mut _features_features_0_conv_Conv_rz: [Variable;64] = [Default::default();64];
		for i in 0..64 {
			let mut _features_features_0_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..16384 {
				let tmp = builder.mul(self.input_mat_ru[j], _features_features_0_conv_Conv_output_0_mat[j][i]);
				_features_features_0_conv_Conv_rz_tmp = builder.add(tmp, _features_features_0_conv_Conv_rz_tmp);
			}
			_features_features_0_conv_Conv_rz[i] = _features_features_0_conv_Conv_rz_tmp;
		}
		let mut _features_features_0_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_0_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..64 {
			let tmp = builder.mul(self.features_0_conv_weight_mat_rv[i], _features_features_0_conv_Conv_rz[i]);
			_features_features_0_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_0_conv_Conv_rrz_tmp);
		}
		_features_features_0_conv_Conv_rrz = _features_features_0_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_0_conv_Conv_rrz, _features_features_0_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_0_Mul_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_0_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_0_conv_Conv_output_0[i][j][k][l], self._features_features_0_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						let tmp1 = builder.mul(self._features_features_0_Div_output_0[i][j][k][l], self._features_features_0_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_0_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_0_Mul_output_0[i][j][k][l], self._features_features_0_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_0_Cast_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_0_Cast_output_0[i][j][k][l] = self._features_features_0_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_0_Cast_1_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_0_Cast_1_output_0[i][j][k][l] = _features_features_0_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_0_Add_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_0_Add_output_0[i][j][k][l] = builder.add(_features_features_0_Cast_1_output_0[i][j][k][l], self._features_features_0_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_2_relu_Cast_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_2_relu_Cast_output_0[i][j][k][l] = _features_features_0_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						let tmp1 = builder.sub(self._features_features_2_relu_Relu_output_0[i][j][k][l], _features_features_2_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_2_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_2_relu_Cast_1_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_2_relu_Cast_1_output_0[i][j][k][l] = self._features_features_2_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// conv operation
		let mut features_3_conv_weight_mat: [[Variable;64];576] = [[Default::default();64];576];
		for i in 0..64 {
			for j in 0..64 {
				for k in 0..3 {
					for l in 0..3 {
						features_3_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_3_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_3_conv_Conv_output_0_mat: [[Variable;64];16384] = [[Default::default();64];16384];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_3_conv_Conv_output_0_mat[((i)*32 + k)*32 + l][j] = self._features_features_3_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_2_relu_Cast_1_output_0_mat: [[Variable;576];16384] = [[Default::default();576];16384];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(64 + 0 + 0 - 64 + 1)).step_by(64) {
				for k in (0..(32 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(32 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..64 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 64 && (k+o-1) >= 0 && (k+o-1) < 32 && (l+p-1) >= 0 && (l+p-1) < 32 { _features_features_2_relu_Cast_1_output_0_mat[((i)*32 + k)*32 + l][((n)*3 + o)*3 + p] = _features_features_2_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_2_relu_Cast_1_output_0_mat[((i)*32 + k)*32 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_3_conv_Conv_rx: [Variable;576] = [Default::default();576];
		let mut _features_features_3_conv_Conv_ry: [Variable;576] = [Default::default();576];
		for i in 0..576 {
			let mut _features_features_3_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..16384 {
				let tmp = builder.mul(self._features_features_2_relu_Cast_1_output_0_mat_ru[j], _features_features_2_relu_Cast_1_output_0_mat[j][i]);
				_features_features_3_conv_Conv_rx_tmp = builder.add(tmp, _features_features_3_conv_Conv_rx_tmp);
			}
			_features_features_3_conv_Conv_rx[i] = _features_features_3_conv_Conv_rx_tmp;
		}
		for i in 0..576 {
			let mut _features_features_3_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..64 {
				let tmp = builder.mul(self.features_3_conv_weight_mat_rv[j], features_3_conv_weight_mat[i][j]);
				_features_features_3_conv_Conv_ry_tmp = builder.add(tmp, _features_features_3_conv_Conv_ry_tmp);
			}
			_features_features_3_conv_Conv_ry[i] = _features_features_3_conv_Conv_ry_tmp;
		}
		let mut _features_features_3_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_3_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..576 {
			let tmp = builder.mul(_features_features_3_conv_Conv_ry[i], _features_features_3_conv_Conv_rx[i]);
			_features_features_3_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_3_conv_Conv_rxy_tmp);
		}
		_features_features_3_conv_Conv_rxy = _features_features_3_conv_Conv_rxy_tmp;
		let mut _features_features_3_conv_Conv_rz: [Variable;64] = [Default::default();64];
		for i in 0..64 {
			let mut _features_features_3_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..16384 {
				let tmp = builder.mul(self._features_features_2_relu_Cast_1_output_0_mat_ru[j], _features_features_3_conv_Conv_output_0_mat[j][i]);
				_features_features_3_conv_Conv_rz_tmp = builder.add(tmp, _features_features_3_conv_Conv_rz_tmp);
			}
			_features_features_3_conv_Conv_rz[i] = _features_features_3_conv_Conv_rz_tmp;
		}
		let mut _features_features_3_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_3_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..64 {
			let tmp = builder.mul(self.features_3_conv_weight_mat_rv[i], _features_features_3_conv_Conv_rz[i]);
			_features_features_3_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_3_conv_Conv_rrz_tmp);
		}
		_features_features_3_conv_Conv_rrz = _features_features_3_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_3_conv_Conv_rrz, _features_features_3_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_3_Mul_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_3_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_3_conv_Conv_output_0[i][j][k][l], self._features_features_3_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						let tmp1 = builder.mul(self._features_features_3_Div_output_0[i][j][k][l], self._features_features_3_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_3_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_3_Mul_output_0[i][j][k][l], self._features_features_3_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_3_Cast_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_3_Cast_output_0[i][j][k][l] = self._features_features_3_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_3_Cast_1_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_3_Cast_1_output_0[i][j][k][l] = _features_features_3_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_3_Add_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_3_Add_output_0[i][j][k][l] = builder.add(_features_features_3_Cast_1_output_0[i][j][k][l], self._features_features_3_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_5_relu_Cast_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_5_relu_Cast_output_0[i][j][k][l] = _features_features_3_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						let tmp1 = builder.sub(self._features_features_5_relu_Relu_output_0[i][j][k][l], _features_features_5_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_5_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_5_relu_Cast_1_output_0: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						_features_features_5_relu_Cast_1_output_0[i][j][k][l] = self._features_features_5_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// maxpool operation
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(64 + 0 + 0 - 1 + 1)).step_by(1) {
				for k in (0..(32 + 0 + 0 - 2 + 1)).step_by(2) {
					for l in (0..(32 + 0 + 0 - 2 + 1)).step_by(2) {
					let mut tmp = builder.constant(1);
						for m in 0..1 {
							for n in 0..1 {
								for o in 0..2 {
									for p in 0..2 {
									let sub_tmp = builder.sub(self._features_features_6_maxpool_MaxPool_output_0[i/1][j/1][k/2][l/2], _features_features_5_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-0][l+p-0]);
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 64 && (k+o-0) >= 0 && (k+o-0) < 32 && (l+p-0) >= 0 && (l+p-0) < 32 { tmp = builder.mul(tmp, sub_tmp)}
									}
								}
							}
						}
					builder.assert_is_zero(tmp);
					}
				}
			}
		}
		// conv operation
		let mut features_7_conv_weight_mat: [[Variable;128];576] = [[Default::default();128];576];
		for i in 0..128 {
			for j in 0..64 {
				for k in 0..3 {
					for l in 0..3 {
						features_7_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_7_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_7_conv_Conv_output_0_mat: [[Variable;128];4096] = [[Default::default();128];4096];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_7_conv_Conv_output_0_mat[((i)*16 + k)*16 + l][j] = self._features_features_7_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_6_maxpool_MaxPool_output_0_mat: [[Variable;576];4096] = [[Default::default();576];4096];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(64 + 0 + 0 - 64 + 1)).step_by(64) {
				for k in (0..(16 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(16 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..64 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 64 && (k+o-1) >= 0 && (k+o-1) < 16 && (l+p-1) >= 0 && (l+p-1) < 16 { _features_features_6_maxpool_MaxPool_output_0_mat[((i)*16 + k)*16 + l][((n)*3 + o)*3 + p] = self._features_features_6_maxpool_MaxPool_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_6_maxpool_MaxPool_output_0_mat[((i)*16 + k)*16 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_7_conv_Conv_rx: [Variable;576] = [Default::default();576];
		let mut _features_features_7_conv_Conv_ry: [Variable;576] = [Default::default();576];
		for i in 0..576 {
			let mut _features_features_7_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..4096 {
				let tmp = builder.mul(self._features_features_6_maxpool_MaxPool_output_0_mat_ru[j], _features_features_6_maxpool_MaxPool_output_0_mat[j][i]);
				_features_features_7_conv_Conv_rx_tmp = builder.add(tmp, _features_features_7_conv_Conv_rx_tmp);
			}
			_features_features_7_conv_Conv_rx[i] = _features_features_7_conv_Conv_rx_tmp;
		}
		for i in 0..576 {
			let mut _features_features_7_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..128 {
				let tmp = builder.mul(self.features_7_conv_weight_mat_rv[j], features_7_conv_weight_mat[i][j]);
				_features_features_7_conv_Conv_ry_tmp = builder.add(tmp, _features_features_7_conv_Conv_ry_tmp);
			}
			_features_features_7_conv_Conv_ry[i] = _features_features_7_conv_Conv_ry_tmp;
		}
		let mut _features_features_7_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_7_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..576 {
			let tmp = builder.mul(_features_features_7_conv_Conv_ry[i], _features_features_7_conv_Conv_rx[i]);
			_features_features_7_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_7_conv_Conv_rxy_tmp);
		}
		_features_features_7_conv_Conv_rxy = _features_features_7_conv_Conv_rxy_tmp;
		let mut _features_features_7_conv_Conv_rz: [Variable;128] = [Default::default();128];
		for i in 0..128 {
			let mut _features_features_7_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..4096 {
				let tmp = builder.mul(self._features_features_6_maxpool_MaxPool_output_0_mat_ru[j], _features_features_7_conv_Conv_output_0_mat[j][i]);
				_features_features_7_conv_Conv_rz_tmp = builder.add(tmp, _features_features_7_conv_Conv_rz_tmp);
			}
			_features_features_7_conv_Conv_rz[i] = _features_features_7_conv_Conv_rz_tmp;
		}
		let mut _features_features_7_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_7_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..128 {
			let tmp = builder.mul(self.features_7_conv_weight_mat_rv[i], _features_features_7_conv_Conv_rz[i]);
			_features_features_7_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_7_conv_Conv_rrz_tmp);
		}
		_features_features_7_conv_Conv_rrz = _features_features_7_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_7_conv_Conv_rrz, _features_features_7_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_7_Mul_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_7_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_7_conv_Conv_output_0[i][j][k][l], self._features_features_7_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						let tmp1 = builder.mul(self._features_features_7_Div_output_0[i][j][k][l], self._features_features_7_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_7_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_7_Mul_output_0[i][j][k][l], self._features_features_7_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_7_Cast_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_7_Cast_output_0[i][j][k][l] = self._features_features_7_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_7_Cast_1_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_7_Cast_1_output_0[i][j][k][l] = _features_features_7_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_7_Add_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_7_Add_output_0[i][j][k][l] = builder.add(_features_features_7_Cast_1_output_0[i][j][k][l], self._features_features_7_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_9_relu_Cast_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_9_relu_Cast_output_0[i][j][k][l] = _features_features_7_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						let tmp1 = builder.sub(self._features_features_9_relu_Relu_output_0[i][j][k][l], _features_features_9_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_9_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_9_relu_Cast_1_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_9_relu_Cast_1_output_0[i][j][k][l] = self._features_features_9_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// conv operation
		let mut features_10_conv_weight_mat: [[Variable;128];1152] = [[Default::default();128];1152];
		for i in 0..128 {
			for j in 0..128 {
				for k in 0..3 {
					for l in 0..3 {
						features_10_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_10_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_10_conv_Conv_output_0_mat: [[Variable;128];4096] = [[Default::default();128];4096];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_10_conv_Conv_output_0_mat[((i)*16 + k)*16 + l][j] = self._features_features_10_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_9_relu_Cast_1_output_0_mat: [[Variable;1152];4096] = [[Default::default();1152];4096];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(128 + 0 + 0 - 128 + 1)).step_by(128) {
				for k in (0..(16 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(16 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..128 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 128 && (k+o-1) >= 0 && (k+o-1) < 16 && (l+p-1) >= 0 && (l+p-1) < 16 { _features_features_9_relu_Cast_1_output_0_mat[((i)*16 + k)*16 + l][((n)*3 + o)*3 + p] = _features_features_9_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_9_relu_Cast_1_output_0_mat[((i)*16 + k)*16 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_10_conv_Conv_rx: [Variable;1152] = [Default::default();1152];
		let mut _features_features_10_conv_Conv_ry: [Variable;1152] = [Default::default();1152];
		for i in 0..1152 {
			let mut _features_features_10_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..4096 {
				let tmp = builder.mul(self._features_features_9_relu_Cast_1_output_0_mat_ru[j], _features_features_9_relu_Cast_1_output_0_mat[j][i]);
				_features_features_10_conv_Conv_rx_tmp = builder.add(tmp, _features_features_10_conv_Conv_rx_tmp);
			}
			_features_features_10_conv_Conv_rx[i] = _features_features_10_conv_Conv_rx_tmp;
		}
		for i in 0..1152 {
			let mut _features_features_10_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..128 {
				let tmp = builder.mul(self.features_10_conv_weight_mat_rv[j], features_10_conv_weight_mat[i][j]);
				_features_features_10_conv_Conv_ry_tmp = builder.add(tmp, _features_features_10_conv_Conv_ry_tmp);
			}
			_features_features_10_conv_Conv_ry[i] = _features_features_10_conv_Conv_ry_tmp;
		}
		let mut _features_features_10_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_10_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..1152 {
			let tmp = builder.mul(_features_features_10_conv_Conv_ry[i], _features_features_10_conv_Conv_rx[i]);
			_features_features_10_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_10_conv_Conv_rxy_tmp);
		}
		_features_features_10_conv_Conv_rxy = _features_features_10_conv_Conv_rxy_tmp;
		let mut _features_features_10_conv_Conv_rz: [Variable;128] = [Default::default();128];
		for i in 0..128 {
			let mut _features_features_10_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..4096 {
				let tmp = builder.mul(self._features_features_9_relu_Cast_1_output_0_mat_ru[j], _features_features_10_conv_Conv_output_0_mat[j][i]);
				_features_features_10_conv_Conv_rz_tmp = builder.add(tmp, _features_features_10_conv_Conv_rz_tmp);
			}
			_features_features_10_conv_Conv_rz[i] = _features_features_10_conv_Conv_rz_tmp;
		}
		let mut _features_features_10_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_10_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..128 {
			let tmp = builder.mul(self.features_10_conv_weight_mat_rv[i], _features_features_10_conv_Conv_rz[i]);
			_features_features_10_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_10_conv_Conv_rrz_tmp);
		}
		_features_features_10_conv_Conv_rrz = _features_features_10_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_10_conv_Conv_rrz, _features_features_10_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_10_Mul_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_10_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_10_conv_Conv_output_0[i][j][k][l], self._features_features_10_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						let tmp1 = builder.mul(self._features_features_10_Div_output_0[i][j][k][l], self._features_features_10_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_10_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_10_Mul_output_0[i][j][k][l], self._features_features_10_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_10_Cast_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_10_Cast_output_0[i][j][k][l] = self._features_features_10_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_10_Cast_1_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_10_Cast_1_output_0[i][j][k][l] = _features_features_10_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_10_Add_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_10_Add_output_0[i][j][k][l] = builder.add(_features_features_10_Cast_1_output_0[i][j][k][l], self._features_features_10_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_12_relu_Cast_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_12_relu_Cast_output_0[i][j][k][l] = _features_features_10_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						let tmp1 = builder.sub(self._features_features_12_relu_Relu_output_0[i][j][k][l], _features_features_12_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_12_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_12_relu_Cast_1_output_0: [[[[Variable;16];16];128];16] = [[[[Default::default();16];16];128];16];
		for i in 0..16 {
			for j in 0..128 {
				for k in 0..16 {
					for l in 0..16 {
						_features_features_12_relu_Cast_1_output_0[i][j][k][l] = self._features_features_12_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// maxpool operation
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(128 + 0 + 0 - 1 + 1)).step_by(1) {
				for k in (0..(16 + 0 + 0 - 2 + 1)).step_by(2) {
					for l in (0..(16 + 0 + 0 - 2 + 1)).step_by(2) {
					let mut tmp = builder.constant(1);
						for m in 0..1 {
							for n in 0..1 {
								for o in 0..2 {
									for p in 0..2 {
									let sub_tmp = builder.sub(self._features_features_13_maxpool_MaxPool_output_0[i/1][j/1][k/2][l/2], _features_features_12_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-0][l+p-0]);
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 128 && (k+o-0) >= 0 && (k+o-0) < 16 && (l+p-0) >= 0 && (l+p-0) < 16 { tmp = builder.mul(tmp, sub_tmp)}
									}
								}
							}
						}
					builder.assert_is_zero(tmp);
					}
				}
			}
		}
		// conv operation
		let mut features_14_conv_weight_mat: [[Variable;256];1152] = [[Default::default();256];1152];
		for i in 0..256 {
			for j in 0..128 {
				for k in 0..3 {
					for l in 0..3 {
						features_14_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_14_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_14_conv_Conv_output_0_mat: [[Variable;256];1024] = [[Default::default();256];1024];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_14_conv_Conv_output_0_mat[((i)*8 + k)*8 + l][j] = self._features_features_14_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_13_maxpool_MaxPool_output_0_mat: [[Variable;1152];1024] = [[Default::default();1152];1024];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(128 + 0 + 0 - 128 + 1)).step_by(128) {
				for k in (0..(8 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(8 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..128 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 128 && (k+o-1) >= 0 && (k+o-1) < 8 && (l+p-1) >= 0 && (l+p-1) < 8 { _features_features_13_maxpool_MaxPool_output_0_mat[((i)*8 + k)*8 + l][((n)*3 + o)*3 + p] = self._features_features_13_maxpool_MaxPool_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_13_maxpool_MaxPool_output_0_mat[((i)*8 + k)*8 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_14_conv_Conv_rx: [Variable;1152] = [Default::default();1152];
		let mut _features_features_14_conv_Conv_ry: [Variable;1152] = [Default::default();1152];
		for i in 0..1152 {
			let mut _features_features_14_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..1024 {
				let tmp = builder.mul(self._features_features_13_maxpool_MaxPool_output_0_mat_ru[j], _features_features_13_maxpool_MaxPool_output_0_mat[j][i]);
				_features_features_14_conv_Conv_rx_tmp = builder.add(tmp, _features_features_14_conv_Conv_rx_tmp);
			}
			_features_features_14_conv_Conv_rx[i] = _features_features_14_conv_Conv_rx_tmp;
		}
		for i in 0..1152 {
			let mut _features_features_14_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..256 {
				let tmp = builder.mul(self.features_14_conv_weight_mat_rv[j], features_14_conv_weight_mat[i][j]);
				_features_features_14_conv_Conv_ry_tmp = builder.add(tmp, _features_features_14_conv_Conv_ry_tmp);
			}
			_features_features_14_conv_Conv_ry[i] = _features_features_14_conv_Conv_ry_tmp;
		}
		let mut _features_features_14_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_14_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..1152 {
			let tmp = builder.mul(_features_features_14_conv_Conv_ry[i], _features_features_14_conv_Conv_rx[i]);
			_features_features_14_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_14_conv_Conv_rxy_tmp);
		}
		_features_features_14_conv_Conv_rxy = _features_features_14_conv_Conv_rxy_tmp;
		let mut _features_features_14_conv_Conv_rz: [Variable;256] = [Default::default();256];
		for i in 0..256 {
			let mut _features_features_14_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..1024 {
				let tmp = builder.mul(self._features_features_13_maxpool_MaxPool_output_0_mat_ru[j], _features_features_14_conv_Conv_output_0_mat[j][i]);
				_features_features_14_conv_Conv_rz_tmp = builder.add(tmp, _features_features_14_conv_Conv_rz_tmp);
			}
			_features_features_14_conv_Conv_rz[i] = _features_features_14_conv_Conv_rz_tmp;
		}
		let mut _features_features_14_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_14_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..256 {
			let tmp = builder.mul(self.features_14_conv_weight_mat_rv[i], _features_features_14_conv_Conv_rz[i]);
			_features_features_14_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_14_conv_Conv_rrz_tmp);
		}
		_features_features_14_conv_Conv_rrz = _features_features_14_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_14_conv_Conv_rrz, _features_features_14_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_14_Mul_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_14_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_14_conv_Conv_output_0[i][j][k][l], self._features_features_14_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						let tmp1 = builder.mul(self._features_features_14_Div_output_0[i][j][k][l], self._features_features_14_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_14_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_14_Mul_output_0[i][j][k][l], self._features_features_14_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_14_Cast_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_14_Cast_output_0[i][j][k][l] = self._features_features_14_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_14_Cast_1_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_14_Cast_1_output_0[i][j][k][l] = _features_features_14_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_14_Add_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_14_Add_output_0[i][j][k][l] = builder.add(_features_features_14_Cast_1_output_0[i][j][k][l], self._features_features_14_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_16_relu_Cast_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_16_relu_Cast_output_0[i][j][k][l] = _features_features_14_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						let tmp1 = builder.sub(self._features_features_16_relu_Relu_output_0[i][j][k][l], _features_features_16_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_16_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_16_relu_Cast_1_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_16_relu_Cast_1_output_0[i][j][k][l] = self._features_features_16_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// conv operation
		let mut features_17_conv_weight_mat: [[Variable;256];2304] = [[Default::default();256];2304];
		for i in 0..256 {
			for j in 0..256 {
				for k in 0..3 {
					for l in 0..3 {
						features_17_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_17_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_17_conv_Conv_output_0_mat: [[Variable;256];1024] = [[Default::default();256];1024];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_17_conv_Conv_output_0_mat[((i)*8 + k)*8 + l][j] = self._features_features_17_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_16_relu_Cast_1_output_0_mat: [[Variable;2304];1024] = [[Default::default();2304];1024];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(256 + 0 + 0 - 256 + 1)).step_by(256) {
				for k in (0..(8 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(8 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..256 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 256 && (k+o-1) >= 0 && (k+o-1) < 8 && (l+p-1) >= 0 && (l+p-1) < 8 { _features_features_16_relu_Cast_1_output_0_mat[((i)*8 + k)*8 + l][((n)*3 + o)*3 + p] = _features_features_16_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_16_relu_Cast_1_output_0_mat[((i)*8 + k)*8 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_17_conv_Conv_rx: [Variable;2304] = [Default::default();2304];
		let mut _features_features_17_conv_Conv_ry: [Variable;2304] = [Default::default();2304];
		for i in 0..2304 {
			let mut _features_features_17_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..1024 {
				let tmp = builder.mul(self._features_features_16_relu_Cast_1_output_0_mat_ru[j], _features_features_16_relu_Cast_1_output_0_mat[j][i]);
				_features_features_17_conv_Conv_rx_tmp = builder.add(tmp, _features_features_17_conv_Conv_rx_tmp);
			}
			_features_features_17_conv_Conv_rx[i] = _features_features_17_conv_Conv_rx_tmp;
		}
		for i in 0..2304 {
			let mut _features_features_17_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..256 {
				let tmp = builder.mul(self.features_17_conv_weight_mat_rv[j], features_17_conv_weight_mat[i][j]);
				_features_features_17_conv_Conv_ry_tmp = builder.add(tmp, _features_features_17_conv_Conv_ry_tmp);
			}
			_features_features_17_conv_Conv_ry[i] = _features_features_17_conv_Conv_ry_tmp;
		}
		let mut _features_features_17_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_17_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..2304 {
			let tmp = builder.mul(_features_features_17_conv_Conv_ry[i], _features_features_17_conv_Conv_rx[i]);
			_features_features_17_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_17_conv_Conv_rxy_tmp);
		}
		_features_features_17_conv_Conv_rxy = _features_features_17_conv_Conv_rxy_tmp;
		let mut _features_features_17_conv_Conv_rz: [Variable;256] = [Default::default();256];
		for i in 0..256 {
			let mut _features_features_17_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..1024 {
				let tmp = builder.mul(self._features_features_16_relu_Cast_1_output_0_mat_ru[j], _features_features_17_conv_Conv_output_0_mat[j][i]);
				_features_features_17_conv_Conv_rz_tmp = builder.add(tmp, _features_features_17_conv_Conv_rz_tmp);
			}
			_features_features_17_conv_Conv_rz[i] = _features_features_17_conv_Conv_rz_tmp;
		}
		let mut _features_features_17_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_17_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..256 {
			let tmp = builder.mul(self.features_17_conv_weight_mat_rv[i], _features_features_17_conv_Conv_rz[i]);
			_features_features_17_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_17_conv_Conv_rrz_tmp);
		}
		_features_features_17_conv_Conv_rrz = _features_features_17_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_17_conv_Conv_rrz, _features_features_17_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_17_Mul_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_17_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_17_conv_Conv_output_0[i][j][k][l], self._features_features_17_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						let tmp1 = builder.mul(self._features_features_17_Div_output_0[i][j][k][l], self._features_features_17_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_17_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_17_Mul_output_0[i][j][k][l], self._features_features_17_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_17_Cast_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_17_Cast_output_0[i][j][k][l] = self._features_features_17_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_17_Cast_1_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_17_Cast_1_output_0[i][j][k][l] = _features_features_17_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_17_Add_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_17_Add_output_0[i][j][k][l] = builder.add(_features_features_17_Cast_1_output_0[i][j][k][l], self._features_features_17_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_19_relu_Cast_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_19_relu_Cast_output_0[i][j][k][l] = _features_features_17_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						let tmp1 = builder.sub(self._features_features_19_relu_Relu_output_0[i][j][k][l], _features_features_19_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_19_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_19_relu_Cast_1_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_19_relu_Cast_1_output_0[i][j][k][l] = self._features_features_19_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// conv operation
		let mut features_20_conv_weight_mat: [[Variable;256];2304] = [[Default::default();256];2304];
		for i in 0..256 {
			for j in 0..256 {
				for k in 0..3 {
					for l in 0..3 {
						features_20_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_20_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_20_conv_Conv_output_0_mat: [[Variable;256];1024] = [[Default::default();256];1024];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_20_conv_Conv_output_0_mat[((i)*8 + k)*8 + l][j] = self._features_features_20_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_19_relu_Cast_1_output_0_mat: [[Variable;2304];1024] = [[Default::default();2304];1024];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(256 + 0 + 0 - 256 + 1)).step_by(256) {
				for k in (0..(8 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(8 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..256 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 256 && (k+o-1) >= 0 && (k+o-1) < 8 && (l+p-1) >= 0 && (l+p-1) < 8 { _features_features_19_relu_Cast_1_output_0_mat[((i)*8 + k)*8 + l][((n)*3 + o)*3 + p] = _features_features_19_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_19_relu_Cast_1_output_0_mat[((i)*8 + k)*8 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_20_conv_Conv_rx: [Variable;2304] = [Default::default();2304];
		let mut _features_features_20_conv_Conv_ry: [Variable;2304] = [Default::default();2304];
		for i in 0..2304 {
			let mut _features_features_20_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..1024 {
				let tmp = builder.mul(self._features_features_19_relu_Cast_1_output_0_mat_ru[j], _features_features_19_relu_Cast_1_output_0_mat[j][i]);
				_features_features_20_conv_Conv_rx_tmp = builder.add(tmp, _features_features_20_conv_Conv_rx_tmp);
			}
			_features_features_20_conv_Conv_rx[i] = _features_features_20_conv_Conv_rx_tmp;
		}
		for i in 0..2304 {
			let mut _features_features_20_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..256 {
				let tmp = builder.mul(self.features_20_conv_weight_mat_rv[j], features_20_conv_weight_mat[i][j]);
				_features_features_20_conv_Conv_ry_tmp = builder.add(tmp, _features_features_20_conv_Conv_ry_tmp);
			}
			_features_features_20_conv_Conv_ry[i] = _features_features_20_conv_Conv_ry_tmp;
		}
		let mut _features_features_20_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_20_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..2304 {
			let tmp = builder.mul(_features_features_20_conv_Conv_ry[i], _features_features_20_conv_Conv_rx[i]);
			_features_features_20_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_20_conv_Conv_rxy_tmp);
		}
		_features_features_20_conv_Conv_rxy = _features_features_20_conv_Conv_rxy_tmp;
		let mut _features_features_20_conv_Conv_rz: [Variable;256] = [Default::default();256];
		for i in 0..256 {
			let mut _features_features_20_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..1024 {
				let tmp = builder.mul(self._features_features_19_relu_Cast_1_output_0_mat_ru[j], _features_features_20_conv_Conv_output_0_mat[j][i]);
				_features_features_20_conv_Conv_rz_tmp = builder.add(tmp, _features_features_20_conv_Conv_rz_tmp);
			}
			_features_features_20_conv_Conv_rz[i] = _features_features_20_conv_Conv_rz_tmp;
		}
		let mut _features_features_20_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_20_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..256 {
			let tmp = builder.mul(self.features_20_conv_weight_mat_rv[i], _features_features_20_conv_Conv_rz[i]);
			_features_features_20_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_20_conv_Conv_rrz_tmp);
		}
		_features_features_20_conv_Conv_rrz = _features_features_20_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_20_conv_Conv_rrz, _features_features_20_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_20_Mul_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_20_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_20_conv_Conv_output_0[i][j][k][l], self._features_features_20_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						let tmp1 = builder.mul(self._features_features_20_Div_output_0[i][j][k][l], self._features_features_20_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_20_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_20_Mul_output_0[i][j][k][l], self._features_features_20_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_20_Cast_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_20_Cast_output_0[i][j][k][l] = self._features_features_20_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_20_Cast_1_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_20_Cast_1_output_0[i][j][k][l] = _features_features_20_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_20_Add_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_20_Add_output_0[i][j][k][l] = builder.add(_features_features_20_Cast_1_output_0[i][j][k][l], self._features_features_20_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_22_relu_Cast_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_22_relu_Cast_output_0[i][j][k][l] = _features_features_20_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						let tmp1 = builder.sub(self._features_features_22_relu_Relu_output_0[i][j][k][l], _features_features_22_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_22_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_22_relu_Cast_1_output_0: [[[[Variable;8];8];256];16] = [[[[Default::default();8];8];256];16];
		for i in 0..16 {
			for j in 0..256 {
				for k in 0..8 {
					for l in 0..8 {
						_features_features_22_relu_Cast_1_output_0[i][j][k][l] = self._features_features_22_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// maxpool operation
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(256 + 0 + 0 - 1 + 1)).step_by(1) {
				for k in (0..(8 + 0 + 0 - 2 + 1)).step_by(2) {
					for l in (0..(8 + 0 + 0 - 2 + 1)).step_by(2) {
					let mut tmp = builder.constant(1);
						for m in 0..1 {
							for n in 0..1 {
								for o in 0..2 {
									for p in 0..2 {
									let sub_tmp = builder.sub(self._features_features_23_maxpool_MaxPool_output_0[i/1][j/1][k/2][l/2], _features_features_22_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-0][l+p-0]);
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 256 && (k+o-0) >= 0 && (k+o-0) < 8 && (l+p-0) >= 0 && (l+p-0) < 8 { tmp = builder.mul(tmp, sub_tmp)}
									}
								}
							}
						}
					builder.assert_is_zero(tmp);
					}
				}
			}
		}
		// conv operation
		let mut features_24_conv_weight_mat: [[Variable;512];2304] = [[Default::default();512];2304];
		for i in 0..512 {
			for j in 0..256 {
				for k in 0..3 {
					for l in 0..3 {
						features_24_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_24_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_24_conv_Conv_output_0_mat: [[Variable;512];256] = [[Default::default();512];256];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_24_conv_Conv_output_0_mat[((i)*4 + k)*4 + l][j] = self._features_features_24_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_23_maxpool_MaxPool_output_0_mat: [[Variable;2304];256] = [[Default::default();2304];256];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(256 + 0 + 0 - 256 + 1)).step_by(256) {
				for k in (0..(4 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(4 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..256 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 256 && (k+o-1) >= 0 && (k+o-1) < 4 && (l+p-1) >= 0 && (l+p-1) < 4 { _features_features_23_maxpool_MaxPool_output_0_mat[((i)*4 + k)*4 + l][((n)*3 + o)*3 + p] = self._features_features_23_maxpool_MaxPool_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_23_maxpool_MaxPool_output_0_mat[((i)*4 + k)*4 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_24_conv_Conv_rx: [Variable;2304] = [Default::default();2304];
		let mut _features_features_24_conv_Conv_ry: [Variable;2304] = [Default::default();2304];
		for i in 0..2304 {
			let mut _features_features_24_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..256 {
				let tmp = builder.mul(self._features_features_23_maxpool_MaxPool_output_0_mat_ru[j], _features_features_23_maxpool_MaxPool_output_0_mat[j][i]);
				_features_features_24_conv_Conv_rx_tmp = builder.add(tmp, _features_features_24_conv_Conv_rx_tmp);
			}
			_features_features_24_conv_Conv_rx[i] = _features_features_24_conv_Conv_rx_tmp;
		}
		for i in 0..2304 {
			let mut _features_features_24_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..512 {
				let tmp = builder.mul(self.features_24_conv_weight_mat_rv[j], features_24_conv_weight_mat[i][j]);
				_features_features_24_conv_Conv_ry_tmp = builder.add(tmp, _features_features_24_conv_Conv_ry_tmp);
			}
			_features_features_24_conv_Conv_ry[i] = _features_features_24_conv_Conv_ry_tmp;
		}
		let mut _features_features_24_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_24_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..2304 {
			let tmp = builder.mul(_features_features_24_conv_Conv_ry[i], _features_features_24_conv_Conv_rx[i]);
			_features_features_24_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_24_conv_Conv_rxy_tmp);
		}
		_features_features_24_conv_Conv_rxy = _features_features_24_conv_Conv_rxy_tmp;
		let mut _features_features_24_conv_Conv_rz: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _features_features_24_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..256 {
				let tmp = builder.mul(self._features_features_23_maxpool_MaxPool_output_0_mat_ru[j], _features_features_24_conv_Conv_output_0_mat[j][i]);
				_features_features_24_conv_Conv_rz_tmp = builder.add(tmp, _features_features_24_conv_Conv_rz_tmp);
			}
			_features_features_24_conv_Conv_rz[i] = _features_features_24_conv_Conv_rz_tmp;
		}
		let mut _features_features_24_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_24_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(self.features_24_conv_weight_mat_rv[i], _features_features_24_conv_Conv_rz[i]);
			_features_features_24_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_24_conv_Conv_rrz_tmp);
		}
		_features_features_24_conv_Conv_rrz = _features_features_24_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_24_conv_Conv_rrz, _features_features_24_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_24_Mul_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_24_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_24_conv_Conv_output_0[i][j][k][l], self._features_features_24_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						let tmp1 = builder.mul(self._features_features_24_Div_output_0[i][j][k][l], self._features_features_24_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_24_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_24_Mul_output_0[i][j][k][l], self._features_features_24_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_24_Cast_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_24_Cast_output_0[i][j][k][l] = self._features_features_24_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_24_Cast_1_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_24_Cast_1_output_0[i][j][k][l] = _features_features_24_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_24_Add_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_24_Add_output_0[i][j][k][l] = builder.add(_features_features_24_Cast_1_output_0[i][j][k][l], self._features_features_24_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_26_relu_Cast_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_26_relu_Cast_output_0[i][j][k][l] = _features_features_24_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						let tmp1 = builder.sub(self._features_features_26_relu_Relu_output_0[i][j][k][l], _features_features_26_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_26_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_26_relu_Cast_1_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_26_relu_Cast_1_output_0[i][j][k][l] = self._features_features_26_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// conv operation
		let mut features_27_conv_weight_mat: [[Variable;512];4608] = [[Default::default();512];4608];
		for i in 0..512 {
			for j in 0..512 {
				for k in 0..3 {
					for l in 0..3 {
						features_27_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_27_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_27_conv_Conv_output_0_mat: [[Variable;512];256] = [[Default::default();512];256];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_27_conv_Conv_output_0_mat[((i)*4 + k)*4 + l][j] = self._features_features_27_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_26_relu_Cast_1_output_0_mat: [[Variable;4608];256] = [[Default::default();4608];256];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(512 + 0 + 0 - 512 + 1)).step_by(512) {
				for k in (0..(4 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(4 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..512 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 512 && (k+o-1) >= 0 && (k+o-1) < 4 && (l+p-1) >= 0 && (l+p-1) < 4 { _features_features_26_relu_Cast_1_output_0_mat[((i)*4 + k)*4 + l][((n)*3 + o)*3 + p] = _features_features_26_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_26_relu_Cast_1_output_0_mat[((i)*4 + k)*4 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_27_conv_Conv_rx: [Variable;4608] = [Default::default();4608];
		let mut _features_features_27_conv_Conv_ry: [Variable;4608] = [Default::default();4608];
		for i in 0..4608 {
			let mut _features_features_27_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..256 {
				let tmp = builder.mul(self._features_features_26_relu_Cast_1_output_0_mat_ru[j], _features_features_26_relu_Cast_1_output_0_mat[j][i]);
				_features_features_27_conv_Conv_rx_tmp = builder.add(tmp, _features_features_27_conv_Conv_rx_tmp);
			}
			_features_features_27_conv_Conv_rx[i] = _features_features_27_conv_Conv_rx_tmp;
		}
		for i in 0..4608 {
			let mut _features_features_27_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..512 {
				let tmp = builder.mul(self.features_27_conv_weight_mat_rv[j], features_27_conv_weight_mat[i][j]);
				_features_features_27_conv_Conv_ry_tmp = builder.add(tmp, _features_features_27_conv_Conv_ry_tmp);
			}
			_features_features_27_conv_Conv_ry[i] = _features_features_27_conv_Conv_ry_tmp;
		}
		let mut _features_features_27_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_27_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..4608 {
			let tmp = builder.mul(_features_features_27_conv_Conv_ry[i], _features_features_27_conv_Conv_rx[i]);
			_features_features_27_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_27_conv_Conv_rxy_tmp);
		}
		_features_features_27_conv_Conv_rxy = _features_features_27_conv_Conv_rxy_tmp;
		let mut _features_features_27_conv_Conv_rz: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _features_features_27_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..256 {
				let tmp = builder.mul(self._features_features_26_relu_Cast_1_output_0_mat_ru[j], _features_features_27_conv_Conv_output_0_mat[j][i]);
				_features_features_27_conv_Conv_rz_tmp = builder.add(tmp, _features_features_27_conv_Conv_rz_tmp);
			}
			_features_features_27_conv_Conv_rz[i] = _features_features_27_conv_Conv_rz_tmp;
		}
		let mut _features_features_27_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_27_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(self.features_27_conv_weight_mat_rv[i], _features_features_27_conv_Conv_rz[i]);
			_features_features_27_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_27_conv_Conv_rrz_tmp);
		}
		_features_features_27_conv_Conv_rrz = _features_features_27_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_27_conv_Conv_rrz, _features_features_27_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_27_Mul_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_27_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_27_conv_Conv_output_0[i][j][k][l], self._features_features_27_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						let tmp1 = builder.mul(self._features_features_27_Div_output_0[i][j][k][l], self._features_features_27_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_27_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_27_Mul_output_0[i][j][k][l], self._features_features_27_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_27_Cast_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_27_Cast_output_0[i][j][k][l] = self._features_features_27_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_27_Cast_1_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_27_Cast_1_output_0[i][j][k][l] = _features_features_27_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_27_Add_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_27_Add_output_0[i][j][k][l] = builder.add(_features_features_27_Cast_1_output_0[i][j][k][l], self._features_features_27_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_29_relu_Cast_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_29_relu_Cast_output_0[i][j][k][l] = _features_features_27_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						let tmp1 = builder.sub(self._features_features_29_relu_Relu_output_0[i][j][k][l], _features_features_29_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_29_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_29_relu_Cast_1_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_29_relu_Cast_1_output_0[i][j][k][l] = self._features_features_29_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// conv operation
		let mut features_30_conv_weight_mat: [[Variable;512];4608] = [[Default::default();512];4608];
		for i in 0..512 {
			for j in 0..512 {
				for k in 0..3 {
					for l in 0..3 {
						features_30_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_30_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_30_conv_Conv_output_0_mat: [[Variable;512];256] = [[Default::default();512];256];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_30_conv_Conv_output_0_mat[((i)*4 + k)*4 + l][j] = self._features_features_30_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_29_relu_Cast_1_output_0_mat: [[Variable;4608];256] = [[Default::default();4608];256];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(512 + 0 + 0 - 512 + 1)).step_by(512) {
				for k in (0..(4 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(4 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..512 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 512 && (k+o-1) >= 0 && (k+o-1) < 4 && (l+p-1) >= 0 && (l+p-1) < 4 { _features_features_29_relu_Cast_1_output_0_mat[((i)*4 + k)*4 + l][((n)*3 + o)*3 + p] = _features_features_29_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_29_relu_Cast_1_output_0_mat[((i)*4 + k)*4 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_30_conv_Conv_rx: [Variable;4608] = [Default::default();4608];
		let mut _features_features_30_conv_Conv_ry: [Variable;4608] = [Default::default();4608];
		for i in 0..4608 {
			let mut _features_features_30_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..256 {
				let tmp = builder.mul(self._features_features_29_relu_Cast_1_output_0_mat_ru[j], _features_features_29_relu_Cast_1_output_0_mat[j][i]);
				_features_features_30_conv_Conv_rx_tmp = builder.add(tmp, _features_features_30_conv_Conv_rx_tmp);
			}
			_features_features_30_conv_Conv_rx[i] = _features_features_30_conv_Conv_rx_tmp;
		}
		for i in 0..4608 {
			let mut _features_features_30_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..512 {
				let tmp = builder.mul(self.features_30_conv_weight_mat_rv[j], features_30_conv_weight_mat[i][j]);
				_features_features_30_conv_Conv_ry_tmp = builder.add(tmp, _features_features_30_conv_Conv_ry_tmp);
			}
			_features_features_30_conv_Conv_ry[i] = _features_features_30_conv_Conv_ry_tmp;
		}
		let mut _features_features_30_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_30_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..4608 {
			let tmp = builder.mul(_features_features_30_conv_Conv_ry[i], _features_features_30_conv_Conv_rx[i]);
			_features_features_30_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_30_conv_Conv_rxy_tmp);
		}
		_features_features_30_conv_Conv_rxy = _features_features_30_conv_Conv_rxy_tmp;
		let mut _features_features_30_conv_Conv_rz: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _features_features_30_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..256 {
				let tmp = builder.mul(self._features_features_29_relu_Cast_1_output_0_mat_ru[j], _features_features_30_conv_Conv_output_0_mat[j][i]);
				_features_features_30_conv_Conv_rz_tmp = builder.add(tmp, _features_features_30_conv_Conv_rz_tmp);
			}
			_features_features_30_conv_Conv_rz[i] = _features_features_30_conv_Conv_rz_tmp;
		}
		let mut _features_features_30_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_30_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(self.features_30_conv_weight_mat_rv[i], _features_features_30_conv_Conv_rz[i]);
			_features_features_30_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_30_conv_Conv_rrz_tmp);
		}
		_features_features_30_conv_Conv_rrz = _features_features_30_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_30_conv_Conv_rrz, _features_features_30_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_30_Mul_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_30_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_30_conv_Conv_output_0[i][j][k][l], self._features_features_30_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						let tmp1 = builder.mul(self._features_features_30_Div_output_0[i][j][k][l], self._features_features_30_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_30_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_30_Mul_output_0[i][j][k][l], self._features_features_30_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_30_Cast_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_30_Cast_output_0[i][j][k][l] = self._features_features_30_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_30_Cast_1_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_30_Cast_1_output_0[i][j][k][l] = _features_features_30_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_30_Add_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_30_Add_output_0[i][j][k][l] = builder.add(_features_features_30_Cast_1_output_0[i][j][k][l], self._features_features_30_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_32_relu_Cast_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_32_relu_Cast_output_0[i][j][k][l] = _features_features_30_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						let tmp1 = builder.sub(self._features_features_32_relu_Relu_output_0[i][j][k][l], _features_features_32_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_32_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_32_relu_Cast_1_output_0: [[[[Variable;4];4];512];16] = [[[[Default::default();4];4];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..4 {
					for l in 0..4 {
						_features_features_32_relu_Cast_1_output_0[i][j][k][l] = self._features_features_32_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// maxpool operation
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(512 + 0 + 0 - 1 + 1)).step_by(1) {
				for k in (0..(4 + 0 + 0 - 2 + 1)).step_by(2) {
					for l in (0..(4 + 0 + 0 - 2 + 1)).step_by(2) {
					let mut tmp = builder.constant(1);
						for m in 0..1 {
							for n in 0..1 {
								for o in 0..2 {
									for p in 0..2 {
									let sub_tmp = builder.sub(self._features_features_33_maxpool_MaxPool_output_0[i/1][j/1][k/2][l/2], _features_features_32_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-0][l+p-0]);
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 512 && (k+o-0) >= 0 && (k+o-0) < 4 && (l+p-0) >= 0 && (l+p-0) < 4 { tmp = builder.mul(tmp, sub_tmp)}
									}
								}
							}
						}
					builder.assert_is_zero(tmp);
					}
				}
			}
		}
		// conv operation
		let mut features_34_conv_weight_mat: [[Variable;512];4608] = [[Default::default();512];4608];
		for i in 0..512 {
			for j in 0..512 {
				for k in 0..3 {
					for l in 0..3 {
						features_34_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_34_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_34_conv_Conv_output_0_mat: [[Variable;512];64] = [[Default::default();512];64];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_34_conv_Conv_output_0_mat[((i)*2 + k)*2 + l][j] = self._features_features_34_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_33_maxpool_MaxPool_output_0_mat: [[Variable;4608];64] = [[Default::default();4608];64];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(512 + 0 + 0 - 512 + 1)).step_by(512) {
				for k in (0..(2 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(2 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..512 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 512 && (k+o-1) >= 0 && (k+o-1) < 2 && (l+p-1) >= 0 && (l+p-1) < 2 { _features_features_33_maxpool_MaxPool_output_0_mat[((i)*2 + k)*2 + l][((n)*3 + o)*3 + p] = self._features_features_33_maxpool_MaxPool_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_33_maxpool_MaxPool_output_0_mat[((i)*2 + k)*2 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_34_conv_Conv_rx: [Variable;4608] = [Default::default();4608];
		let mut _features_features_34_conv_Conv_ry: [Variable;4608] = [Default::default();4608];
		for i in 0..4608 {
			let mut _features_features_34_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..64 {
				let tmp = builder.mul(self._features_features_33_maxpool_MaxPool_output_0_mat_ru[j], _features_features_33_maxpool_MaxPool_output_0_mat[j][i]);
				_features_features_34_conv_Conv_rx_tmp = builder.add(tmp, _features_features_34_conv_Conv_rx_tmp);
			}
			_features_features_34_conv_Conv_rx[i] = _features_features_34_conv_Conv_rx_tmp;
		}
		for i in 0..4608 {
			let mut _features_features_34_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..512 {
				let tmp = builder.mul(self.features_34_conv_weight_mat_rv[j], features_34_conv_weight_mat[i][j]);
				_features_features_34_conv_Conv_ry_tmp = builder.add(tmp, _features_features_34_conv_Conv_ry_tmp);
			}
			_features_features_34_conv_Conv_ry[i] = _features_features_34_conv_Conv_ry_tmp;
		}
		let mut _features_features_34_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_34_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..4608 {
			let tmp = builder.mul(_features_features_34_conv_Conv_ry[i], _features_features_34_conv_Conv_rx[i]);
			_features_features_34_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_34_conv_Conv_rxy_tmp);
		}
		_features_features_34_conv_Conv_rxy = _features_features_34_conv_Conv_rxy_tmp;
		let mut _features_features_34_conv_Conv_rz: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _features_features_34_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..64 {
				let tmp = builder.mul(self._features_features_33_maxpool_MaxPool_output_0_mat_ru[j], _features_features_34_conv_Conv_output_0_mat[j][i]);
				_features_features_34_conv_Conv_rz_tmp = builder.add(tmp, _features_features_34_conv_Conv_rz_tmp);
			}
			_features_features_34_conv_Conv_rz[i] = _features_features_34_conv_Conv_rz_tmp;
		}
		let mut _features_features_34_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_34_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(self.features_34_conv_weight_mat_rv[i], _features_features_34_conv_Conv_rz[i]);
			_features_features_34_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_34_conv_Conv_rrz_tmp);
		}
		_features_features_34_conv_Conv_rrz = _features_features_34_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_34_conv_Conv_rrz, _features_features_34_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_34_Mul_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_34_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_34_conv_Conv_output_0[i][j][k][l], self._features_features_34_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						let tmp1 = builder.mul(self._features_features_34_Div_output_0[i][j][k][l], self._features_features_34_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_34_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_34_Mul_output_0[i][j][k][l], self._features_features_34_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_34_Cast_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_34_Cast_output_0[i][j][k][l] = self._features_features_34_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_34_Cast_1_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_34_Cast_1_output_0[i][j][k][l] = _features_features_34_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_34_Add_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_34_Add_output_0[i][j][k][l] = builder.add(_features_features_34_Cast_1_output_0[i][j][k][l], self._features_features_34_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_36_relu_Cast_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_36_relu_Cast_output_0[i][j][k][l] = _features_features_34_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						let tmp1 = builder.sub(self._features_features_36_relu_Relu_output_0[i][j][k][l], _features_features_36_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_36_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_36_relu_Cast_1_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_36_relu_Cast_1_output_0[i][j][k][l] = self._features_features_36_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// conv operation
		let mut features_37_conv_weight_mat: [[Variable;512];4608] = [[Default::default();512];4608];
		for i in 0..512 {
			for j in 0..512 {
				for k in 0..3 {
					for l in 0..3 {
						features_37_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_37_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_37_conv_Conv_output_0_mat: [[Variable;512];64] = [[Default::default();512];64];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_37_conv_Conv_output_0_mat[((i)*2 + k)*2 + l][j] = self._features_features_37_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_36_relu_Cast_1_output_0_mat: [[Variable;4608];64] = [[Default::default();4608];64];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(512 + 0 + 0 - 512 + 1)).step_by(512) {
				for k in (0..(2 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(2 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..512 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 512 && (k+o-1) >= 0 && (k+o-1) < 2 && (l+p-1) >= 0 && (l+p-1) < 2 { _features_features_36_relu_Cast_1_output_0_mat[((i)*2 + k)*2 + l][((n)*3 + o)*3 + p] = _features_features_36_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_36_relu_Cast_1_output_0_mat[((i)*2 + k)*2 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_37_conv_Conv_rx: [Variable;4608] = [Default::default();4608];
		let mut _features_features_37_conv_Conv_ry: [Variable;4608] = [Default::default();4608];
		for i in 0..4608 {
			let mut _features_features_37_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..64 {
				let tmp = builder.mul(self._features_features_36_relu_Cast_1_output_0_mat_ru[j], _features_features_36_relu_Cast_1_output_0_mat[j][i]);
				_features_features_37_conv_Conv_rx_tmp = builder.add(tmp, _features_features_37_conv_Conv_rx_tmp);
			}
			_features_features_37_conv_Conv_rx[i] = _features_features_37_conv_Conv_rx_tmp;
		}
		for i in 0..4608 {
			let mut _features_features_37_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..512 {
				let tmp = builder.mul(self.features_37_conv_weight_mat_rv[j], features_37_conv_weight_mat[i][j]);
				_features_features_37_conv_Conv_ry_tmp = builder.add(tmp, _features_features_37_conv_Conv_ry_tmp);
			}
			_features_features_37_conv_Conv_ry[i] = _features_features_37_conv_Conv_ry_tmp;
		}
		let mut _features_features_37_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_37_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..4608 {
			let tmp = builder.mul(_features_features_37_conv_Conv_ry[i], _features_features_37_conv_Conv_rx[i]);
			_features_features_37_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_37_conv_Conv_rxy_tmp);
		}
		_features_features_37_conv_Conv_rxy = _features_features_37_conv_Conv_rxy_tmp;
		let mut _features_features_37_conv_Conv_rz: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _features_features_37_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..64 {
				let tmp = builder.mul(self._features_features_36_relu_Cast_1_output_0_mat_ru[j], _features_features_37_conv_Conv_output_0_mat[j][i]);
				_features_features_37_conv_Conv_rz_tmp = builder.add(tmp, _features_features_37_conv_Conv_rz_tmp);
			}
			_features_features_37_conv_Conv_rz[i] = _features_features_37_conv_Conv_rz_tmp;
		}
		let mut _features_features_37_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_37_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(self.features_37_conv_weight_mat_rv[i], _features_features_37_conv_Conv_rz[i]);
			_features_features_37_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_37_conv_Conv_rrz_tmp);
		}
		_features_features_37_conv_Conv_rrz = _features_features_37_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_37_conv_Conv_rrz, _features_features_37_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_37_Mul_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_37_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_37_conv_Conv_output_0[i][j][k][l], self._features_features_37_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						let tmp1 = builder.mul(self._features_features_37_Div_output_0[i][j][k][l], self._features_features_37_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_37_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_37_Mul_output_0[i][j][k][l], self._features_features_37_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_37_Cast_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_37_Cast_output_0[i][j][k][l] = self._features_features_37_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_37_Cast_1_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_37_Cast_1_output_0[i][j][k][l] = _features_features_37_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_37_Add_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_37_Add_output_0[i][j][k][l] = builder.add(_features_features_37_Cast_1_output_0[i][j][k][l], self._features_features_37_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_39_relu_Cast_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_39_relu_Cast_output_0[i][j][k][l] = _features_features_37_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						let tmp1 = builder.sub(self._features_features_39_relu_Relu_output_0[i][j][k][l], _features_features_39_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_39_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_39_relu_Cast_1_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_39_relu_Cast_1_output_0[i][j][k][l] = self._features_features_39_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// conv operation
		let mut features_40_conv_weight_mat: [[Variable;512];4608] = [[Default::default();512];4608];
		for i in 0..512 {
			for j in 0..512 {
				for k in 0..3 {
					for l in 0..3 {
						features_40_conv_weight_mat[((j)*3 + k)*3 + l][i] = self.features_40_conv_weight[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_40_conv_Conv_output_0_mat: [[Variable;512];64] = [[Default::default();512];64];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_40_conv_Conv_output_0_mat[((i)*2 + k)*2 + l][j] = self._features_features_40_conv_Conv_output_0[i][j][k][l];
					}
				}
			}
		}
		let mut _features_features_39_relu_Cast_1_output_0_mat: [[Variable;4608];64] = [[Default::default();4608];64];
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(512 + 0 + 0 - 512 + 1)).step_by(512) {
				for k in (0..(2 + 1 + 1 - 3 + 1)).step_by(1) {
					for l in (0..(2 + 1 + 1 - 3 + 1)).step_by(1) {
						for m in 0..1 {
							for n in 0..512 {
								for o in 0..3 {
									for p in 0..3 {
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 512 && (k+o-1) >= 0 && (k+o-1) < 2 && (l+p-1) >= 0 && (l+p-1) < 2 { _features_features_39_relu_Cast_1_output_0_mat[((i)*2 + k)*2 + l][((n)*3 + o)*3 + p] = _features_features_39_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-1][l+p-1]}
									else { _features_features_39_relu_Cast_1_output_0_mat[((i)*2 + k)*2 + l][((n)*3 + o)*3 + p] = builder.constant(0)}; 
									}
								}
							}
						}
					}
				}
			}
		}
		let mut _features_features_40_conv_Conv_rx: [Variable;4608] = [Default::default();4608];
		let mut _features_features_40_conv_Conv_ry: [Variable;4608] = [Default::default();4608];
		for i in 0..4608 {
			let mut _features_features_40_conv_Conv_rx_tmp = builder.constant(0);
			for j in 0..64 {
				let tmp = builder.mul(self._features_features_39_relu_Cast_1_output_0_mat_ru[j], _features_features_39_relu_Cast_1_output_0_mat[j][i]);
				_features_features_40_conv_Conv_rx_tmp = builder.add(tmp, _features_features_40_conv_Conv_rx_tmp);
			}
			_features_features_40_conv_Conv_rx[i] = _features_features_40_conv_Conv_rx_tmp;
		}
		for i in 0..4608 {
			let mut _features_features_40_conv_Conv_ry_tmp = builder.constant(0);
			for j in 0..512 {
				let tmp = builder.mul(self.features_40_conv_weight_mat_rv[j], features_40_conv_weight_mat[i][j]);
				_features_features_40_conv_Conv_ry_tmp = builder.add(tmp, _features_features_40_conv_Conv_ry_tmp);
			}
			_features_features_40_conv_Conv_ry[i] = _features_features_40_conv_Conv_ry_tmp;
		}
		let mut _features_features_40_conv_Conv_rxy: Variable = Default::default();
		let mut _features_features_40_conv_Conv_rxy_tmp = builder.constant(0);
		for i in 0..4608 {
			let tmp = builder.mul(_features_features_40_conv_Conv_ry[i], _features_features_40_conv_Conv_rx[i]);
			_features_features_40_conv_Conv_rxy_tmp = builder.add(tmp, _features_features_40_conv_Conv_rxy_tmp);
		}
		_features_features_40_conv_Conv_rxy = _features_features_40_conv_Conv_rxy_tmp;
		let mut _features_features_40_conv_Conv_rz: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _features_features_40_conv_Conv_rz_tmp = builder.constant(0);
			for j in 0..64 {
				let tmp = builder.mul(self._features_features_39_relu_Cast_1_output_0_mat_ru[j], _features_features_40_conv_Conv_output_0_mat[j][i]);
				_features_features_40_conv_Conv_rz_tmp = builder.add(tmp, _features_features_40_conv_Conv_rz_tmp);
			}
			_features_features_40_conv_Conv_rz[i] = _features_features_40_conv_Conv_rz_tmp;
		}
		let mut _features_features_40_conv_Conv_rrz: Variable = Default::default();
		let mut _features_features_40_conv_Conv_rrz_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(self.features_40_conv_weight_mat_rv[i], _features_features_40_conv_Conv_rz[i]);
			_features_features_40_conv_Conv_rrz_tmp = builder.add(tmp, _features_features_40_conv_Conv_rrz_tmp);
		}
		_features_features_40_conv_Conv_rrz = _features_features_40_conv_Conv_rrz_tmp;
		builder.assert_is_equal(_features_features_40_conv_Conv_rrz, _features_features_40_conv_Conv_rxy);

		// constant operation
		// multiply operation
		let mut _features_features_40_Mul_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_40_Mul_output_0[i][j][k][l] = builder.mul(self._features_features_40_conv_Conv_output_0[i][j][k][l], self._features_features_40_Constant_output_0);
					}
				}
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						let tmp1 = builder.mul(self._features_features_40_Div_output_0[i][j][k][l], self._features_features_40_Constant_1_output_0);
						table.rangeproof(builder, self._features_features_40_Div_output_0_r[i][j][k][l], 24);
						let tmp2 = builder.sub(_features_features_40_Mul_output_0[i][j][k][l], self._features_features_40_Div_output_0_r[i][j][k][l]);
						builder.assert_is_equal(tmp1, tmp2);
											}
				}
			}
		}
		// cast operation
		let mut _features_features_40_Cast_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_40_Cast_output_0[i][j][k][l] = self._features_features_40_Div_output_0[i][j][k][l];
					}
				}
			}
		}
		// cast operation
		let mut _features_features_40_Cast_1_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_40_Cast_1_output_0[i][j][k][l] = _features_features_40_Cast_output_0[i][j][k][l];
					}
				}
			}
		}
		// constant operation
		// add operation
		let mut _features_features_40_Add_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_40_Add_output_0[i][j][k][l] = builder.add(_features_features_40_Cast_1_output_0[i][j][k][l], self._features_features_40_Constant_2_output_0[j][k][l]);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_42_relu_Cast_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_42_relu_Cast_output_0[i][j][k][l] = _features_features_40_Add_output_0[i][j][k][l];
					}
				}
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						let tmp1 = builder.sub(self._features_features_42_relu_Relu_output_0[i][j][k][l], _features_features_42_relu_Cast_output_0[i][j][k][l]);
						let tmp2 = builder.mul(tmp1, self._features_features_42_relu_Relu_output_0[i][j][k][l]);
						builder.assert_is_zero(tmp2);
					}
				}
			}
		}
		// cast operation
		let mut _features_features_42_relu_Cast_1_output_0: [[[[Variable;2];2];512];16] = [[[[Default::default();2];2];512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..2 {
					for l in 0..2 {
						_features_features_42_relu_Cast_1_output_0[i][j][k][l] = self._features_features_42_relu_Relu_output_0[i][j][k][l];
					}
				}
			}
		}
		// maxpool operation
		for i in (0..(16 + 0 + 0 - 1 + 1)).step_by(1) {
			for j in (0..(512 + 0 + 0 - 1 + 1)).step_by(1) {
				for k in (0..(2 + 0 + 0 - 2 + 1)).step_by(2) {
					for l in (0..(2 + 0 + 0 - 2 + 1)).step_by(2) {
					let mut tmp = builder.constant(1);
						for m in 0..1 {
							for n in 0..1 {
								for o in 0..2 {
									for p in 0..2 {
									let sub_tmp = builder.sub(self._features_features_43_maxpool_MaxPool_output_0[i/1][j/1][k/2][l/2], _features_features_42_relu_Cast_1_output_0[i+m-0][j+n-0][k+o-0][l+p-0]);
									if true && (i+m-0) >= 0 && (i+m-0) < 16 && (j+n-0) >= 0 && (j+n-0) < 512 && (k+o-0) >= 0 && (k+o-0) < 2 && (l+p-0) >= 0 && (l+p-0) < 2 { tmp = builder.mul(tmp, sub_tmp)}
									}
								}
							}
						}
					builder.assert_is_zero(tmp);
					}
				}
			}
		}
		// flatten operation
		let mut _Flatten_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				for k in 0..1 {
					for l in 0..1 {
						_Flatten_output_0[i][((j)*1 + k)*1 + l] = self._features_features_43_maxpool_MaxPool_output_0[i][j][k][l];
					}
				}
			}
		}
		// matmul operation
		let mut _classifier_classifier_0_linear_MatMul_rx: [Variable;512] = [Default::default();512];
		let mut _classifier_classifier_0_linear_MatMul_ry: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _classifier_classifier_0_linear_MatMul_rx_tmp = builder.constant(0);
			for j in 0..16 {
				let tmp = builder.mul(self._Flatten_output_0_mat_ru[j], _Flatten_output_0[j][i]);
				_classifier_classifier_0_linear_MatMul_rx_tmp = builder.add(tmp, _classifier_classifier_0_linear_MatMul_rx_tmp);
			}
			_classifier_classifier_0_linear_MatMul_rx[i] = _classifier_classifier_0_linear_MatMul_rx_tmp;
		}
		for i in 0..512 {
			let mut _classifier_classifier_0_linear_MatMul_ry_tmp = builder.constant(0);
			for j in 0..512 {
				let tmp = builder.mul(self.onnx__MatMul_215_mat_rv[j], self.onnx__MatMul_215[i][j]);
				_classifier_classifier_0_linear_MatMul_ry_tmp = builder.add(tmp, _classifier_classifier_0_linear_MatMul_ry_tmp);
			}
			_classifier_classifier_0_linear_MatMul_ry[i] = _classifier_classifier_0_linear_MatMul_ry_tmp;
		}
		let mut _classifier_classifier_0_linear_MatMul_rxy: Variable = Default::default();
		let mut _classifier_classifier_0_linear_MatMul_rxy_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(_classifier_classifier_0_linear_MatMul_ry[i], _classifier_classifier_0_linear_MatMul_rx[i]);
			_classifier_classifier_0_linear_MatMul_rxy_tmp = builder.add(tmp, _classifier_classifier_0_linear_MatMul_rxy_tmp);
			}
				_classifier_classifier_0_linear_MatMul_rxy = _classifier_classifier_0_linear_MatMul_rxy_tmp;
		let mut _classifier_classifier_0_linear_MatMul_rz: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _classifier_classifier_0_linear_MatMul_rz_tmp = builder.constant(0);
			for j in 0..16 {
				let tmp = builder.mul(self._Flatten_output_0_mat_ru[j], self._classifier_classifier_0_linear_MatMul_output_0[j][i]);
				_classifier_classifier_0_linear_MatMul_rz_tmp = builder.add(tmp, _classifier_classifier_0_linear_MatMul_rz_tmp);
			}
			_classifier_classifier_0_linear_MatMul_rz[i] = _classifier_classifier_0_linear_MatMul_rz_tmp;
		}
		let mut _classifier_classifier_0_linear_MatMul_rrz: Variable = Default::default();
		let mut _classifier_classifier_0_linear_MatMul_rrz_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(self.onnx__MatMul_215_mat_rv[i], _classifier_classifier_0_linear_MatMul_rz[i]);
			_classifier_classifier_0_linear_MatMul_rrz_tmp = builder.add(tmp, _classifier_classifier_0_linear_MatMul_rrz_tmp);
			}
		_classifier_classifier_0_linear_MatMul_rrz = _classifier_classifier_0_linear_MatMul_rrz_tmp;
		builder.assert_is_equal(_classifier_classifier_0_linear_MatMul_rrz, _classifier_classifier_0_linear_MatMul_rxy);
		// constant operation
		// multiply operation
		let mut _classifier_classifier_0_Mul_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_0_Mul_output_0[i][j] = builder.mul(self._classifier_classifier_0_linear_MatMul_output_0[i][j], self._classifier_classifier_0_Constant_output_0);
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..512 {
				let tmp1 = builder.mul(self._classifier_classifier_0_Div_output_0[i][j], self._classifier_classifier_0_Constant_1_output_0);
				table.rangeproof(builder, self._classifier_classifier_0_Div_output_0_r[i][j], 24);
				let tmp2 = builder.sub(_classifier_classifier_0_Mul_output_0[i][j], self._classifier_classifier_0_Div_output_0_r[i][j]);
				builder.assert_is_equal(tmp1, tmp2);
							}
		}
		// cast operation
		let mut _classifier_classifier_0_Cast_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_0_Cast_output_0[i][j] = self._classifier_classifier_0_Div_output_0[i][j];
			}
		}
		// cast operation
		let mut _classifier_classifier_0_Cast_1_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_0_Cast_1_output_0[i][j] = _classifier_classifier_0_Cast_output_0[i][j];
			}
		}
		// constant operation
		// add operation
		let mut _classifier_classifier_0_Add_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_0_Add_output_0[i][j] = builder.add(_classifier_classifier_0_Cast_1_output_0[i][j], self._classifier_classifier_0_Constant_2_output_0[j]);
			}
		}
		// cast operation
		let mut _classifier_classifier_1_relu_Cast_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_1_relu_Cast_output_0[i][j] = _classifier_classifier_0_Add_output_0[i][j];
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..512 {
				let tmp1 = builder.sub(self._classifier_classifier_1_relu_Relu_output_0[i][j], _classifier_classifier_1_relu_Cast_output_0[i][j]);
				let tmp2 = builder.mul(tmp1, self._classifier_classifier_1_relu_Relu_output_0[i][j]);
				builder.assert_is_zero(tmp2);
			}
		}
		// cast operation
		let mut _classifier_classifier_1_relu_Cast_1_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_1_relu_Cast_1_output_0[i][j] = self._classifier_classifier_1_relu_Relu_output_0[i][j];
			}
		}
		// matmul operation
		let mut _classifier_classifier_3_linear_MatMul_rx: [Variable;512] = [Default::default();512];
		let mut _classifier_classifier_3_linear_MatMul_ry: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _classifier_classifier_3_linear_MatMul_rx_tmp = builder.constant(0);
			for j in 0..16 {
				let tmp = builder.mul(self._classifier_classifier_1_relu_Cast_1_output_0_mat_ru[j], _classifier_classifier_1_relu_Cast_1_output_0[j][i]);
				_classifier_classifier_3_linear_MatMul_rx_tmp = builder.add(tmp, _classifier_classifier_3_linear_MatMul_rx_tmp);
			}
			_classifier_classifier_3_linear_MatMul_rx[i] = _classifier_classifier_3_linear_MatMul_rx_tmp;
		}
		for i in 0..512 {
			let mut _classifier_classifier_3_linear_MatMul_ry_tmp = builder.constant(0);
			for j in 0..512 {
				let tmp = builder.mul(self.onnx__MatMul_216_mat_rv[j], self.onnx__MatMul_216[i][j]);
				_classifier_classifier_3_linear_MatMul_ry_tmp = builder.add(tmp, _classifier_classifier_3_linear_MatMul_ry_tmp);
			}
			_classifier_classifier_3_linear_MatMul_ry[i] = _classifier_classifier_3_linear_MatMul_ry_tmp;
		}
		let mut _classifier_classifier_3_linear_MatMul_rxy: Variable = Default::default();
		let mut _classifier_classifier_3_linear_MatMul_rxy_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(_classifier_classifier_3_linear_MatMul_ry[i], _classifier_classifier_3_linear_MatMul_rx[i]);
			_classifier_classifier_3_linear_MatMul_rxy_tmp = builder.add(tmp, _classifier_classifier_3_linear_MatMul_rxy_tmp);
			}
				_classifier_classifier_3_linear_MatMul_rxy = _classifier_classifier_3_linear_MatMul_rxy_tmp;
		let mut _classifier_classifier_3_linear_MatMul_rz: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _classifier_classifier_3_linear_MatMul_rz_tmp = builder.constant(0);
			for j in 0..16 {
				let tmp = builder.mul(self._classifier_classifier_1_relu_Cast_1_output_0_mat_ru[j], self._classifier_classifier_3_linear_MatMul_output_0[j][i]);
				_classifier_classifier_3_linear_MatMul_rz_tmp = builder.add(tmp, _classifier_classifier_3_linear_MatMul_rz_tmp);
			}
			_classifier_classifier_3_linear_MatMul_rz[i] = _classifier_classifier_3_linear_MatMul_rz_tmp;
		}
		let mut _classifier_classifier_3_linear_MatMul_rrz: Variable = Default::default();
		let mut _classifier_classifier_3_linear_MatMul_rrz_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(self.onnx__MatMul_216_mat_rv[i], _classifier_classifier_3_linear_MatMul_rz[i]);
			_classifier_classifier_3_linear_MatMul_rrz_tmp = builder.add(tmp, _classifier_classifier_3_linear_MatMul_rrz_tmp);
			}
		_classifier_classifier_3_linear_MatMul_rrz = _classifier_classifier_3_linear_MatMul_rrz_tmp;
		builder.assert_is_equal(_classifier_classifier_3_linear_MatMul_rrz, _classifier_classifier_3_linear_MatMul_rxy);
		// constant operation
		// multiply operation
		let mut _classifier_classifier_3_Mul_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_3_Mul_output_0[i][j] = builder.mul(self._classifier_classifier_3_linear_MatMul_output_0[i][j], self._classifier_classifier_3_Constant_output_0);
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..512 {
				let tmp1 = builder.mul(self._classifier_classifier_3_Div_output_0[i][j], self._classifier_classifier_3_Constant_1_output_0);
				table.rangeproof(builder, self._classifier_classifier_3_Div_output_0_r[i][j], 24);
				let tmp2 = builder.sub(_classifier_classifier_3_Mul_output_0[i][j], self._classifier_classifier_3_Div_output_0_r[i][j]);
				builder.assert_is_equal(tmp1, tmp2);
							}
		}
		// cast operation
		let mut _classifier_classifier_3_Cast_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_3_Cast_output_0[i][j] = self._classifier_classifier_3_Div_output_0[i][j];
			}
		}
		// cast operation
		let mut _classifier_classifier_3_Cast_1_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_3_Cast_1_output_0[i][j] = _classifier_classifier_3_Cast_output_0[i][j];
			}
		}
		// constant operation
		// add operation
		let mut _classifier_classifier_3_Add_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_3_Add_output_0[i][j] = builder.add(_classifier_classifier_3_Cast_1_output_0[i][j], self._classifier_classifier_3_Constant_2_output_0[j]);
			}
		}
		// cast operation
		let mut _classifier_classifier_4_relu_Cast_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_4_relu_Cast_output_0[i][j] = _classifier_classifier_3_Add_output_0[i][j];
			}
		}
		// relu operation
		for i in 0..16 {
			for j in 0..512 {
				let tmp1 = builder.sub(self._classifier_classifier_4_relu_Relu_output_0[i][j], _classifier_classifier_4_relu_Cast_output_0[i][j]);
				let tmp2 = builder.mul(tmp1, self._classifier_classifier_4_relu_Relu_output_0[i][j]);
				builder.assert_is_zero(tmp2);
			}
		}
		// cast operation
		let mut _classifier_classifier_4_relu_Cast_1_output_0: [[Variable;512];16] = [[Default::default();512];16];
		for i in 0..16 {
			for j in 0..512 {
				_classifier_classifier_4_relu_Cast_1_output_0[i][j] = self._classifier_classifier_4_relu_Relu_output_0[i][j];
			}
		}
		// matmul operation
		let mut _classifier_classifier_6_linear_MatMul_rx: [Variable;512] = [Default::default();512];
		let mut _classifier_classifier_6_linear_MatMul_ry: [Variable;512] = [Default::default();512];
		for i in 0..512 {
			let mut _classifier_classifier_6_linear_MatMul_rx_tmp = builder.constant(0);
			for j in 0..16 {
				let tmp = builder.mul(self._classifier_classifier_4_relu_Cast_1_output_0_mat_ru[j], _classifier_classifier_4_relu_Cast_1_output_0[j][i]);
				_classifier_classifier_6_linear_MatMul_rx_tmp = builder.add(tmp, _classifier_classifier_6_linear_MatMul_rx_tmp);
			}
			_classifier_classifier_6_linear_MatMul_rx[i] = _classifier_classifier_6_linear_MatMul_rx_tmp;
		}
		for i in 0..512 {
			let mut _classifier_classifier_6_linear_MatMul_ry_tmp = builder.constant(0);
			for j in 0..10 {
				let tmp = builder.mul(self.onnx__MatMul_217_mat_rv[j], self.onnx__MatMul_217[i][j]);
				_classifier_classifier_6_linear_MatMul_ry_tmp = builder.add(tmp, _classifier_classifier_6_linear_MatMul_ry_tmp);
			}
			_classifier_classifier_6_linear_MatMul_ry[i] = _classifier_classifier_6_linear_MatMul_ry_tmp;
		}
		let mut _classifier_classifier_6_linear_MatMul_rxy: Variable = Default::default();
		let mut _classifier_classifier_6_linear_MatMul_rxy_tmp = builder.constant(0);
		for i in 0..512 {
			let tmp = builder.mul(_classifier_classifier_6_linear_MatMul_ry[i], _classifier_classifier_6_linear_MatMul_rx[i]);
			_classifier_classifier_6_linear_MatMul_rxy_tmp = builder.add(tmp, _classifier_classifier_6_linear_MatMul_rxy_tmp);
			}
				_classifier_classifier_6_linear_MatMul_rxy = _classifier_classifier_6_linear_MatMul_rxy_tmp;
		let mut _classifier_classifier_6_linear_MatMul_rz: [Variable;10] = [Default::default();10];
		for i in 0..10 {
			let mut _classifier_classifier_6_linear_MatMul_rz_tmp = builder.constant(0);
			for j in 0..16 {
				let tmp = builder.mul(self._classifier_classifier_4_relu_Cast_1_output_0_mat_ru[j], self._classifier_classifier_6_linear_MatMul_output_0[j][i]);
				_classifier_classifier_6_linear_MatMul_rz_tmp = builder.add(tmp, _classifier_classifier_6_linear_MatMul_rz_tmp);
			}
			_classifier_classifier_6_linear_MatMul_rz[i] = _classifier_classifier_6_linear_MatMul_rz_tmp;
		}
		let mut _classifier_classifier_6_linear_MatMul_rrz: Variable = Default::default();
		let mut _classifier_classifier_6_linear_MatMul_rrz_tmp = builder.constant(0);
		for i in 0..10 {
			let tmp = builder.mul(self.onnx__MatMul_217_mat_rv[i], _classifier_classifier_6_linear_MatMul_rz[i]);
			_classifier_classifier_6_linear_MatMul_rrz_tmp = builder.add(tmp, _classifier_classifier_6_linear_MatMul_rrz_tmp);
			}
		_classifier_classifier_6_linear_MatMul_rrz = _classifier_classifier_6_linear_MatMul_rrz_tmp;
		builder.assert_is_equal(_classifier_classifier_6_linear_MatMul_rrz, _classifier_classifier_6_linear_MatMul_rxy);
		// constant operation
		// multiply operation
		let mut _classifier_classifier_6_Mul_output_0: [[Variable;10];16] = [[Default::default();10];16];
		for i in 0..16 {
			for j in 0..10 {
				_classifier_classifier_6_Mul_output_0[i][j] = builder.mul(self._classifier_classifier_6_linear_MatMul_output_0[i][j], self._classifier_classifier_6_Constant_output_0);
			}
		}
		// constant operation
		// divide operation
		for i in 0..16 {
			for j in 0..10 {
				let tmp1 = builder.mul(self._classifier_classifier_6_Div_output_0[i][j], self._classifier_classifier_6_Constant_1_output_0);
				table.rangeproof(builder, self._classifier_classifier_6_Div_output_0_r[i][j], 24);
				let tmp2 = builder.sub(_classifier_classifier_6_Mul_output_0[i][j], self._classifier_classifier_6_Div_output_0_r[i][j]);
				builder.assert_is_equal(tmp1, tmp2);
							}
		}
		// cast operation
		let mut _classifier_classifier_6_Cast_output_0: [[Variable;10];16] = [[Default::default();10];16];
		for i in 0..16 {
			for j in 0..10 {
				_classifier_classifier_6_Cast_output_0[i][j] = self._classifier_classifier_6_Div_output_0[i][j];
			}
		}
		// cast operation
		let mut _classifier_classifier_6_Cast_1_output_0: [[Variable;10];16] = [[Default::default();10];16];
		for i in 0..16 {
			for j in 0..10 {
				_classifier_classifier_6_Cast_1_output_0[i][j] = _classifier_classifier_6_Cast_output_0[i][j];
			}
		}
		// constant operation
		// add operation
		let mut output: [[Variable;10];16] = [[Default::default();10];16];
		for i in 0..16 {
			for j in 0..10 {
				output[i][j] = builder.add(_classifier_classifier_6_Cast_1_output_0[i][j], self._classifier_classifier_6_Constant_2_output_0[j]);
			}
		}
		table.final_check(builder);
	}
}

#[test]
fn expander_circuit() -> std::io::Result<()>{ 
	let compile_result = stacker::grow(32 * 1024 * 1024 * 1024, ||
		{
			let mut circuit = Circuit::<Variable>::default();
			circuit.output =  vec![vec![Variable::default();10];16];
			circuit.input =  vec![vec![vec![vec![Variable::default();32];32];3];16];
			circuit._features_features_0_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_0_Constant_output_0 =  Variable::default();
			circuit._features_features_0_Constant_1_output_0 =  Variable::default();
			circuit._features_features_0_Div_output_0_r =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_0_Div_output_0 =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_0_Constant_2_output_0 =  vec![vec![vec![Variable::default();32];32];64];
			circuit._features_features_2_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_3_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_3_Constant_output_0 =  Variable::default();
			circuit._features_features_3_Constant_1_output_0 =  Variable::default();
			circuit._features_features_3_Div_output_0_r =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_3_Div_output_0 =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_3_Constant_2_output_0 =  vec![vec![vec![Variable::default();32];32];64];
			circuit._features_features_5_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_6_maxpool_MaxPool_output_0 =  vec![vec![vec![vec![Variable::default();16];16];64];16];
			circuit._features_features_7_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();16];16];128];16];
			circuit._features_features_7_Constant_output_0 =  Variable::default();
			circuit._features_features_7_Constant_1_output_0 =  Variable::default();
			circuit._features_features_7_Div_output_0_r =  vec![vec![vec![vec![Variable::default();16];16];128];16];
			circuit._features_features_7_Div_output_0 =  vec![vec![vec![vec![Variable::default();16];16];128];16];
			circuit._features_features_7_Constant_2_output_0 =  vec![vec![vec![Variable::default();16];16];128];
			circuit._features_features_9_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();16];16];128];16];
			circuit._features_features_10_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();16];16];128];16];
			circuit._features_features_10_Constant_output_0 =  Variable::default();
			circuit._features_features_10_Constant_1_output_0 =  Variable::default();
			circuit._features_features_10_Div_output_0_r =  vec![vec![vec![vec![Variable::default();16];16];128];16];
			circuit._features_features_10_Div_output_0 =  vec![vec![vec![vec![Variable::default();16];16];128];16];
			circuit._features_features_10_Constant_2_output_0 =  vec![vec![vec![Variable::default();16];16];128];
			circuit._features_features_12_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();16];16];128];16];
			circuit._features_features_13_maxpool_MaxPool_output_0 =  vec![vec![vec![vec![Variable::default();8];8];128];16];
			circuit._features_features_14_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_14_Constant_output_0 =  Variable::default();
			circuit._features_features_14_Constant_1_output_0 =  Variable::default();
			circuit._features_features_14_Div_output_0_r =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_14_Div_output_0 =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_14_Constant_2_output_0 =  vec![vec![vec![Variable::default();8];8];256];
			circuit._features_features_16_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_17_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_17_Constant_output_0 =  Variable::default();
			circuit._features_features_17_Constant_1_output_0 =  Variable::default();
			circuit._features_features_17_Div_output_0_r =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_17_Div_output_0 =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_17_Constant_2_output_0 =  vec![vec![vec![Variable::default();8];8];256];
			circuit._features_features_19_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_20_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_20_Constant_output_0 =  Variable::default();
			circuit._features_features_20_Constant_1_output_0 =  Variable::default();
			circuit._features_features_20_Div_output_0_r =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_20_Div_output_0 =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_20_Constant_2_output_0 =  vec![vec![vec![Variable::default();8];8];256];
			circuit._features_features_22_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();8];8];256];16];
			circuit._features_features_23_maxpool_MaxPool_output_0 =  vec![vec![vec![vec![Variable::default();4];4];256];16];
			circuit._features_features_24_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_24_Constant_output_0 =  Variable::default();
			circuit._features_features_24_Constant_1_output_0 =  Variable::default();
			circuit._features_features_24_Div_output_0_r =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_24_Div_output_0 =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_24_Constant_2_output_0 =  vec![vec![vec![Variable::default();4];4];512];
			circuit._features_features_26_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_27_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_27_Constant_output_0 =  Variable::default();
			circuit._features_features_27_Constant_1_output_0 =  Variable::default();
			circuit._features_features_27_Div_output_0_r =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_27_Div_output_0 =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_27_Constant_2_output_0 =  vec![vec![vec![Variable::default();4];4];512];
			circuit._features_features_29_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_30_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_30_Constant_output_0 =  Variable::default();
			circuit._features_features_30_Constant_1_output_0 =  Variable::default();
			circuit._features_features_30_Div_output_0_r =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_30_Div_output_0 =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_30_Constant_2_output_0 =  vec![vec![vec![Variable::default();4];4];512];
			circuit._features_features_32_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();4];4];512];16];
			circuit._features_features_33_maxpool_MaxPool_output_0 =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_34_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_34_Constant_output_0 =  Variable::default();
			circuit._features_features_34_Constant_1_output_0 =  Variable::default();
			circuit._features_features_34_Div_output_0_r =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_34_Div_output_0 =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_34_Constant_2_output_0 =  vec![vec![vec![Variable::default();2];2];512];
			circuit._features_features_36_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_37_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_37_Constant_output_0 =  Variable::default();
			circuit._features_features_37_Constant_1_output_0 =  Variable::default();
			circuit._features_features_37_Div_output_0_r =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_37_Div_output_0 =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_37_Constant_2_output_0 =  vec![vec![vec![Variable::default();2];2];512];
			circuit._features_features_39_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_40_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_40_Constant_output_0 =  Variable::default();
			circuit._features_features_40_Constant_1_output_0 =  Variable::default();
			circuit._features_features_40_Div_output_0_r =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_40_Div_output_0 =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_40_Constant_2_output_0 =  vec![vec![vec![Variable::default();2];2];512];
			circuit._features_features_42_relu_Relu_output_0 =  vec![vec![vec![vec![Variable::default();2];2];512];16];
			circuit._features_features_43_maxpool_MaxPool_output_0 =  vec![vec![vec![vec![Variable::default();1];1];512];16];
			circuit._classifier_classifier_0_linear_MatMul_output_0 =  vec![vec![Variable::default();512];16];
			circuit._classifier_classifier_0_Constant_output_0 =  Variable::default();
			circuit._classifier_classifier_0_Constant_1_output_0 =  Variable::default();
			circuit._classifier_classifier_0_Div_output_0_r =  vec![vec![Variable::default();512];16];
			circuit._classifier_classifier_0_Div_output_0 =  vec![vec![Variable::default();512];16];
			circuit._classifier_classifier_0_Constant_2_output_0 =  vec![Variable::default();512];
			circuit._classifier_classifier_1_relu_Relu_output_0 =  vec![vec![Variable::default();512];16];
			circuit._classifier_classifier_3_linear_MatMul_output_0 =  vec![vec![Variable::default();512];16];
			circuit._classifier_classifier_3_Constant_output_0 =  Variable::default();
			circuit._classifier_classifier_3_Constant_1_output_0 =  Variable::default();
			circuit._classifier_classifier_3_Div_output_0_r =  vec![vec![Variable::default();512];16];
			circuit._classifier_classifier_3_Div_output_0 =  vec![vec![Variable::default();512];16];
			circuit._classifier_classifier_3_Constant_2_output_0 =  vec![Variable::default();512];
			circuit._classifier_classifier_4_relu_Relu_output_0 =  vec![vec![Variable::default();512];16];
			circuit._classifier_classifier_6_linear_MatMul_output_0 =  vec![vec![Variable::default();10];16];
			circuit._classifier_classifier_6_Constant_output_0 =  Variable::default();
			circuit._classifier_classifier_6_Constant_1_output_0 =  Variable::default();
			circuit._classifier_classifier_6_Div_output_0_r =  vec![vec![Variable::default();10];16];
			circuit._classifier_classifier_6_Div_output_0 =  vec![vec![Variable::default();10];16];
			circuit._classifier_classifier_6_Constant_2_output_0 =  vec![Variable::default();10];
			circuit.features_0_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];3];64];
			circuit.features_3_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];64];64];
			circuit.features_7_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];64];128];
			circuit.features_10_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];128];128];
			circuit.features_14_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];128];256];
			circuit.features_17_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];256];256];
			circuit.features_20_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];256];256];
			circuit.features_24_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];256];512];
			circuit.features_27_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];512];512];
			circuit.features_30_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];512];512];
			circuit.features_34_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];512];512];
			circuit.features_37_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];512];512];
			circuit.features_40_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];512];512];
			circuit.onnx__MatMul_215 =  vec![vec![Variable::default();512];512];
			circuit.onnx__MatMul_216 =  vec![vec![Variable::default();512];512];
			circuit.onnx__MatMul_217 =  vec![vec![Variable::default();10];512];
			circuit.input_mat_ru = vec![Variable::default();16384]; 
			circuit.features_0_conv_weight_mat_rv = vec![Variable::default();64]; 
			circuit._features_features_2_relu_Cast_1_output_0_mat_ru = vec![Variable::default();16384]; 
			circuit.features_3_conv_weight_mat_rv = vec![Variable::default();64]; 
			circuit._features_features_6_maxpool_MaxPool_output_0_mat_ru = vec![Variable::default();4096]; 
			circuit.features_7_conv_weight_mat_rv = vec![Variable::default();128]; 
			circuit._features_features_9_relu_Cast_1_output_0_mat_ru = vec![Variable::default();4096]; 
			circuit.features_10_conv_weight_mat_rv = vec![Variable::default();128]; 
			circuit._features_features_13_maxpool_MaxPool_output_0_mat_ru = vec![Variable::default();1024]; 
			circuit.features_14_conv_weight_mat_rv = vec![Variable::default();256]; 
			circuit._features_features_16_relu_Cast_1_output_0_mat_ru = vec![Variable::default();1024]; 
			circuit.features_17_conv_weight_mat_rv = vec![Variable::default();256]; 
			circuit._features_features_19_relu_Cast_1_output_0_mat_ru = vec![Variable::default();1024]; 
			circuit.features_20_conv_weight_mat_rv = vec![Variable::default();256]; 
			circuit._features_features_23_maxpool_MaxPool_output_0_mat_ru = vec![Variable::default();256]; 
			circuit.features_24_conv_weight_mat_rv = vec![Variable::default();512]; 
			circuit._features_features_26_relu_Cast_1_output_0_mat_ru = vec![Variable::default();256]; 
			circuit.features_27_conv_weight_mat_rv = vec![Variable::default();512]; 
			circuit._features_features_29_relu_Cast_1_output_0_mat_ru = vec![Variable::default();256]; 
			circuit.features_30_conv_weight_mat_rv = vec![Variable::default();512]; 
			circuit._features_features_33_maxpool_MaxPool_output_0_mat_ru = vec![Variable::default();64]; 
			circuit.features_34_conv_weight_mat_rv = vec![Variable::default();512]; 
			circuit._features_features_36_relu_Cast_1_output_0_mat_ru = vec![Variable::default();64]; 
			circuit.features_37_conv_weight_mat_rv = vec![Variable::default();512]; 
			circuit._features_features_39_relu_Cast_1_output_0_mat_ru = vec![Variable::default();64]; 
			circuit.features_40_conv_weight_mat_rv = vec![Variable::default();512]; 
			circuit._Flatten_output_0_mat_ru = vec![Variable::default();16]; 
			circuit.onnx__MatMul_215_mat_rv = vec![Variable::default();512]; 
			circuit._classifier_classifier_1_relu_Cast_1_output_0_mat_ru = vec![Variable::default();16]; 
			circuit.onnx__MatMul_216_mat_rv = vec![Variable::default();512]; 
			circuit._classifier_classifier_4_relu_Cast_1_output_0_mat_ru = vec![Variable::default();16]; 
			circuit.onnx__MatMul_217_mat_rv = vec![Variable::default();10]; 
			println!("Compile Circuit");
			let compile_result = compile(&circuit, CompileOptions::default()).unwrap();
			let file = std::fs::File::create("circuit.txt").unwrap();
			let writer = std::io::BufWriter::new(file);
			compile_result
				.layered_circuit
				.serialize_into(writer)
				.unwrap();
			let file = std::fs::File::create("witness_solver.txt").unwrap();
			let writer = std::io::BufWriter::new(file);
			compile_result
				.witness_solver
				.serialize_into(writer)
				.unwrap();
		}
	);
	Ok(())
}
