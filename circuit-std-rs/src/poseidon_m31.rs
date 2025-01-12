use extra::*;
use expander_compiler::frontend::*;

const P:u64 = 2147483647;
const POSEIDON_HASH_LENGTH:usize = 8;
pub struct PoseidonParams {
    pub num_full_rounds: usize,
    pub num_half_full_rounds: usize,
    pub num_part_rounds: usize,
    pub num_half_partial_rounds: usize,
    pub num_states: usize,
    pub mds_matrix: Vec<Vec<u32>>,
    pub external_round_constant: Vec<Vec<u32>>,
    pub internal_round_constant: Vec<u32>,
}
impl PoseidonParams{
    pub fn new() -> PoseidonParams {
        let mut rng = rand::thread_rng();
        let num_full_rounds = 8;
        let num_part_rounds = 14;
        let num_states = 16;
        let external_round_constant = vec![
            vec![1602144611, 283469975, 447079688, 896869500, 188198846, 1645802691, 1343797066, 1651182352], 
            vec![1645164257, 628681638, 1012020211, 936051935, 1553415631, 520713656, 704421656, 2002329306], 
            vec![912104457, 927309554, 1896963422, 2021987360, 512526164, 1461005868, 1230890889, 975482550], 
            vec![655249440, 180293723, 1833042356, 558181838, 1935993014, 906657358, 1577735965, 952246556], 
            vec![1686276583, 2135981572, 301879258, 2033580949, 1875784301, 1528511887, 651033144, 546509366], 
            vec![1044572822, 1099335126, 1096147649, 244154321, 878215424, 308591376, 1437140741, 187187417],
            vec![244141211, 1241082190, 802875957, 758482785, 1882334571, 1260030334, 140962496, 1004550553], 
            vec![1946499876, 1224110803, 167959573, 1112131775, 1777017360, 794128708, 719742823, 964768064], 
            vec![260466031, 1280875791, 1311686993, 334003808, 1708822585, 1575283902, 1085939132, 853116326], 
            vec![1948693740, 1676460526, 592034112, 175515361, 1925748203, 423003568, 2064012607, 1607483085], 
            vec![1781173630, 1992696998, 1172364027, 1007563471, 1904034292, 888768061, 1369442223, 671255092], 
            vec![919393366, 1262786628, 1174193565, 2120008749, 1514449614, 841831014, 741479278, 124916655], 
            vec![1418908281, 1941555103, 1776135713, 340191556, 59007516, 2070331360, 1475967456, 597566110], 
            vec![1802709549, 1794127988, 126858077, 1413234214, 1033418513, 593787007, 1588170737, 658446415], 
            vec![657013279, 1078572839, 1394546710, 1029801709, 1397378755, 6060087, 474304723, 1469825109], 
            vec![780675426, 359271844, 1731998572, 801241649, 926841567, 2109531454, 275946491, 1847955509]
        ];
        let internal_round_constant = vec![477367687, 1846273021, 220762869, 2068417058, 618183629, 1079533163, 1835801580, 1649855374, 1027781798, 670016125, 1011893799, 664483678, 669708402, 787762663];
        let mut mds = vec![vec![0; 16]; num_states];
        mds[0] = vec![1, 1, 51, 1, 11, 17, 2, 1, 101, 63, 15, 2, 67, 22, 13, 3];
        for i in 1..16 {
            mds[i] = vec![0; 16];
            for j in 0..16 {
                mds[i][j] = mds[0][(j+i)%16];
            }
        }
        PoseidonParams {
            num_full_rounds,
            num_half_full_rounds: num_full_rounds / 2,
            num_part_rounds,
            num_half_partial_rounds: num_part_rounds / 2,
            num_states,
            mds_matrix: mds,
            external_round_constant,
            internal_round_constant,
        }
    }
}
#[derive(Default)]
pub struct PoseidonInternalState {
    after_half_full_round: Vec<u64>,
    after_half_partial_round: Vec<u64>,
    after_partial_round: Vec<u64>,
}
pub fn padding_zeros_poseidon_input_element(input: Vec<u64>, num_states: usize) -> Vec<u64> {
    let mut input = input;
    while input.len() % num_states != 0 {
        input.push(0);
    }
    input
}
pub fn poseidon_elements_unsafe(param: &PoseidonParams, input: Vec<u64>, with_state: bool) -> Vec<u64> {
    let mut input = padding_zeros_poseidon_input_element(input, param.num_states);

    while input.len() >= param.num_states {
        input = padding_zeros_poseidon_input_element(input, param.num_states);
        for i in 0..input.len()/param.num_states {
            let mut state = vec![0; param.num_states];
            state.copy_from_slice(&input[i*param.num_states..(i+1)*param.num_states]);
            let output = poseidon_m31_with_internal_states(param, state, with_state);
            input[i*POSEIDON_HASH_LENGTH..(i+1)*POSEIDON_HASH_LENGTH].copy_from_slice(&output[..POSEIDON_HASH_LENGTH]);
        }
        input = input[..input.len()/2].to_vec();
    }
    input[..POSEIDON_HASH_LENGTH].to_vec()
}
pub fn poseidon_m31_with_internal_states(param: &PoseidonParams, input: Vec<u64>, with_state: bool) -> Vec<u64> {
    if input.len() != param.num_states {
        panic!("input length does not match the number of states in the Poseidon parameters");
    }
    let mut state = input;
    let mut internal_state = PoseidonInternalState::default();
    for i in 0..param.num_half_full_rounds {
        for j in 0..param.num_states {
            state[j] = (state[j] + param.external_round_constant[j][i] as u64) % P;
        }
        state = apply_mds_matrix(state, &param.mds_matrix);
        for j in 0..param.num_states {
            state[j] = s_box(state[j]);
        }
    }
    if with_state {
        internal_state.after_half_full_round.copy_from_slice(&state);
    }
    for i in 0..param.num_half_partial_rounds {
        state[0] = (state[0] +  param.internal_round_constant[i] as u64) % P;
        state = apply_mds_matrix(state, &param.mds_matrix);
        state[0] = s_box(state[0]);
    }
    if with_state {
        internal_state.after_half_partial_round.copy_from_slice(&state);
    }
    for i in 0..param.num_half_partial_rounds {
        state[0] = (state[0] + param.internal_round_constant[i+param.num_half_partial_rounds] as u64) % P;
        state = apply_mds_matrix(state, &param.mds_matrix);
        state[0] = s_box(state[0]);
    }
    if with_state {
        internal_state.after_partial_round.copy_from_slice(&state);
    }
    for i in 0..param.num_half_full_rounds {
        for j in 0..param.num_states {
            state[j] = (state[j] + param.external_round_constant[j][i+param.num_half_full_rounds] as u64) % P;
        }
        state = apply_mds_matrix(state, &param.mds_matrix);
        for j in 0..param.num_states {
            state[j] = s_box(state[j]);
        }
    }
    state
}

pub fn apply_mds_matrix(state: Vec<u64>, mds: &Vec<Vec<u32>>) -> Vec<u64> {
    let mut tmp = vec![0; state.len()];
    for i in 0..state.len() {
        tmp[i] = state[0] * mds[i][0] as u64;
        for j in 1..state.len() {
            tmp[i] = tmp[i] + state[j] *  mds[i][j] as u64;
            tmp[i] = tmp[i] % P;
        }
    }
    tmp
}
pub fn s_box(f: u64) -> u64 {
    let mut x2 = f * f;
    x2 %= P;
    let mut x4 = x2 * x2;
    x4 %= P;
    (x4 * f) % P
}

pub fn poseidon_elements_hint<C: Config, B: RootAPI<C>>(native: &mut B, _: &PoseidonParams, inputs: Vec<Variable>, _: bool) -> Vec<Variable> {
    native.new_hint("myhint.poseidonhint",  &inputs, POSEIDON_HASH_LENGTH)
}

pub fn poseidon_hint(inputs: &[M31], outputs: &mut [M31]) -> Result<(), Error> {
    let mut inputs_u64 = vec![0;inputs.len()];
    for i in 0..inputs.len() {
        inputs_u64[i] = inputs[i].to_u256().as_u64();
    }
    let param = PoseidonParams::new();
    let output = poseidon_elements_unsafe(&param, inputs_u64, false);
    for i in 0..output.len() {
        outputs[i] = M31::from(output[i] as u32);
    }
    Ok(())
}

declare_circuit!(PoseidonElementCircuit{
    inputs: [Variable; 16],
    outputs: [Variable; POSEIDON_HASH_LENGTH],
}
);

impl GenericDefine<M31Config> for PoseidonElementCircuit<Variable> {
	fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let outputs = poseidon_elements_hint(builder, &PoseidonParams::new(), self.inputs.to_vec(), false);
        for i in 0..POSEIDON_HASH_LENGTH {
            // println!("i: {}, outputs[i]: {:?}, expect:{:?}", i, builder.value_of(outputs[i]),  builder.value_of(self.outputs[i]));
            builder.assert_is_equal(self.outputs[i], outputs[i]);
        }
    }
}

    
#[test]
fn test_poseidon_element(){
    let mut input = vec![0;31];
    for i in 0..input.len() {
        input[i] = i as u64;
    }
    let param = PoseidonParams::new();
    let output = poseidon_elements_unsafe(&param, input, false);
    println!("{:?}", output);
}

#[test]
fn test_poseidon_element_circuit(){
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