use circuit_std_rs::poseidon_m31::*;
use expander_compiler::frontend::*;
use extra::*;

declare_circuit!(PoseidonElementCircuit {
    inputs: [Variable; 16],
    outputs: [Variable; POSEIDON_HASH_LENGTH],
});

impl GenericDefine<M31Config> for PoseidonElementCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let outputs =
            poseidon_elements_hint(builder, &PoseidonParams::new(), self.inputs.to_vec(), false);
        for i in 0..POSEIDON_HASH_LENGTH {
            // println!("i: {}, outputs[i]: {:?}, expect:{:?}", i, builder.value_of(outputs[i]),  builder.value_of(self.outputs[i]));
            builder.assert_is_equal(self.outputs[i], outputs[i]);
        }
    }
}

#[test]
fn test_poseidon_element() {
    let mut input = vec![0; 31];
    for i in 0..input.len() {
        input[i] = i as u64;
    }
    let param = PoseidonParams::new();
    let output = poseidon_elements_unsafe(&param, input, false);
    println!("{:?}", output);
}

#[test]
fn test_poseidon_element_circuit() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.poseidonhint", poseidon_hint);
    let mut input = vec![];
    for i in 0..16 {
        input.push(i as u64);
    }
    let output = poseidon_elements_unsafe(&PoseidonParams::new(), input.clone(), false);
    let mut assignment = PoseidonElementCircuit::default();
    for i in 0..16 {
        assignment.inputs[i] = M31::from(input[i].clone() as u32);
    }
    for i in 0..POSEIDON_HASH_LENGTH {
        assignment.outputs[i] = M31::from(output[i] as u32);
    }

    debug_eval(
        &PoseidonElementCircuit::default(),
        &assignment,
        hint_registry,
    );
}
