use std::mem::transmute;

use expander_compiler::frontend::*;
use expander_compiler::{
    declare_circuit,
    frontend::{BN254Config, Define, Variable, API},
};
use halo2curves::bn256::Fr;

use crate::constants::{BN_TWO_TO_120, MASK120, MASK8, N_LIMBS};
use crate::native::RSAFieldElement;
use crate::u2048::U2048Variable;

declare_circuit!(MulModCircuit {
    x: [Variable; N_LIMBS],
    y: [Variable; N_LIMBS],
    result: [Variable; N_LIMBS],
    carry: [Variable; N_LIMBS],
    modulus: [Variable; N_LIMBS],
});

impl Define<BN254Config> for MulModCircuit<Variable> {
    fn define(&self, builder: &mut API<BN254Config>) {
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
    let compile_result = compile(&MulModCircuit::default()).unwrap();

    {
        // Test case 1: Simple modular multiplication
        let mut x = [[0, 0]; N_LIMBS];
        let mut y = [[0, 0]; N_LIMBS];
        let mut result = [[0, 0]; N_LIMBS];
        let mut carry = [[0, 0]; N_LIMBS];
        let mut modulus = [[0, 0]; N_LIMBS];

        x[0] = [7, 0];
        y[0] = [5, 0];
        result[0] = [5, 0]; // (7 * 5) % 10 = 35 % 10 = 5
        carry[0] = [3, 0]; // floor(35/10) = 3
        modulus[0] = [10, 0];

        let assignment = MulModCircuit::<Fr>::create_circuit(x, y, result, carry, modulus);
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);
    }

    {
        // Test case 2: Multiplication with no reduction needed
        let mut x = [[0, 0]; N_LIMBS];
        let mut y = [[0, 0]; N_LIMBS];
        let mut result = [[0, 0]; N_LIMBS];
        let carry = [[0, 0]; N_LIMBS];
        let mut modulus = [[0, 0]; N_LIMBS];

        x[0] = [5, 0];
        y[0] = [3, 0];
        result[0] = [15, 0];
        modulus[0] = [20, 0];

        let assignment = MulModCircuit::<Fr>::create_circuit(x, y, result, carry, modulus);
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);
    }

    {
        // Test case 3: Cross-limb multiplication with modular reduction
        let mut x = [[0, 0]; N_LIMBS];
        let mut y = [[0, 0]; N_LIMBS];
        let mut result = [[0, 0]; N_LIMBS];
        let mut carry = [[0, 0]; N_LIMBS];
        let mut modulus = [[0, 0]; N_LIMBS];

        x[0] = [1, 0];
        x[1] = [1, 0];
        y[0] = [1, 0];
        y[1] = [1, 0];
        result[0] = [1, 0];
        carry[0] = [0, 0x100000000000000];
        modulus[0] = [2, 0];
        modulus[1] = [1, 0];

        let assignment = MulModCircuit::<Fr>::create_circuit(x, y, result, carry, modulus);
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);
    }

    {
        // Test case 4: Multiplication by zero
        let mut x = [[0, 0]; N_LIMBS];
        let y = [[0, 0]; N_LIMBS];
        let result = [[0, 0]; N_LIMBS];
        let carry = [[0, 0]; N_LIMBS];
        let mut modulus = [[0, 0]; N_LIMBS];

        x[0] = [5, 0];
        modulus[0] = [10, 0];

        let assignment = MulModCircuit::<Fr>::create_circuit(x, y, result, carry, modulus);
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);
    }

    {
        // Test case 5: Large numbers with modular reduction
        let x = [[1, 0]; N_LIMBS];
        let y = [[2, 0]; N_LIMBS];

        let mut result = [[0, 0]; N_LIMBS];
        let mut carry = [[0, 0]; N_LIMBS];
        let mut modulus = [[0, 0]; N_LIMBS];

        result[0] = [2, 0]; // 0x2
        result[1] = [4, 0]; // 0x4
        result[2] = [6, 0]; // 0x6
        result[3] = [8, 0]; // 0x8
        result[4] = [10, 0]; // 0xa
        result[5] = [12, 0]; // 0xc
        result[6] = [14, 0]; // 0xe
        result[7] = [16, 0]; // 0x10
        result[8] = [18, 0]; // 0x12
        result[9] = [20, 0]; // 0x14
        result[10] = [22, 0]; // 0x16
        result[11] = [24, 0]; // 0x18
        result[12] = [26, 0]; // 0x1a
        result[13] = [28, 0]; // 0x1c
        result[14] = [30, 0]; // 0x1e
        result[15] = [32, 0]; // 0x20
        result[16] = [34, 0]; // 0x22
        result[17] = [36, 0]; // 0x24

        carry[0] = [0, 34 << 54]; // 34 * 2^54
        carry[1] = [0, 32 << 54]; // 32 * 2^54
        carry[2] = [0, 30 << 54]; // 30 * 2^54
        carry[3] = [0, 28 << 54]; // 28 * 2^54
        carry[4] = [0, 26 << 54]; // 26 * 2^54
        carry[5] = [0, 24 << 54]; // 24 * 2^54
        carry[6] = [0, 22 << 54]; // 22 * 2^54
        carry[7] = [0, 20 << 54]; // 20 * 2^54
        carry[8] = [0, 18 << 54]; // 18 * 2^54
        carry[9] = [0, 16 << 54]; // 16 * 2^54
        carry[10] = [0, 14 << 54]; // 14 * 2^54
        carry[11] = [0, 12 << 54]; // 12 * 2^54
        carry[12] = [0, 10 << 54]; // 10 * 2^54
        carry[13] = [0, 8 << 54]; // 8 * 2^54
        carry[14] = [0, 6 << 54]; // 6 * 2^54
        carry[15] = [0, 4 << 54]; // 4 * 2^54
        carry[16] = [0, 2 << 54]; // 2 * 2^54

        modulus[N_LIMBS - 1] = [1 << 8, 0];

        let assignment = MulModCircuit::<Fr>::create_circuit(x, y, result, carry, modulus);
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);
    }

    // Negative test cases
    {
        // Test case 6: Result >= modulus
        let mut x = [[0, 0]; N_LIMBS];
        let mut y = [[0, 0]; N_LIMBS];
        let mut result = [[0, 0]; N_LIMBS];
        let carry = [[0, 0]; N_LIMBS];
        let mut modulus = [[0, 0]; N_LIMBS];

        x[0] = [7, 0];
        y[0] = [5, 0];
        result[0] = [10, 0]; // Invalid: result >= modulus
        modulus[0] = [10, 0];

        let assignment = MulModCircuit::<Fr>::create_circuit(x, y, result, carry, modulus);
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![false]);
    }

    {
        // Test case 7: Incorrect carry value
        let mut x = [[0, 0]; N_LIMBS];
        let mut y = [[0, 0]; N_LIMBS];
        let mut result = [[0, 0]; N_LIMBS];
        let mut carry = [[0, 0]; N_LIMBS];
        let mut modulus = [[0, 0]; N_LIMBS];

        x[0] = [7, 0];
        y[0] = [5, 0];
        result[0] = [5, 0];
        carry[0] = [2, 0]; // Wrong carry (should be 3)
        modulus[0] = [10, 0];

        let assignment = MulModCircuit::<Fr>::create_circuit(x, y, result, carry, modulus);
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![false]);
    }

    {
        // Test case 8: Incorrect result
        let mut x = [[0, 0]; N_LIMBS];
        let mut y = [[0, 0]; N_LIMBS];
        let mut result = [[0, 0]; N_LIMBS];
        let mut carry = [[0, 0]; N_LIMBS];
        let mut modulus = [[0, 0]; N_LIMBS];

        x[0] = [7, 0];
        y[0] = [5, 0];
        result[0] = [6, 0]; // Wrong result (should be 5)
        carry[0] = [3, 0];
        modulus[0] = [10, 0];

        let assignment = MulModCircuit::<Fr>::create_circuit(x, y, result, carry, modulus);
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![false]);
    }
}
