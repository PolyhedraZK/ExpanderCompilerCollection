use crate::{
    circuit::{
        config::{BN254Config, Config, GF2Config, M31Config},
        ir::{
            common::rand_gen::{RandomCircuitConfig, RandomRange},
            source::RootCircuit as IrSourceRoot,
        },
    },
    compile::compile,
    utils::error::Error,
};

fn do_test<C: Config>(mut config: RandomCircuitConfig, seed: RandomRange) {
    for i in seed.min..seed.max {
        config.seed = i;
        let root = IrSourceRoot::<C>::random(&config);
        assert_eq!(root.validate(), Ok(()));
        let res = compile(&root);
        match res {
            Ok((ir_hint_normalized, layered_circuit)) => {
                assert_eq!(ir_hint_normalized.validate(), Ok(()));
                assert_eq!(layered_circuit.validate(), Ok(()));
            }
            Err(e) => match e {
                Error::UserError(_) => {}
                Error::InternalError(e) => {
                    panic!("{:?}", e);
                }
            },
        }
    }
}

fn do_tests<C: Config>(seed: usize) {
    let config = RandomCircuitConfig {
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
    do_test::<C>(
        config,
        RandomRange {
            min: 100000 + seed,
            max: 103000 + seed,
        },
    );
    let config = RandomCircuitConfig {
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
    do_test::<C>(
        config,
        RandomRange {
            min: 200000 + seed,
            max: 201000 + seed,
        },
    );
}

#[test]
fn test_m31() {
    do_tests::<M31Config>(1000000);
}

#[test]
fn test_bn254() {
    do_tests::<BN254Config>(2000000);
}

/*#[test]
fn test_gf2() {
    do_tests::<GF2Config>(3000000);
}*/
// TODO: currently GF2 won't work, since we disabled random combination in the dumb way
