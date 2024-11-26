use expander_compiler::frontend::*;
use expander_compiler::{
    declare_circuit,
    frontend::{BN254Config, BasicAPI, Define, Variable, API},
};
use halo2curves::{bn256::Bn256, bn256::Fr, pairing::Engine};

use crate::constants::N_LIMBS;
use crate::u120;

declare_circuit!(AddCircuit {
    x: [Variable; N_LIMBS],
    y: [Variable; N_LIMBS],
    r: [Variable; N_LIMBS],
    carry_ins: [Variable; N_LIMBS],
    carry_outs: [Variable; N_LIMBS],
    result: [Variable; N_LIMBS],
});

impl Define<BN254Config> for AddCircuit<Variable> {
    fn define(&self, builder: &mut API<BN254Config>) {
        for i in 0..N_LIMBS {
            u120::assert_add_120_with_carry(
                &self.x[i],
                &self.y[i],
                &self.carry_ins[i],
                &self.result[i],
                &self.carry_outs[i],
                builder,
            );
        }
    }
}

// Helper function to create circuit instance with given inputs
fn create_circuit(
    x: [u64; N_LIMBS],
    y: [u64; N_LIMBS],
    r: [u64; N_LIMBS],
    carry_ins: [u64; N_LIMBS],
    carry_outs: [u64; N_LIMBS],
    result: [u64; N_LIMBS],
) -> AddCircuit<Fr> {
    AddCircuit {
        x: x.map(|v| Fr::from(v)),
        y: y.map(|v| Fr::from(v)),
        r: r.map(|v| Fr::from(v)),
        carry_ins: carry_ins.map(|v| Fr::from(v)),
        carry_outs: carry_outs.map(|v| Fr::from(v)),
        result: result.map(|v| Fr::from(v)),
    }
}

#[test]
fn test_rsa_circuit_120_addition() {
    let compile_result = compile(&AddCircuit::default()).unwrap();

    {
        // Test case: Simple addition without carries
        let mut x = [0u64; N_LIMBS];
        x[0] = 50;

        let mut y = [0u64; N_LIMBS];
        y[0] = 30;

        let mut result = [0u64; N_LIMBS];
        result[0] = 80;

        let carry_ins = [0u64; N_LIMBS];
        let carry_outs = [0u64; N_LIMBS];

        let r = [0u64; N_LIMBS];

        let assignment = create_circuit(x, y, r, carry_ins, carry_outs, result);
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();

        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);
    }
    {
        // Test case: negative case
        let mut x = [0u64; N_LIMBS];
        x[0] = 50;

        let mut y = [0u64; N_LIMBS];
        y[0] = 40;

        let mut result = [0u64; N_LIMBS];
        result[0] = 80;

        let carry_ins = [0u64; N_LIMBS];
        let carry_outs = [0u64; N_LIMBS];

        let r = [0u64; N_LIMBS];

        let assignment = create_circuit(x, y, r, carry_ins, carry_outs, result);
        let witness = compile_result
            .witness_solver
            .solve_witness(&assignment)
            .unwrap();

        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![false]);
    }
}
