use expander_compiler::frontend::*;
use expander_compiler::circuit::layered::{NormalInputType, CrossLayerInputType};
use expander_compiler::Proof;
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

impl Define<BN254Config> for Circuit<Variable> {
	// fn define(&self, builder: &mut API<BN254Config>) {
	fn define<Builder: RootAPI<BN254Config>>(&self, builder: &mut Builder) {
		let mut table = LogUpRangeProofTable::new(16);
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
						let tmp3 = builder.sub(self._features_features_0_Constant_1_output_0, self._features_features_0_Div_output_0_r[i][j][k][l]);
						table.rangeproof(builder, tmp3, 24);
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
		let mut output: [[[[Variable;32];32];64];16] = [[[[Default::default();32];32];64];16];
		for i in 0..16 {
			for j in 0..64 {
				for k in 0..32 {
					for l in 0..32 {
						output[i][j][k][l] = builder.add(_features_features_0_Cast_1_output_0[i][j][k][l], self._features_features_0_Constant_2_output_0[j][k][l]);
					}
				}
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
			circuit.output =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit.input =  vec![vec![vec![vec![Variable::default();32];32];3];16];
			circuit._features_features_0_conv_Conv_output_0 =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_0_Constant_output_0 =  Variable::default();
			circuit._features_features_0_Constant_1_output_0 =  Variable::default();
			circuit._features_features_0_Div_output_0_r =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_0_Div_output_0 =  vec![vec![vec![vec![Variable::default();32];32];64];16];
			circuit._features_features_0_Constant_2_output_0 =  vec![vec![vec![Variable::default();32];32];64];
			circuit.features_0_conv_weight =  vec![vec![vec![vec![Variable::default();3];3];3];64];
			circuit.input_mat_ru = vec![Variable::default();16384]; 
			circuit.features_0_conv_weight_mat_rv = vec![Variable::default();64]; 
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
