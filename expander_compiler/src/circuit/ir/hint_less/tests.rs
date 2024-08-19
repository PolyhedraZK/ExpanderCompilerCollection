use rand::{Rng, RngCore};

use super::{
    Instruction::{self, ConstantOrRandom, LinComb, Mul},
    RootCircuit,
};
use crate::circuit::{
    config::{Config, M31Config as C},
    ir::{common::rand_gen::*, expr},
    layered::Coef,
};
use crate::field::Field;

type CField = <C as Config>::CircuitField;

impl<C: Config> RandomInstruction for Instruction<C> {
    fn random_no_sub_circuit(
        mut rnd: impl RngCore,
        num_terms: &RandomRange,
        num_vars: usize,
    ) -> Self {
        if rnd.gen::<f64>() < 0.2 {
            ConstantOrRandom(Coef::Constant(C::CircuitField::from(rnd.next_u32())))
        } else if rnd.gen::<f64>() < 0.5 {
            LinComb(expr::LinComb {
                terms: (0..num_terms.random(&mut rnd))
                    .map(|_| expr::LinCombTerm {
                        coef: C::CircuitField::from(rnd.next_u32()),
                        var: rnd.next_u64() as usize % num_vars + 1,
                    })
                    .collect(),
                constant: C::CircuitField::from(rnd.next_u32()),
            })
        } else {
            Mul((0..num_terms.random(&mut rnd).max(2))
                .map(|_| rnd.next_u64() as usize % num_vars + 1)
                .collect())
        }
    }
}

#[test]
fn opt_remove_unreachable() {
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
    for i in 0..3000 {
        config.seed = i;
        let root = RootCircuit::<C>::random(&config);
        assert_eq!(root.validate(), Ok(()));
        let (optroot, im) = root.remove_unreachable();
        assert_eq!(im.cur_size(), root.input_size());
        assert_eq!(optroot.validate(), Ok(()));
        let inputs: Vec<CField> = (0..root.input_size())
            .map(|_| CField::random_unsafe())
            .collect();
        let (out1, cond1) = root.eval_unsafe(inputs.clone());
        let (out2, cond2) = optroot.eval_unsafe(im.map_inputs(&inputs));
        assert_eq!(out1, out2);
        assert_eq!(cond1, cond2);
    }
}

#[test]
fn opt_remove_unreachable_2() {
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
        config.seed = i;
        let root = RootCircuit::<C>::random(&config);
        assert_eq!(root.validate(), Ok(()));
        let (optroot, im) = root.remove_unreachable();
        assert_eq!(im.cur_size(), root.input_size());
        assert_eq!(optroot.validate(), Ok(()));
        let inputs: Vec<CField> = (0..root.input_size())
            .map(|_| CField::random_unsafe())
            .collect();
        let (out1, cond1) = root.eval_unsafe(inputs.clone());
        let (out2, cond2) = optroot.eval_unsafe(im.map_inputs(&inputs));
        assert_eq!(out1, out2);
        assert_eq!(cond1, cond2);
    }
}