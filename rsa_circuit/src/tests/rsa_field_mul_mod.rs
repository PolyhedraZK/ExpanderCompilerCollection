use std::mem::transmute;

use circuit_std_rs::{U2048Variable, BN_TWO_TO_120, N_LIMBS};
use expander_compiler::frontend::*;
use halo2curves::bn256::Fr;
use num_bigint::BigUint;
use num_traits::Num;

use crate::RSAFieldElement;

declare_circuit!(MulModCircuit {
    x: [Variable; N_LIMBS],
    y: [Variable; N_LIMBS],
    result: [Variable; N_LIMBS],
    carry: [Variable; N_LIMBS],
    modulus: [Variable; N_LIMBS],
});

impl Define<BN254Config> for MulModCircuit<Variable> {
    fn define<Builder: RootAPI<BN254Config>>(&self, builder: &mut Builder) {
        let x = U2048Variable::from_raw(self.x);
        let y = U2048Variable::from_raw(self.y);
        let result = U2048Variable::from_raw(self.result);
        let carry = U2048Variable::from_raw(self.carry);
        let modulus = U2048Variable::from_raw(self.modulus);
        let two_to_120 = builder.constant(BN_TWO_TO_120);

        U2048Variable::assert_mul(&x, &y, &result, &carry, &modulus, &two_to_120, builder);
    }
}

impl MulModCircuit<Fr> {
    fn create_circuit(
        x: [[u64; 2]; N_LIMBS],
        y: [[u64; 2]; N_LIMBS],
        result: [[u64; 2]; N_LIMBS],
        carry: [[u64; 2]; N_LIMBS],
        modulus: [[u64; 2]; N_LIMBS],
    ) -> MulModCircuit<Fr> {
        let mut x_limbs = [Fr::zero(); N_LIMBS];
        let mut y_limbs = [Fr::zero(); N_LIMBS];
        let mut result_limbs = [Fr::zero(); N_LIMBS];
        let mut carry_limbs = [Fr::zero(); N_LIMBS];
        let mut modulus_limbs = [Fr::zero(); N_LIMBS];

        for i in 0..N_LIMBS {
            x_limbs[i] = Fr::from_raw([x[i][0], x[i][1], 0, 0]);
            y_limbs[i] = Fr::from_raw([y[i][0], y[i][1], 0, 0]);
            result_limbs[i] = Fr::from_raw([result[i][0], result[i][1], 0, 0]);
            carry_limbs[i] = Fr::from_raw([carry[i][0], carry[i][1], 0, 0]);
            modulus_limbs[i] = Fr::from_raw([modulus[i][0], modulus[i][1], 0, 0]);
        }

        Self {
            x: x_limbs,
            y: y_limbs,
            result: result_limbs,
            carry: carry_limbs,
            modulus: modulus_limbs,
        }
    }
}

#[test]
fn test_mul_mod() {
    let compile_result = compile(&MulModCircuit::default(), CompileOptions::default()).unwrap();

    let x = BigUint::from_str_radix(
        "7f\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            000000000000000000000000000000",
        16,
    )
    .unwrap();
    let modulus = BigUint::from_str_radix(
        "80\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000000\
            000000000000000000000000000001\
            000000000000000000000000000000",
        16,
    )
    .unwrap();

    let res = BigUint::from_str_radix(
        "4000000000000000000000000000000000000000000000000000000000000",
        16,
    )
    .unwrap();
    let carry = BigUint::from_str_radix(
        "7f\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            ffffffffffffffffffffffffffffff\
            fffffffffffffffffffffffffffffd\
            000000000000000000000000000000",
        16,
    )
    .unwrap();
    assert_eq!(&x * &x, &res + &carry * &modulus);

    let x = RSAFieldElement::from_big_uint(x);
    let modulus = RSAFieldElement::from_big_uint(modulus);
    let res = RSAFieldElement::from_big_uint(res);
    let carry = RSAFieldElement::from_big_uint(carry);

    let x = x
        .data
        .iter()
        .map(|&x| unsafe {
            let tmp = transmute::<u128, [u64; 2]>(x);
            [tmp[0], tmp[1]]
        })
        .collect::<Vec<_>>();
    let x = x.try_into().unwrap();
    let modulus = modulus
        .data
        .iter()
        .map(|&x| unsafe {
            let tmp = transmute::<u128, [u64; 2]>(x);
            [tmp[0], tmp[1]]
        })
        .collect::<Vec<_>>();
    let modulus = modulus.try_into().unwrap();
    let res = res
        .data
        .iter()
        .map(|&x| unsafe {
            let tmp = transmute::<u128, [u64; 2]>(x);
            [tmp[0], tmp[1]]
        })
        .collect::<Vec<_>>();
    let res = res.try_into().unwrap();
    let carry = carry
        .data
        .iter()
        .map(|&x| unsafe {
            let tmp = transmute::<u128, [u64; 2]>(x);
            [tmp[0], tmp[1]]
        })
        .collect::<Vec<_>>();
    let carry = carry.try_into().unwrap();

    let assignment = MulModCircuit::<Fr>::create_circuit(x, x, res, carry, modulus);
    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);

    // println!("x");
    // for i in 0..N_LIMBS {
    //     println!("{} {:0x?}", i, x[i]);
    // }
    // println!("modulus");
    // for i in 0..N_LIMBS {
    //     println!("{} {:0x?}", i, modulus[i]);
    // }
    // println!("res");
    // for i in 0..N_LIMBS {
    //     println!("{} {:0x?}", i, res[i]);
    // }
    // println!("carry");
    // for i in 0..N_LIMBS {
    //     println!("{} {:0x?}", i, carry[i]);
    // }

    assert_eq!(output, vec![true]);
}
