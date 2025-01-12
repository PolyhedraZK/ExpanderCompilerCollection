use extra::*;
use expander_compiler::frontend::*;
use crate::poseidon_m31::{poseidon_elements_unsafe, PoseidonParams};

const POSEIDON_HASH_LENGTH:usize = 8;
#[derive(Default)]
pub struct PoseidonInternalState {
    after_half_full_round: Vec<Variable>,
    after_half_partial_round: Vec<Variable>,
    after_partial_round: Vec<Variable>,
}
pub fn padding_zeros_poseidon_input_variable<C: Config, B: RootAPI<C>>(builder: &mut B, input: Vec<Variable>, num_states: usize) -> Vec<Variable> {
    let mut input = input;
    let zero_var = builder.constant(0);
    while input.len() % num_states != 0 {
        input.push(zero_var.clone());
    }
    input
}
pub fn poseidon_variable_unsafe<C: Config, B: RootAPI<C>>(builder: &mut B, param: &PoseidonParams, input: Vec<Variable>, with_state: bool) -> Vec<Variable> {
    let mut input = padding_zeros_poseidon_input_variable(builder, input, param.num_states);

    while input.len() >= param.num_states {
        input = padding_zeros_poseidon_input_variable(builder, input, param.num_states);
        for i in 0..input.len()/param.num_states {
            let mut state = vec![Variable::default(); param.num_states];
            state.copy_from_slice(&input[i*param.num_states..(i+1)*param.num_states]);
            let output = poseidon_m31_with_internal_states_variable(builder, param, state, with_state);
            input[i*POSEIDON_HASH_LENGTH..(i+1)*POSEIDON_HASH_LENGTH].copy_from_slice(&output[..POSEIDON_HASH_LENGTH]);
        }
        input = input[..input.len()/2].to_vec();
    }
    input[..POSEIDON_HASH_LENGTH].to_vec()
}
pub fn poseidon_m31_with_internal_states_variable<C: Config, B: RootAPI<C>>(builder: &mut B, param: &PoseidonParams, input: Vec<Variable>, with_state: bool) -> Vec<Variable> {
    if input.len() != param.num_states {
        panic!("input length does not match the number of states in the Poseidon parameters");
    }
    let mut state = input;
    let mut internal_state = PoseidonInternalState::default();
    for i in 0..param.num_half_full_rounds {
        for j in 0..param.num_states {
            state[j] = builder.add(state[j], param.external_round_constant[j][i]);
        }
        state = apply_mds_matrix(builder, state, &param.mds_matrix);
        for j in 0..param.num_states {
            state[j] = s_box(builder, state[j]);
        }
    }
    if with_state {
        internal_state.after_half_full_round.copy_from_slice(&state);
    }
    for i in 0..param.num_half_partial_rounds {
        state[0] = builder.add(state[0], param.internal_round_constant[i]);
        state = apply_mds_matrix(builder, state, &param.mds_matrix);
        state[0] = s_box(builder, state[0]);
    }
    if with_state {
        internal_state.after_half_partial_round.copy_from_slice(&state);
    }
    for i in 0..param.num_half_partial_rounds {
        state[0] = builder.add(state[0], param.internal_round_constant[i+param.num_half_partial_rounds]);
        state = apply_mds_matrix(builder, state, &param.mds_matrix);
        state[0] = s_box(builder, state[0]);
    }
    if with_state {
        internal_state.after_partial_round.copy_from_slice(&state);
    }
    for i in 0..param.num_half_full_rounds {
        for j in 0..param.num_states {
            state[j] = builder.add(state[j], param.external_round_constant[j][i+param.num_half_full_rounds]);
        }
        state = apply_mds_matrix(builder, state, &param.mds_matrix);
        for j in 0..param.num_states {
            state[j] = s_box(builder, state[j]);
        }
    }
    state
}

pub fn apply_mds_matrix<C: Config, B: RootAPI<C>>(builder: &mut B, state: Vec<Variable>, mds: &Vec<Vec<u32>>) -> Vec<Variable> {
    let mut tmp = vec![Variable::default(); state.len()];
    for i in 0..state.len() {
        tmp[i] = builder.mul(state[0].clone(), mds[i][0]);
        for j in 1..state.len() {
            let tmp2 = builder.mul(state[j].clone(), mds[i][j]);
            tmp[i] = builder.add(tmp[i], tmp2);
        }
    }
    tmp
}
pub fn s_box<C: Config, B: RootAPI<C>>(builder: &mut B, f: Variable) -> Variable {
    let x2 = builder.mul(f.clone(), f.clone());
    let x4 = builder.mul(x2.clone(), x2.clone());
    builder.mul(x4, f)
}

declare_circuit!(PoseidonCircuit{
    inputs: [Variable; 64],
    outputs: [Variable; POSEIDON_HASH_LENGTH],
}
);

impl GenericDefine<M31Config> for PoseidonCircuit<Variable> {
	fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let outputs = poseidon_variable_unsafe(builder, &PoseidonParams::new(), self.inputs.to_vec(), false);
        for i in 0..POSEIDON_HASH_LENGTH {
            builder.assert_is_equal(self.outputs[i], outputs[i]);
        }
    }
}


#[test]
fn test_poseidon_circuit(){
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
    

	debug_eval(
        &PoseidonCircuit::default(),
        &assignment,
        hint_registry,
    );
}