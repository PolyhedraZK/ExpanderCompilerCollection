use crate::circuit::{
    config::{Config, M31Config as C},
    input_mapping::InputMapping,
    ir::{
        common::rand_gen::*,
        dest::{Circuit as IrCircuit, Instruction, RootCircuit as IrRootCircuit},
    },
    layered,
};

use crate::field::Field;

use super::compile;
type CField = <C as Config>::CircuitField;

pub fn test_input<C: Config>(
    rc: &IrRootCircuit<C>,
    lc: &layered::Circuit<C>,
    input_mapping: &InputMapping,
    input: &Vec<C::CircuitField>,
) {
    let (rc_output, rc_cond) = rc.eval_unsafe(input.clone());
    let lc_input = input_mapping.map_inputs(input);
    let (lc_output, lc_cond) = lc.eval_unsafe(lc_input);
    //println!("{:?}", rc_output);
    //println!("{:?}", lc_output);
    assert_eq!(rc_cond, lc_cond);
    assert_eq!(rc_output, lc_output);
}

pub fn compile_and_random_test<C: Config>(
    rc: &IrRootCircuit<C>,
    n_tests: usize,
) -> (layered::Circuit<C>, InputMapping) {
    assert!(rc.validate().is_ok());
    let (lc, input_mapping) = compile(rc);
    //print!("{}", lc);
    assert_eq!(lc.validate(), Ok(()));
    assert_eq!(rc.input_size(), input_mapping.cur_size());
    let input_size = rc.input_size();
    for _ in 0..n_tests {
        let input: Vec<C::CircuitField> = (0..input_size)
            .map(|_| C::CircuitField::random_unsafe())
            .collect();
        test_input(rc, &lc, &input_mapping, &input);
    }
    (lc, input_mapping)
}

#[test]
fn simple() {
    use crate::circuit::ir::expr::{Expression, Term};

    let mut root = IrRootCircuit::<C>::default();
    root.circuits.insert(
        0,
        IrCircuit {
            instructions: vec![
                Instruction::SubCircuitCall {
                    sub_circuit_id: 1,
                    inputs: vec![1],
                    num_outputs: 1,
                },
                Instruction::SubCircuitCall {
                    sub_circuit_id: 1,
                    inputs: vec![2],
                    num_outputs: 1,
                },
            ],
            constraints: vec![1],
            outputs: vec![3],
            num_inputs: 1,
            num_hint_inputs: 0,
        },
    );
    root.circuits.insert(
        1,
        IrCircuit {
            instructions: vec![
                Instruction::SubCircuitCall {
                    sub_circuit_id: 2,
                    inputs: vec![1],
                    num_outputs: 1,
                },
                Instruction::SubCircuitCall {
                    sub_circuit_id: 2,
                    inputs: vec![3],
                    num_outputs: 1,
                },
                Instruction::InternalVariable {
                    expr: Expression::from_terms(vec![
                        Term::new_linear(CField::one(), 4),
                        Term::new_linear(CField::from(9), 2),
                    ]),
                },
            ],
            constraints: vec![],
            outputs: vec![5],
            num_inputs: 1,
            num_hint_inputs: 1,
        },
    );
    root.circuits.insert(
        2,
        IrCircuit {
            instructions: vec![Instruction::InternalVariable {
                expr: Expression::from_terms(vec![
                    Term::new_quad(CField::one(), 1, 1),
                    Term::new_linear(CField::from(10), 2),
                ]),
            }],
            constraints: vec![],
            outputs: vec![3],
            num_inputs: 1,
            num_hint_inputs: 1,
        },
    );
    let (lc, input_mapping) = compile_and_random_test(&root, 100);
    test_input(
        &root,
        &lc,
        &input_mapping,
        &vec![CField::zero(); root.input_size()],
    );
}

#[test]
fn random_circuits_1() {
    let mut config = RandomCircuitConfig {
        seed: 0,
        num_circuits: RandomRange { min: 1, max: 10 },
        num_inputs: RandomRange { min: 1, max: 10 },
        num_hint_inputs: RandomRange { min: 0, max: 10 },
        num_instructions: RandomRange { min: 1, max: 10 },
        num_constraints: RandomRange { min: 0, max: 10 },
        num_outputs: RandomRange { min: 1, max: 10 },
        num_terms: RandomRange { min: 1, max: 5 },
        sub_circuit_prob: 0.5,
    };
    for i in 0..1000 {
        config.seed = i;
        let root = IrRootCircuit::<C>::random(&config);
        assert_eq!(root.validate(), Ok(()));
        compile_and_random_test(&root, 5);
    }
}

#[test]
fn random_circuits_2() {
    let mut config = RandomCircuitConfig {
        seed: 0,
        num_circuits: RandomRange { min: 1, max: 100 },
        num_inputs: RandomRange { min: 1, max: 3 },
        num_hint_inputs: RandomRange { min: 0, max: 2 },
        num_instructions: RandomRange { min: 5, max: 10 },
        num_constraints: RandomRange { min: 0, max: 5 },
        num_outputs: RandomRange { min: 1, max: 3 },
        num_terms: RandomRange { min: 1, max: 5 },
        sub_circuit_prob: 0.1,
    };
    for i in 0..1000 {
        config.seed = i + 10000;
        let root = IrRootCircuit::<C>::random(&config);
        assert_eq!(root.validate(), Ok(()));
        compile_and_random_test(&root, 5);
    }
}

#[test]
fn random_circuits_3() {
    let mut config = RandomCircuitConfig {
        seed: 0,
        num_circuits: RandomRange { min: 1, max: 20 },
        num_inputs: RandomRange { min: 1, max: 3 },
        num_hint_inputs: RandomRange { min: 0, max: 2 },
        num_instructions: RandomRange { min: 30, max: 50 },
        num_constraints: RandomRange { min: 0, max: 5 },
        num_outputs: RandomRange { min: 1, max: 3 },
        num_terms: RandomRange { min: 1, max: 5 },
        sub_circuit_prob: 0.05,
    };
    for i in 0..1000 {
        config.seed = i + 20000;
        let root = IrRootCircuit::<C>::random(&config);
        assert_eq!(root.validate(), Ok(()));
        compile_and_random_test(&root, 5);
    }
}

#[test]
fn random_circuits_4() {
    let mut config = RandomCircuitConfig {
        seed: 0,
        num_circuits: RandomRange { min: 1, max: 1 },
        num_inputs: RandomRange { min: 100, max: 200 },
        num_hint_inputs: RandomRange { min: 100, max: 200 },
        num_instructions: RandomRange {
            min: 1000,
            max: 2000,
        },
        num_constraints: RandomRange { min: 0, max: 5 },
        num_outputs: RandomRange { min: 1, max: 3 },
        num_terms: RandomRange { min: 1, max: 30 },
        sub_circuit_prob: 0.0,
    };
    for i in 0..200 {
        config.seed = i + 30000;
        let root = IrRootCircuit::<C>::random(&config);
        assert_eq!(root.validate(), Ok(()));
        compile_and_random_test(&root, 5);
    }
}
