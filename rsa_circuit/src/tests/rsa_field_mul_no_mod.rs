use std::mem::transmute;

use circuit_std_rs::{U2048Variable, BN_TWO_TO_120, N_LIMBS};
use expander_compiler::frontend::*;
use expander_compiler::{
    declare_circuit,
    frontend::{BN254Config, Variable},
};
use extra::debug_eval;
use halo2curves::bn256::Fr;
use num_bigint::BigUint;
use num_traits::Num;

use crate::RSAFieldElement;

declare_circuit!(MulNoModCircuit {
    x: [Variable; N_LIMBS],
    y: [Variable; N_LIMBS],
    result: [Variable; 2 * N_LIMBS],
});

impl GenericDefine<BN254Config> for MulNoModCircuit<Variable> {
    fn define<Builder: RootAPI<BN254Config>>(&self, builder: &mut Builder) {
        let x = U2048Variable::from_raw(self.x);
        let y = U2048Variable::from_raw(self.y);
        let two_to_120 = builder.constant(BN_TWO_TO_120);

        let res = U2048Variable::mul_without_mod_reduction(&x, &y, &two_to_120, builder);

        // builder.display("first limb", x.limbs[0]);

        for i in 0..2 * N_LIMBS {
            builder.assert_is_equal(res[i], self.result[i]);
        }
    }
}

impl MulNoModCircuit<Fr> {
    fn create_circuit(
        x: [[u64; 2]; N_LIMBS],
        y: [[u64; 2]; N_LIMBS],
        result: [[u64; 2]; 2 * N_LIMBS],
    ) -> MulNoModCircuit<Fr> {
        let mut x_limbs = [Fr::zero(); N_LIMBS];
        let mut y_limbs = [Fr::zero(); N_LIMBS];
        let mut result_limbs = [Fr::zero(); 2 * N_LIMBS];

        for i in 0..N_LIMBS {
            x_limbs[i] = Fr::from_raw([x[i][0], x[i][1], 0, 0]);
            y_limbs[i] = Fr::from_raw([y[i][0], y[i][1], 0, 0]);
        }

        for i in 0..2 * N_LIMBS {
            result_limbs[i] = Fr::from_raw([result[i][0], result[i][1], 0, 0]);
        }

        Self {
            x: x_limbs,
            y: y_limbs,
            result: result_limbs,
        }
    }
}

#[test]
fn test_mul_without_mod() {
    let compile_result =
        compile_generic(&MulNoModCircuit::default(), CompileOptions::default()).unwrap();

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
    let mut res = BigUint::from_str_radix(
        "3fff\
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
        ffffffffffffffffffffffffffff00\
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
        000000000000000000000000000000\
        000000000000000000000000000000",
        16,
    )
    .unwrap();
    assert_eq!(&x * &x, res);

    let x = RSAFieldElement::from_big_uint(x);
    let x = x
        .data
        .iter()
        .map(|&x| unsafe {
            let tmp = transmute::<u128, [u64; 2]>(x);
            [tmp[0], tmp[1]]
        })
        .collect::<Vec<_>>();
    let x = x.try_into().unwrap();

    let mut result = [[0, 0]; 2 * N_LIMBS];
    let two_to_120 = BigUint::from(1u64) << 120;
    for i in 0..2 * N_LIMBS {
        let tmp: BigUint = &res % &two_to_120;
        res >>= 120;
        let tmp = tmp.to_u64_digits();
        match tmp.len() {
            0 => {
                result[i] = [0, 0];
            }
            1 => {
                result[i] = [tmp[0], 0];
            }
            2 => {
                result[i] = [tmp[0], tmp[1]];
            }
            _ => panic!("Unexpected length"),
        }
    }

    let assignment = MulNoModCircuit::<Fr>::create_circuit(x, x, result);

    let witness = compile_result
        .witness_solver
        .solve_witness(&assignment)
        .unwrap();
    let output = compile_result.layered_circuit.run(&witness);

    debug_eval(&MulNoModCircuit::default(), &assignment);

    assert_eq!(output, vec![true]);
}
