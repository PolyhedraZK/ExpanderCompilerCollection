use expander_compiler::frontend::M31Config;
use expander_compiler::frontend::*;
use expander_compiler::zkcuda::kernel::Kernel;
use expander_compiler::zkcuda::kernel::*;
use circuit_std_rs::logup::LogUpSingleKeyTable;
use circuit_std_rs::poseidon_m31::{PoseidonM31Params, POSEIDON_M31X16_FULL_ROUNDS, POSEIDON_M31X16_PARTIAL_ROUNDS, POSEIDON_M31X16_RATE};
use circuit_std_rs::sha256::m31_utils::from_binary;
use circuit_std_rs::utils::{simple_lookup2, simple_select};
use crate::permutation::{QUERY_SIZE, TABLE_SIZE, VALIDATOR_COUNT};
use crate::utils::sub_vector;

fn verify_permutation_hash_inner<C: Config>(api: &mut API<C>, p: &Vec<Variable>) -> Vec<Variable> {
    let index = &p[..TABLE_SIZE];
    let value = &p[TABLE_SIZE..TABLE_SIZE*2];
    let table = &p[TABLE_SIZE*2..TABLE_SIZE*3];
    let mut lutable = LogUpSingleKeyTable::new(8);
    let mut table_key = vec![];
    for i in 0..TABLE_SIZE {
        table_key.push(api.constant(i as u32));
    }
    let mut table_values = vec![];
    for i in 0..TABLE_SIZE {
        table_values.push(vec![table[i]]);
    }
    lutable.new_table(table_key, table_values);
    let mut query_values = vec![];
    for i in 0..TABLE_SIZE {
        query_values.push(vec![value[i]]);
    }
    lutable.batch_query(index.to_vec(), query_values);
    //m31 field, repeat 3 times
    lutable.final_check(api);
    lutable.final_check(api);
    lutable.final_check(api);

    return vec![api.constant(1)];
}


fn verify_permutation_indices_validator_hashes_inner<C: Config>(api: &mut API<C>, p: &Vec<Variable>) -> Vec<Variable> {
   
    let (query_indices, pos) = sub_vector(p, 0, QUERY_SIZE);
    let ( query_validator_hashes, pos) = sub_vector(p, pos, POSEIDON_M31X16_RATE*QUERY_SIZE);
    let (active_validator_bits_hash, pos) = sub_vector(p, pos, QUERY_SIZE);
    let (active_validator_bits, pos) = sub_vector(p, pos, VALIDATOR_COUNT);
    let (table_validator_hashes, pos) = sub_vector(p, pos, POSEIDON_M31X16_RATE*VALIDATOR_COUNT);
    let (real_keys, pos) = sub_vector(p, pos, VALIDATOR_COUNT);

    let zero_var = api.constant(0);
    let neg_one_count = api.sub(1, VALIDATOR_COUNT as u32);
    //check the activeValidatorBitsHash
    if active_validator_bits.len() % 16 != 0 {
        panic!("activeValidatorBits length must be multiple of 16")
    }
    let mut active_validator_16_bits = vec![];
    for i in 0..VALIDATOR_COUNT / 16 {
        active_validator_16_bits.push(from_binary(
            api,
            active_validator_bits[i * 16..(i + 1) * 16].to_vec(),
        ));
    }
    let params = PoseidonM31Params::new(
        api,
        POSEIDON_M31X16_RATE,
        16,
        POSEIDON_M31X16_FULL_ROUNDS,
        POSEIDON_M31X16_PARTIAL_ROUNDS,
    );
    let active_validator_hash = params.hash_to_state(api, &active_validator_16_bits);
    for (i, active_validator_hashbit) in active_validator_hash
        .iter()
        .enumerate()
        .take(POSEIDON_M31X16_RATE)
    {
        api.assert_is_equal(active_validator_hashbit, active_validator_bits_hash[i]);
    }
    //move inactive validators to the end
    let mut sorted_table_key = [Variable::default(); VALIDATOR_COUNT];
    sorted_table_key[..VALIDATOR_COUNT].copy_from_slice(&real_keys[..VALIDATOR_COUNT]); //if active, use curKey, else use curInactiveKey
    //for the first one, if active, use 0, else use -ValidatorCount
    let shift = simple_select(
        api,
        active_validator_bits[0],
        zero_var,
        neg_one_count,
    );
    let shift_key = api.add(sorted_table_key[0], shift);
    let shift_key_zero = api.is_zero(shift_key);
    api.assert_is_equal(shift_key_zero, 1); //the first key must be 0 or ValidatorCount-1
    for i in 1..VALIDATOR_COUNT {
        //for every validator, its key can be
        //active and active: previous key + 1
        //active and inactive: previous key - ValidatorCount + 1
        //inactive and active: previous key + ValidatorCount
        //inactive and inactive: previous key
        //1 1 --> previous key + 1
        //1 0 --> previous key - ValidatorCount + 1
        //0 1 --> previous key + ValidatorCount
        //0 0 --> previous key
        let previous_plus_one = api.add(sorted_table_key[i - 1], 1);
        let previous_minus_count_plus_one =
            api.sub(previous_plus_one, VALIDATOR_COUNT as u32);
        let previous_plus_count = api.add(sorted_table_key[i - 1], VALIDATOR_COUNT as u32);
        let expected_key = simple_lookup2(
            api,
            active_validator_bits[i - 1],
            active_validator_bits[i],
            sorted_table_key[i - 1],
            previous_plus_count,
            previous_minus_count_plus_one,
            previous_plus_one,
        );
        //if current one is active, the diff must be 1. Otherwise, the diff must be 0. That is, always equal to activeValidatorBits[i]
        let diff = api.sub(expected_key, sorted_table_key[i]);
        let diff_zero = api.is_zero(diff);
        api.assert_is_equal(diff_zero, 1);
    }
    //logup
    let mut logup = LogUpSingleKeyTable::new(8);
    let mut table_values = vec![];
    for i in 0..VALIDATOR_COUNT {
        table_values.push(table_validator_hashes[i*POSEIDON_M31X16_RATE..(i+1)*POSEIDON_M31X16_RATE].to_vec());
    }
    //build a table with sorted key, i.e., the inactive validators have been moved to the end
    logup.new_table(sorted_table_key.to_vec(), table_values);
    //logup
    let mut query_values = vec![];
    for i in 0..QUERY_SIZE {
        query_values.push(query_validator_hashes[i*POSEIDON_M31X16_RATE..(i+1)*POSEIDON_M31X16_RATE].to_vec());
    }
    logup.batch_query(query_indices.to_vec(), query_values);
    logup.final_check(api);
    logup.final_check(api);
    logup.final_check(api);
    return vec![api.constant(1)];
}

#[kernel]
fn verify_permutation_hash<C: Config>(
    api: &mut API<C>,
    input: &[InputVariable; TABLE_SIZE*3],
    output: &mut OutputVariable,
) {
    let outc = api.memorized_simple_call(verify_permutation_hash_inner, input);
    *output = outc[0]
}

#[kernel]
fn verify_permutation_indices_validator_hashes<C: Config>(
    api: &mut API<C>,
    input: &[InputVariable; 29362176],
    output: &mut OutputVariable,
) {
    let outc = api.memorized_simple_call(verify_permutation_indices_validator_hashes_inner, input);
    *output = outc[0]
}

#[test]
fn test_zkcuda_permutation_hash() {
    let _: Kernel<M31Config> = compile_verify_permutation_hash().unwrap();
    println!("compile ok");
}

// #[test]
// fn test_zkcuda_permutation_indices_validator_hashes() {
//     let _: Kernel<M31Config> = compile_verify_permutation_indices_validator_hashes().unwrap();
//     println!("compile ok");
// }