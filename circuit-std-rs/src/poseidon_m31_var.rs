use std::cell::RefCell;
use extra::*;
use expander_compiler::frontend::*;
use sha2::{Digest, Sha256};
use crate::{big_int::{big_array_add, big_endian_m31_array_put_uint32, bit_array_to_m31, bytes_to_bits, cap_sigma0, cap_sigma1, ch, m31_to_bit_array, maj, sigma0, sigma1, to_binary_hint}, poseidon_m31::{poseidon_elements_unsafe, PoseidonParams}};

const P:u64 = 2147483647;
const PoseidonHashLength:usize = 8;
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
            input[i*PoseidonHashLength..(i+1)*PoseidonHashLength].copy_from_slice(&output[..PoseidonHashLength]);
        }
        input = input[..input.len()/2].to_vec();
    }
    input[..PoseidonHashLength].to_vec()
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
    outputs: [Variable; PoseidonHashLength],
}
);

impl GenericDefine<M31Config> for PoseidonCircuit<Variable> {
	fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let outputs = poseidon_variable_unsafe(builder, &PoseidonParams::new(), self.inputs.to_vec(), false);
        for i in 0..PoseidonHashLength {
            builder.assert_is_equal(self.outputs[i], outputs[i]);
        }
    }
}


#[test]
fn test_poseidon_circuit(){
	let mut hint_registry = HintRegistry::<M31>::new();
    let mut input = vec![];
    for i in 0..64 {
        input.push(i as u64);
    }
    let output = poseidon_elements_unsafe(&PoseidonParams::new(), input.clone(), false);
    let mut assignment = PoseidonCircuit::default();
    for i in 0..64 {
        assignment.inputs[i] = M31::from(input[i].clone() as u32);
    }
    for i in 0..PoseidonHashLength {
        assignment.outputs[i] = M31::from(output[i] as u32);
    }
    

	debug_eval(
        &PoseidonCircuit::default(),
        &assignment,
        hint_registry,
    );
}