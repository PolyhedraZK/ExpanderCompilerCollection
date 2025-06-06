use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proof::ComputationGraph;
use expander_compiler::zkcuda::proving_system::{ExpanderGKRProvingSystem, ParallelizedExpanderGKRProvingSystem, ProvingSystem,};
use expander_compiler::zkcuda::{context::*, kernel::*};
use gkr::BN254ConfigSha2Hyrax;
use gkr_engine::FieldEngine;
use serdes::ExpSerde;
use serde::{Deserialize, Serialize};
use std::fs;

		// conv operation
#[kernel]		// multiply operation
fn _features_features_0_features_0_0_Conv_mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_features_0_0_Conv_output_0_conv: &[[[InputVariable;112];112];32],
	onnx__Conv_621_nscale: &InputVariable,
	_features_features_0_features_0_0_Conv_output_0_mul: &mut [[[OutputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_0_features_0_0_Conv_output_0_mul[i][j][k] = api.mul(_features_features_0_features_0_0_Conv_output_0_conv[i][j][k], onnx__Conv_621_nscale);
			}
		}
	}
}
		// divide operation
		// floor operation
#[kernel]		// add operation
fn _features_features_0_features_0_0_Conv_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_features_0_0_Conv_output_0_floor: &[[[InputVariable;112];112];32],
	onnx__Conv_622_q: &[[[InputVariable;1];1];32],
	_features_features_0_features_0_0_Conv_output_0: &mut [[[OutputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_0_features_0_0_Conv_output_0[i][j][k] = api.add(_features_features_0_features_0_0_Conv_output_0_floor[i][j][k], onnx__Conv_622_q[i][0][0]);
			}
		}
	}
}
#[kernel]		// relu operation
fn _features_features_0_features_0_2_PRelu_relu_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_features_0_0_Conv_output_0: &[[[InputVariable;112];112];32],
	_features_features_0_features_0_0_Conv_output_0_relu: &mut [[[InputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				let tmp1 = api.sub(_features_features_0_features_0_0_Conv_output_0_relu[i][j][k], _features_features_0_features_0_0_Conv_output_0[i][j][k]);
				let tmp2 = api.mul(tmp1, _features_features_0_features_0_0_Conv_output_0_relu[i][j][k]);
				api.assert_is_zero(tmp2);
			}
		}
	}
}
#[kernel]		// multiply operation
fn _features_features_0_features_0_2_PRelu_pos_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_features_0_0_Conv_output_0_relu: &[[[InputVariable;112];112];32],
	onnx__PRelu_779_dscale: &InputVariable,
	_features_features_0_features_0_0_Conv_output_0_pos: &mut [[[OutputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_0_features_0_0_Conv_output_0_pos[i][j][k] = api.mul(_features_features_0_features_0_0_Conv_output_0_relu[i][j][k], onnx__PRelu_779_dscale);
			}
		}
	}
}
#[kernel]		// multiply operation
fn _features_features_0_features_0_2_PRelu_qn_macro<C: Config>(
	api: &mut API<C>,
	onnx__PRelu_779_q: &[[InputVariable;1];1],
	onnx__PRelu_779_nscale: &InputVariable,
	onnx__PRelu_779_qn: &mut [[OutputVariable;1];1],
) {
	for i in 0..1 {
		for j in 0..1 {
			onnx__PRelu_779_qn[i][j] = api.mul(onnx__PRelu_779_q[0][0], onnx__PRelu_779_nscale);
		}
	}
}
		// min operation
#[kernel]		// multiply operation
fn _features_features_0_features_0_2_PRelu_neg_macro<C: Config>(
	api: &mut API<C>,
	onnx__PRelu_779_qn: &[[[InputVariable;1];1];32],
	_features_features_0_features_0_0_Conv_output_0_min: &[[[InputVariable;112];112];32],
	_features_features_0_features_0_0_Conv_output_0_neg: &mut [[[OutputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_0_features_0_0_Conv_output_0_neg[i][j][k] = api.mul(onnx__PRelu_779_qn[i][0][0], _features_features_0_features_0_0_Conv_output_0_min[i][j][k]);
			}
		}
	}
}
#[kernel]		// add operation
fn _features_features_0_features_0_2_PRelu_prelu_macro<C: Config>(
	api: &mut API<C>,
	_features_features_0_features_0_0_Conv_output_0_pos: &[[[InputVariable;112];112];32],
	_features_features_0_features_0_0_Conv_output_0_neg: &[[[InputVariable;112];112];32],
	_features_features_0_features_0_0_Conv_output_0_prelu: &mut [[[OutputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_0_features_0_0_Conv_output_0_prelu[i][j][k] = api.add(_features_features_0_features_0_0_Conv_output_0_pos[i][j][k], _features_features_0_features_0_0_Conv_output_0_neg[i][j][k]);
			}
		}
	}
}
		// divide operation
		// floor operation
		// conv operation
#[kernel]		// multiply operation
fn _features_features_1_conv_conv_0_conv_0_0_Conv_mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv: &[[[InputVariable;112];112];32],
	onnx__Conv_624_nscale: &InputVariable,
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_mul: &mut [[[OutputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_mul[i][j][k] = api.mul(_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k], onnx__Conv_624_nscale);
			}
		}
	}
}
		// divide operation
		// floor operation
#[kernel]		// add operation
fn _features_features_1_conv_conv_0_conv_0_0_Conv_macro<C: Config>(
	api: &mut API<C>,
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor: &[[[InputVariable;112];112];32],
	onnx__Conv_625_q: &[[[InputVariable;1];1];32],
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0: &mut [[[OutputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_1_conv_conv_0_conv_0_0_Conv_output_0[i][j][k] = api.add(_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k], onnx__Conv_625_q[i][0][0]);
			}
		}
	}
}
#[kernel]		// relu operation
fn _features_features_1_conv_conv_0_conv_0_2_PRelu_relu_macro<C: Config>(
	api: &mut API<C>,
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0: &[[[InputVariable;112];112];32],
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu: &mut [[[InputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				let tmp1 = api.sub(_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k], _features_features_1_conv_conv_0_conv_0_0_Conv_output_0[i][j][k]);
				let tmp2 = api.mul(tmp1, _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k]);
				api.assert_is_zero(tmp2);
			}
		}
	}
}
#[kernel]		// multiply operation
fn _features_features_1_conv_conv_0_conv_0_2_PRelu_pos_macro<C: Config>(
	api: &mut API<C>,
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu: &[[[InputVariable;112];112];32],
	onnx__PRelu_780_dscale: &InputVariable,
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_pos: &mut [[[OutputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_pos[i][j][k] = api.mul(_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k], onnx__PRelu_780_dscale);
			}
		}
	}
}
#[kernel]		// multiply operation
fn _features_features_1_conv_conv_0_conv_0_2_PRelu_qn_macro<C: Config>(
	api: &mut API<C>,
	onnx__PRelu_780_q: &[[InputVariable;1];1],
	onnx__PRelu_780_nscale: &InputVariable,
	onnx__PRelu_780_qn: &mut [[OutputVariable;1];1],
) {
	for i in 0..1 {
		for j in 0..1 {
			onnx__PRelu_780_qn[i][j] = api.mul(onnx__PRelu_780_q[0][0], onnx__PRelu_780_nscale);
		}
	}
}
		// min operation
#[kernel]		// multiply operation
fn _features_features_1_conv_conv_0_conv_0_2_PRelu_neg_macro<C: Config>(
	api: &mut API<C>,
	onnx__PRelu_780_qn: &[[[InputVariable;1];1];32],
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min: &[[[InputVariable;112];112];32],
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_neg: &mut [[[OutputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_neg[i][j][k] = api.mul(onnx__PRelu_780_qn[i][0][0], _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k]);
			}
		}
	}
}
#[kernel]		// add operation
fn _features_features_1_conv_conv_0_conv_0_2_PRelu_prelu_macro<C: Config>(
	api: &mut API<C>,
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_pos: &[[[InputVariable;112];112];32],
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_neg: &[[[InputVariable;112];112];32],
	_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_prelu: &mut [[[OutputVariable;112];112];32],
) {
	for i in 0..32 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_prelu[i][j][k] = api.add(_features_features_1_conv_conv_0_conv_0_0_Conv_output_0_pos[i][j][k], _features_features_1_conv_conv_0_conv_0_0_Conv_output_0_neg[i][j][k]);
			}
		}
	}
}
		// divide operation
		// floor operation
		// conv operation
#[kernel]		// multiply operation
fn _features_features_1_conv_conv_1_Conv_mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_1_conv_conv_1_Conv_output_0_conv: &[[[InputVariable;112];112];16],
	onnx__Conv_627_nscale: &InputVariable,
	_features_features_1_conv_conv_1_Conv_output_0_mul: &mut [[[OutputVariable;112];112];16],
) {
	for i in 0..16 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_1_conv_conv_1_Conv_output_0_mul[i][j][k] = api.mul(_features_features_1_conv_conv_1_Conv_output_0_conv[i][j][k], onnx__Conv_627_nscale);
			}
		}
	}
}
		// divide operation
		// floor operation
#[kernel]		// add operation
fn _features_features_1_conv_conv_1_Conv_macro<C: Config>(
	api: &mut API<C>,
	_features_features_1_conv_conv_1_Conv_output_0_floor: &[[[InputVariable;112];112];16],
	onnx__Conv_628_q: &[[[InputVariable;1];1];16],
	_features_features_1_conv_conv_1_Conv_output_0: &mut [[[OutputVariable;112];112];16],
) {
	for i in 0..16 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_1_conv_conv_1_Conv_output_0[i][j][k] = api.add(_features_features_1_conv_conv_1_Conv_output_0_floor[i][j][k], onnx__Conv_628_q[i][0][0]);
			}
		}
	}
}
		// conv operation
#[kernel]		// multiply operation
fn _features_features_2_conv_conv_0_conv_0_0_Conv_mul_macro<C: Config>(
	api: &mut API<C>,
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv: &[[[InputVariable;112];112];96],
	onnx__Conv_630_nscale: &InputVariable,
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_mul: &mut [[[OutputVariable;112];112];96],
) {
	for i in 0..96 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_mul[i][j][k] = api.mul(_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_conv[i][j][k], onnx__Conv_630_nscale);
			}
		}
	}
}
		// divide operation
		// floor operation
#[kernel]		// add operation
fn _features_features_2_conv_conv_0_conv_0_0_Conv_macro<C: Config>(
	api: &mut API<C>,
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor: &[[[InputVariable;112];112];96],
	onnx__Conv_631_q: &[[[InputVariable;1];1];96],
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0: &mut [[[OutputVariable;112];112];96],
) {
	for i in 0..96 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_2_conv_conv_0_conv_0_0_Conv_output_0[i][j][k] = api.add(_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_floor[i][j][k], onnx__Conv_631_q[i][0][0]);
			}
		}
	}
}
#[kernel]		// relu operation
fn _features_features_2_conv_conv_0_conv_0_2_PRelu_relu_macro<C: Config>(
	api: &mut API<C>,
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0: &[[[InputVariable;112];112];96],
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu: &mut [[[InputVariable;112];112];96],
) {
	for i in 0..96 {
		for j in 0..112 {
			for k in 0..112 {
				let tmp1 = api.sub(_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k], _features_features_2_conv_conv_0_conv_0_0_Conv_output_0[i][j][k]);
				let tmp2 = api.mul(tmp1, _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k]);
				api.assert_is_zero(tmp2);
			}
		}
	}
}
#[kernel]		// multiply operation
fn _features_features_2_conv_conv_0_conv_0_2_PRelu_pos_macro<C: Config>(
	api: &mut API<C>,
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu: &[[[InputVariable;112];112];96],
	onnx__PRelu_781_dscale: &InputVariable,
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_pos: &mut [[[OutputVariable;112];112];96],
) {
	for i in 0..96 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_pos[i][j][k] = api.mul(_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_relu[i][j][k], onnx__PRelu_781_dscale);
			}
		}
	}
}
#[kernel]		// multiply operation
fn _features_features_2_conv_conv_0_conv_0_2_PRelu_qn_macro<C: Config>(
	api: &mut API<C>,
	onnx__PRelu_781_q: &[[InputVariable;1];1],
	onnx__PRelu_781_nscale: &InputVariable,
	onnx__PRelu_781_qn: &mut [[OutputVariable;1];1],
) {
	for i in 0..1 {
		for j in 0..1 {
			onnx__PRelu_781_qn[i][j] = api.mul(onnx__PRelu_781_q[0][0], onnx__PRelu_781_nscale);
		}
	}
}
		// min operation
#[kernel]		// multiply operation
fn _features_features_2_conv_conv_0_conv_0_2_PRelu_neg_macro<C: Config>(
	api: &mut API<C>,
	onnx__PRelu_781_qn: &[[[InputVariable;1];1];96],
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min: &[[[InputVariable;112];112];96],
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_neg: &mut [[[OutputVariable;112];112];96],
) {
	for i in 0..96 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_neg[i][j][k] = api.mul(onnx__PRelu_781_qn[i][0][0], _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_min[i][j][k]);
			}
		}
	}
}
#[kernel]		// add operation
fn _features_features_2_conv_conv_0_conv_0_2_PRelu_prelu_macro<C: Config>(
	api: &mut API<C>,
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_pos: &[[[InputVariable;112];112];96],
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_neg: &[[[InputVariable;112];112];96],
	_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_prelu: &mut [[[OutputVariable;112];112];96],
) {
	for i in 0..96 {
		for j in 0..112 {
			for k in 0..112 {
				_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_prelu[i][j][k] = api.add(_features_features_2_conv_conv_0_conv_0_0_Conv_output_0_pos[i][j][k], _features_features_2_conv_conv_0_conv_0_0_Conv_output_0_neg[i][j][k]);
			}
		}
	}
}
		// divide operation
		// floor operation
		// conv operation
#[test]
fn expander_circuit() -> std::io::Result<()>{ 
	let compile_result = stacker::grow(32 * 1024 * 1024 * 1024, ||
		{
			let kernel__features_features_0_features_0_0_Conv_mul: Kernel<BN254Config> = compile__features_features_0_features_0_0_Conv_mul_macro().unwrap();
			let kernel__features_features_0_features_0_0_Conv: Kernel<BN254Config> = compile__features_features_0_features_0_0_Conv_macro().unwrap();
			let kernel__features_features_0_features_0_2_PRelu_relu: Kernel<BN254Config> = compile__features_features_0_features_0_2_PRelu_relu_macro().unwrap();
			let kernel__features_features_0_features_0_2_PRelu_pos: Kernel<BN254Config> = compile__features_features_0_features_0_2_PRelu_pos_macro().unwrap();
			let kernel__features_features_0_features_0_2_PRelu_qn: Kernel<BN254Config> = compile__features_features_0_features_0_2_PRelu_qn_macro().unwrap();
			let kernel__features_features_0_features_0_2_PRelu_neg: Kernel<BN254Config> = compile__features_features_0_features_0_2_PRelu_neg_macro().unwrap();
			let kernel__features_features_0_features_0_2_PRelu_prelu: Kernel<BN254Config> = compile__features_features_0_features_0_2_PRelu_prelu_macro().unwrap();
			let kernel__features_features_1_conv_conv_0_conv_0_0_Conv_mul: Kernel<BN254Config> = compile__features_features_1_conv_conv_0_conv_0_0_Conv_mul_macro().unwrap();
			let kernel__features_features_1_conv_conv_0_conv_0_0_Conv: Kernel<BN254Config> = compile__features_features_1_conv_conv_0_conv_0_0_Conv_macro().unwrap();
			let kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_relu: Kernel<BN254Config> = compile__features_features_1_conv_conv_0_conv_0_2_PRelu_relu_macro().unwrap();
			let kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_pos: Kernel<BN254Config> = compile__features_features_1_conv_conv_0_conv_0_2_PRelu_pos_macro().unwrap();
			let kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_qn: Kernel<BN254Config> = compile__features_features_1_conv_conv_0_conv_0_2_PRelu_qn_macro().unwrap();
			let kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_neg: Kernel<BN254Config> = compile__features_features_1_conv_conv_0_conv_0_2_PRelu_neg_macro().unwrap();
			let kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_prelu: Kernel<BN254Config> = compile__features_features_1_conv_conv_0_conv_0_2_PRelu_prelu_macro().unwrap();
			let kernel__features_features_1_conv_conv_1_Conv_mul: Kernel<BN254Config> = compile__features_features_1_conv_conv_1_Conv_mul_macro().unwrap();
			let kernel__features_features_1_conv_conv_1_Conv: Kernel<BN254Config> = compile__features_features_1_conv_conv_1_Conv_macro().unwrap();
			let kernel__features_features_2_conv_conv_0_conv_0_0_Conv_mul: Kernel<BN254Config> = compile__features_features_2_conv_conv_0_conv_0_0_Conv_mul_macro().unwrap();
			let kernel__features_features_2_conv_conv_0_conv_0_0_Conv: Kernel<BN254Config> = compile__features_features_2_conv_conv_0_conv_0_0_Conv_macro().unwrap();
			let kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_relu: Kernel<BN254Config> = compile__features_features_2_conv_conv_0_conv_0_2_PRelu_relu_macro().unwrap();
			let kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_pos: Kernel<BN254Config> = compile__features_features_2_conv_conv_0_conv_0_2_PRelu_pos_macro().unwrap();
			let kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_qn: Kernel<BN254Config> = compile__features_features_2_conv_conv_0_conv_0_2_PRelu_qn_macro().unwrap();
			let kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_neg: Kernel<BN254Config> = compile__features_features_2_conv_conv_0_conv_0_2_PRelu_neg_macro().unwrap();
			let kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_prelu: Kernel<BN254Config> = compile__features_features_2_conv_conv_0_conv_0_2_PRelu_prelu_macro().unwrap();
			let kernels = vec![kernel__features_features_0_features_0_0_Conv_mul,kernel__features_features_0_features_0_0_Conv,kernel__features_features_0_features_0_2_PRelu_relu,kernel__features_features_0_features_0_2_PRelu_pos,kernel__features_features_0_features_0_2_PRelu_qn,kernel__features_features_0_features_0_2_PRelu_neg,kernel__features_features_0_features_0_2_PRelu_prelu,kernel__features_features_1_conv_conv_0_conv_0_0_Conv_mul,kernel__features_features_1_conv_conv_0_conv_0_0_Conv,kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_relu,kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_pos,kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_qn,kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_neg,kernel__features_features_1_conv_conv_0_conv_0_2_PRelu_prelu,kernel__features_features_1_conv_conv_1_Conv_mul,kernel__features_features_1_conv_conv_1_Conv,kernel__features_features_2_conv_conv_0_conv_0_0_Conv_mul,kernel__features_features_2_conv_conv_0_conv_0_0_Conv,kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_relu,kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_pos,kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_qn,kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_neg,kernel__features_features_2_conv_conv_0_conv_0_2_PRelu_prelu];
			let file = std::fs::File::create("circuit.txt").unwrap();
			let writer = std::io::BufWriter::new(file);
			kernels.serialize_into(writer);
		}
	);
	Ok(())
}
