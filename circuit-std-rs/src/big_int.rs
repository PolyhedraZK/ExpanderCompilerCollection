use expander_compiler::frontend::*;
use num_bigint::BigInt;

pub fn bytes_to_bits<C: Config, B: RootAPI<C>>(api: &mut B, vals: &[Variable]) -> Vec<Variable> {
	let mut ret = to_binary(api, vals[0], 8);
	for i in 1..vals.len() {
		ret = to_binary(api, vals[i], 8).into_iter().chain(ret.into_iter()).collect();
	}
	ret
}
pub fn right_shift<C: Config, B: RootAPI<C>>(api: &mut B, bits: &[Variable], shift: usize) -> Vec<Variable> {
	if bits.len() != 32 {
		panic!("RightShift: len(bits) != 32");
	}
	let mut shifted_bits = bits[shift..].to_vec();
	for _ in 0..shift {
		shifted_bits.push(api.constant(0));
	}
	shifted_bits
}
pub fn rotate_right(bits: &[Variable], shift: usize) -> Vec<Variable> {
	if bits.len() != 32 {
		panic!("RotateRight: len(bits) != 32");
	}
	let mut rotated_bits = bits[shift..].to_vec();
	rotated_bits.extend_from_slice(&bits[..shift]);
	rotated_bits
}
pub fn sigma0<C: Config, B: RootAPI<C>>(api: &mut B, bits: &[Variable]) -> Vec<Variable> {
	if bits.len() != 32 {
		panic!("Sigma0: len(bits) != 32");
	}
	let mut bits1 = bits.to_vec();
	let mut bits2 = bits.to_vec();
	let mut bits3 = bits.to_vec();
	let v1 = rotate_right(&bits1, 7);
	let v2 = rotate_right(&bits2, 18);
	let v3 = right_shift(api, &bits3, 3);
	let mut ret = vec![];
	for i in 0..32 {
		let tmp = api.xor(v1[i], v2[i]);
		ret.push(api.xor(tmp, v3[i]));
	}
	ret
}
pub fn sigma1<C: Config, B: RootAPI<C>>(api: &mut B, bits: &[Variable]) -> Vec<Variable> {
	if bits.len() != 32 {
		panic!("Sigma1: len(bits) != 32");
	}
	let mut bits1 = bits.to_vec();
	let mut bits2 = bits.to_vec();
	let mut bits3 = bits.to_vec();
	let v1 = rotate_right(&bits1, 17);
	let v2 = rotate_right(&bits2, 19);
	let v3 = right_shift(api, &bits3, 10);
	let mut ret = vec![];
	for i in 0..32 {
		let tmp = api.xor(v1[i], v2[i]);
		ret.push(api.xor(tmp, v3[i]));
	}
	ret
}
pub fn cap_sigma0<C: Config, B: RootAPI<C>>(api: &mut B, bits: &[Variable]) -> Vec<Variable> {
	if bits.len() != 32 {
		panic!("CapSigma0: len(bits) != 32");
	}
	let mut bits1 = bits.to_vec();
	let mut bits2 = bits.to_vec();
	let mut bits3 = bits.to_vec();
	let v1 = rotate_right(&bits1, 2);
	let v2 = rotate_right(&bits2, 13);
	let v3 = rotate_right(&bits3, 22);
	let mut ret = vec![];
	for i in 0..32 {
		let tmp = api.xor(v1[i], v2[i]);
		ret.push(api.xor(tmp, v3[i]));
	}
	ret
}
pub fn cap_sigma1<C: Config, B: RootAPI<C>>(api: &mut B, bits: &[Variable]) -> Vec<Variable> {
	if bits.len() != 32 {
		panic!("CapSigma1: len(bits) != 32");
	}
	let mut bits1 = bits.to_vec();
	let mut bits2 = bits.to_vec();
	let mut bits3 = bits.to_vec();
	let v1 = rotate_right(&bits1, 6);
	let v2 = rotate_right(&bits2, 11);
	let v3 = rotate_right(&bits3, 25);
	let mut ret = vec![];
	for i in 0..32 {
		let tmp = api.xor(v1[i], v2[i]);
		ret.push(api.xor(tmp, v3[i]));
	}
	ret
}
pub fn ch<C: Config, B: RootAPI<C>>(api: &mut B, x: &[Variable], y: &[Variable], z: &[Variable]) -> Vec<Variable> {
	if x.len() != 32 || y.len() != 32 || z.len() != 32 {
		panic!("Ch: len(x) != 32 || len(y) != 32 || len(z) != 32");
	}
	let mut ret = vec![];
	for i in 0..32 {
		let tmp1 = api.and(x[i], y[i]);
		let tmp2 = api.xor(x[i], 1);
		let tmp3 = api.and(tmp2, z[i]);
		ret.push(api.xor(tmp1, tmp3));
	}
	ret
}
pub fn maj<C: Config, B: RootAPI<C>>(api: &mut B, x: &[Variable], y: &[Variable], z: &[Variable]) -> Vec<Variable> {
	if x.len() != 32 || y.len() != 32 || z.len() != 32 {
		panic!("Maj: len(x) != 32 || len(y) != 32 || len(z) != 32");
	}
	let mut ret = vec![];
	for i in 0..32 {
		let tmp1 = api.and(x[i], y[i]);
		let tmp2 = api.and(x[i], z[i]);
		let tmp3 = api.and(y[i], z[i]);
		let tmp4 = api.xor(tmp1, tmp2);
		ret.push(api.xor(tmp3, tmp4));
	}
	ret
}
pub fn big_array_add<C: Config, B: RootAPI<C>>(api: &mut B, a: &[Variable], b: &[Variable], nb_bits: usize) -> Vec<Variable> {
	if a.len() != b.len() {
		panic!("BigArrayAdd: length of a and b must be equal");
	}
	let mut c = vec![api.constant(0); a.len()];
	let mut carry = api.constant(0);
	for i in 0..a.len() {
		c[i] = api.add(a[i], b[i]);
		c[i] = api.add(c[i], carry);
		carry = to_binary(api, c[i], nb_bits + 1)[nb_bits];
		let tmp  = api.mul(carry, 1 << nb_bits);
		c[i] = api.sub(c[i], tmp);
	}
	c
}
pub fn bit_array_to_m31<C: Config, B: RootAPI<C>>(api: &mut B, bits: &[Variable]) -> [Variable; 2] {
	if bits.len() >= 60 {
		panic!("BitArrayToM31: length of bits must be less than 60");
	}
	[from_binary(api, bits[..30].to_vec()), from_binary(api, bits[30..].to_vec())]
}

pub fn big_endian_m31_array_put_uint32<C: Config, B: RootAPI<C>>(api: &mut B, b: &mut [Variable], x: [Variable; 2]) {
	let mut quo = x[0];
	for i in (1..=3).rev() {
		let (q, r) = idiv_mod_bit(api, quo, 8);
		b[i] = r;
		quo = q;
	}
	let shift = api.mul(x[1], 1 << 6);
	b[0] = api.add(quo, shift);
}

pub fn big_endian_put_uint64<C: Config, B: RootAPI<C>>(api: &mut B, b: &mut [Variable], x: Variable) {
	let mut quo = x;
	for i in (1..=7).rev() {
		let (q, r) = idiv_mod_bit(api, quo, 8);
		b[i] = r;
		quo = q;
	}
	b[0] = quo;
}
pub fn m31_to_bit_array<C: Config, B: RootAPI<C>>(api: &mut B, m31: &[Variable]) -> Vec<Variable> {
	let mut bits = vec![];
	for i in 0..m31.len() {
		bits.extend_from_slice(&to_binary(api, m31[i], 30));
	}
	bits
}
pub fn to_binary<C: Config, B: RootAPI<C>>(api: &mut B, x: Variable, n_bits: usize) -> Vec<Variable> {
    api.new_hint("myhint.tobinary", &vec![x], n_bits)
}

pub fn from_binary<C: Config, B: RootAPI<C>>(api: &mut B, bits: Vec<Variable>) -> Variable {
    let mut res = api.constant(0);
    for i in 0..bits.len() {
        let coef = 1 << i;
        let cur = api.mul(coef, bits[i]);
        res = api.add(res, cur);
    }
    res
}

pub fn to_binary_hint(x: &[M31], y: &mut [M31]) -> Result<(), Error> {
    let t = x[0].to_u256();
    for (i, k) in y.iter_mut().enumerate() {
        *k = M31::from_u256(t >> i as u32 & 1);
    }
    Ok(())
}

pub fn idiv_mod_bit<C: Config, B: RootAPI<C>>(builder: &mut B, a: Variable, b: u64) -> (Variable, Variable) {
	let bits = to_binary(builder, a, 30);
	let quotient = from_binary(builder, bits[b as usize..].to_vec());
	let remainder = from_binary(builder, bits[..b as usize].to_vec());
	(quotient, remainder)
}

declare_circuit!(IDIVMODBITCircuit {
	value: PublicVariable,
	quotient: Variable,
	remainder: Variable,
});

impl Define<M31Config> for IDIVMODBITCircuit<Variable> {
	fn define(&self, builder: &mut API<M31Config>) {
		let (quotient, remainder) = idiv_mod_bit(builder, self.value, 8);
		builder.assert_is_equal(quotient, self.quotient);
		builder.assert_is_equal(remainder, self.remainder);
	}
}
#[test]
fn test_idiv_mod_bit() {
	//register hints
	let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);
	//compile and test
	let compile_result = compile(&IDIVMODBITCircuit::default()).unwrap();
	let assignment = IDIVMODBITCircuit::<M31> {
		value: M31::from(3845),
		quotient: M31::from(15),
		remainder: M31::from(5),
	};
	let witness = compile_result
		.witness_solver
		.solve_witness_with_hints(&assignment, &mut hint_registry)
            .unwrap();
	let output = compile_result.layered_circuit.run(&witness);
	assert_eq!(output, vec![true]);
}


declare_circuit!(BITCONVERTCircuit {
	big_int: PublicVariable,
	big_int_bytes: [Variable; 8],
	big_int_m31: [Variable; 2],
	big_int_m31_bytes: [Variable; 4],
});

impl Define<M31Config> for BITCONVERTCircuit<Variable> {
	fn define(&self, builder: &mut API<M31Config>) {
		let mut big_int_bytes = [builder.constant(0); 8];
		big_endian_put_uint64(builder, &mut big_int_bytes, self.big_int);
		for i in 0..8 {
			builder.assert_is_equal(big_int_bytes[i], self.big_int_bytes[i]);
		}
		let mut big_int_m31 = [builder.constant(0); 4];
		big_endian_m31_array_put_uint32(builder, &mut big_int_m31, self.big_int_m31);
		for i in 0..4 {
			builder.assert_is_equal(big_int_m31[i], self.big_int_m31_bytes[i]);
		}
	}
}
#[test]
fn test_bit_convert() {
	//register hints
	let mut hint_registry = HintRegistry::<M31>::new();
	hint_registry.register("myhint.tobinary", to_binary_hint);
	//compile and test
	let compile_result = compile(&BITCONVERTCircuit::default()).unwrap();
	let assignment = BITCONVERTCircuit::<M31> {
		big_int: M31::from(3845),
		big_int_bytes: [M31::from(0), M31::from(0), M31::from(0), M31::from(0), M31::from(0), M31::from(0), M31::from(15), M31::from(5)],
		big_int_m31: [M31::from(3845), M31::from(0)],
		big_int_m31_bytes: [M31::from(0), M31::from(0),M31::from(15), M31::from(5)],
	};
	let witness = compile_result
		.witness_solver
		.solve_witness_with_hints(&assignment, &mut hint_registry)
			.unwrap();
	let output = compile_result.layered_circuit.run(&witness);
	assert_eq!(output, vec![true]);
}



#[test]
fn test_300() {
    let hex_str = "ffffffffffffffff";
	let hex_byte = hex_str.as_bytes();
    let input = BigInt::parse_bytes(hex_byte, 16).unwrap();
    println!("input: {}", input);
}