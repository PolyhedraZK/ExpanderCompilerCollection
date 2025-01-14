use circuit_std_rs::{
    poseidon_m31::{poseidon_elements_unsafe, PoseidonParams, POSEIDON_HASH_LENGTH},
    poseidon_m31_var::poseidon_variable_unsafe,
};
use expander_compiler::frontend::*;
use extra::*;

declare_circuit!(PoseidonCircuit {
    inputs: [Variable; 64],
    outputs: [Variable; POSEIDON_HASH_LENGTH],
});

impl GenericDefine<M31Config> for PoseidonCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let outputs =
            poseidon_variable_unsafe(builder, &PoseidonParams::new(), self.inputs.to_vec(), false);
        for i in 0..POSEIDON_HASH_LENGTH {
            builder.assert_is_equal(self.outputs[i], outputs[i]);
        }
    }
}

#[test]
fn test_poseidon_circuit() {
    let hint_registry = HintRegistry::<M31>::new();
    let mut input = vec![];
    for i in 0..64 {
        input.push(i as u64);
    }
    let output = poseidon_elements_unsafe(&PoseidonParams::new(), input.clone(), false);
    let mut assignment = PoseidonCircuit::default();
    for i in 0..64 {
        assignment.inputs[i] = M31::from(input[i].clone() as u32);
    }
    for i in 0..POSEIDON_HASH_LENGTH {
        assignment.outputs[i] = M31::from(output[i] as u32);
    }

    debug_eval(&PoseidonCircuit::default(), &assignment, hint_registry);
}
