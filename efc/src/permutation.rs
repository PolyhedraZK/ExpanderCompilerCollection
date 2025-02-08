use crate::utils::{ensure_directory_exists, read_from_json_file};
use circuit_std_rs::logup::LogUpSingleKeyTable;
use circuit_std_rs::poseidon_m31::*;
use circuit_std_rs::sha256::m31_utils::*;
use circuit_std_rs::utils::{register_hint, simple_lookup2, simple_select};
use expander_compiler::circuit::ir::hint_normalized::witness_solver;
use expander_compiler::frontend::extra::*;
use expander_compiler::frontend::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

pub const TABLE_SIZE: usize = 1024;
declare_circuit!(PermutationHashCircuit {
    index: [Variable; TABLE_SIZE],
    value: [Variable; TABLE_SIZE],
    table: [Variable; TABLE_SIZE],
});
impl PermutationHashCircuit<M31> {
    pub fn from_entry(
        hashtable_bits: &[Vec<u8>],
        query_indices: &[Vec<u64>],
        query_bits: &[Vec<u8>],
        hashtable_size: usize,
        row: usize,
    ) -> Self {
        let mut assignment = PermutationHashCircuit {
            index: [M31::from(0); TABLE_SIZE],
            value: [M31::from(0); TABLE_SIZE],
            table: [M31::from(0); TABLE_SIZE],
        };

        for j in 0..TABLE_SIZE {
            assignment.table[j] = M31::from(0);
            if j < hashtable_size {
                assignment.table[j] = M31::from(hashtable_bits[row][j] as u32);
            }
        }
        for j in 0..TABLE_SIZE {
            // Initialize with zero.
            assignment.index[j] = M31::from(0);
            assignment.value[j] = M31::from(0);
            if j < hashtable_size {
                assignment.index[j] = M31::from(query_indices[row][j] as u32);
                assignment.value[j] = M31::from(query_bits[row][j] as u32);
            }
        }
        assignment
    }
}

impl GenericDefine<M31Config> for PermutationHashCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut table = LogUpSingleKeyTable::new(8);
        let mut table_key = vec![];
        for i in 0..TABLE_SIZE {
            table_key.push(builder.constant(i as u32));
        }
        let mut table_values = vec![];
        for i in 0..TABLE_SIZE {
            table_values.push(vec![self.table[i]]);
        }
        table.new_table(table_key, table_values);
        let mut query_values = vec![];
        for i in 0..TABLE_SIZE {
            query_values.push(vec![self.value[i]]);
        }
        table.batch_query(self.index.to_vec(), query_values);
        //m31 field, repeat 3 times
        table.final_check(builder);
        table.final_check(builder);
        table.final_check(builder);
    }
}

#[test]
fn test_permutation_hash() {
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = PermutationHashCircuit::<M31> {
        index: [M31::from(0); TABLE_SIZE],
        value: [M31::from(0); TABLE_SIZE],
        table: [M31::from(0); TABLE_SIZE],
    };
    for i in 0..TABLE_SIZE {
        assignment.index[i] = M31::from(i as u32);
        assignment.value[i] = M31::from((i as u32 + 571) * 79);
        assignment.table[i] = M31::from((i as u32 + 571) * 79);
    }
    debug_eval(
        &PermutationHashCircuit::default(),
        &assignment,
        hint_registry,
    );
}

pub const QUERY_SIZE: usize = 1024 * 1024;
pub const VALIDATOR_COUNT: usize = QUERY_SIZE * 2;
declare_circuit!(PermutationIndicesValidatorHashesCircuit {
    query_indices: [Variable; QUERY_SIZE],
    query_validator_hashes: [[Variable; POSEIDON_M31X16_RATE]; QUERY_SIZE],
    active_validator_bits_hash: [Variable; POSEIDON_M31X16_RATE],
    active_validator_bits: [Variable; VALIDATOR_COUNT],
    table_validator_hashes: [[Variable; POSEIDON_M31X16_RATE]; VALIDATOR_COUNT],
    real_keys: [Variable; VALIDATOR_COUNT],
});
impl PermutationIndicesValidatorHashesCircuit<M31> {
    pub fn from_entry(
        real_keys: &[u64],                      
        active_validator_bits_hash: &[u32],  
        active_validator_bits: &[u64], 
        validator_hashes: &[Vec<u32>],
        shuffle_indices: &[u64],  
        committee_indices: &[u64],
        valid_validator_list: &[u64], 
    ) -> Self {
        let mut assignment = PermutationIndicesValidatorHashesCircuit {
            query_indices: [M31::from(0); QUERY_SIZE],
            query_validator_hashes: [[M31::from(0); POSEIDON_M31X16_RATE]; QUERY_SIZE],
            active_validator_bits_hash: [M31::from(0); POSEIDON_M31X16_RATE],
            active_validator_bits: [M31::from(0); VALIDATOR_COUNT],
            table_validator_hashes: [[M31::from(0); POSEIDON_M31X16_RATE]; VALIDATOR_COUNT],
            real_keys: [M31::from(0); VALIDATOR_COUNT],
        };
        for i in 0..VALIDATOR_COUNT {
            assignment.real_keys[i] = M31::from(real_keys[i] as u32);
        }
        for i in 0..POSEIDON_M31X16_RATE {
            assignment.active_validator_bits_hash[i] = M31::from(active_validator_bits_hash[i]);
        }

        for i in 0..VALIDATOR_COUNT {
            assignment.active_validator_bits[i] = M31::from(0);
            if i < validator_hashes.len() {
                assignment.active_validator_bits[i] = M31::from(active_validator_bits[i] as u32);
            }
            for j in 0..POSEIDON_M31X16_RATE {
                assignment.table_validator_hashes[i][j] = M31::from(0);
                if i < validator_hashes.len() {
                    assignment.table_validator_hashes[i][j] = M31::from(validator_hashes[i][j]);
                }
            }
        }

        if shuffle_indices.len() != committee_indices.len() {
            panic!("The length of shuffleIndices is not equal to the length of committeeIndices");
        }

        for i in 0..QUERY_SIZE {
            assignment.query_indices[i] = M31::from(0);
            if i < shuffle_indices.len() {
                assignment.query_indices[i] = M31::from(shuffle_indices[i] as u32);
            }
            for j in 0..POSEIDON_M31X16_RATE {
                assignment.query_validator_hashes[i][j] = M31::from(0);
                if i < shuffle_indices.len() {
                    let valid_idx = valid_validator_list[shuffle_indices[i] as usize] as usize;
                    assignment.query_validator_hashes[i][j] =
                        M31::from(validator_hashes[valid_idx][j] as u32);
                }
            }
        }

        assignment
    }
}

impl PermutationIndicesValidatorHashesCircuit<M31> {
    pub fn from_assignment(entry: &PermutationHashEntry) -> Self {
        let mut assignment = PermutationIndicesValidatorHashesCircuit {
            query_indices: [M31::from(0); QUERY_SIZE],
            query_validator_hashes: [[M31::from(0); POSEIDON_M31X16_RATE]; QUERY_SIZE],
            active_validator_bits_hash: [M31::from(0); POSEIDON_M31X16_RATE],
            active_validator_bits: [M31::from(0); VALIDATOR_COUNT],
            table_validator_hashes: [[M31::from(0); POSEIDON_M31X16_RATE]; VALIDATOR_COUNT],
            real_keys: [M31::from(0); VALIDATOR_COUNT],
        };
        for i in 0..VALIDATOR_COUNT {
            for j in 0..POSEIDON_M31X16_RATE {
                assignment.table_validator_hashes[i][j] =
                    M31::from(entry.table_validator_hashes[i][j]);
            }
            assignment.real_keys[i] = M31::from(entry.real_keys[i]);
            assignment.active_validator_bits[i] =
                M31::from(entry.active_validator_bits[i]);
        }
        for i in 0..QUERY_SIZE {
            assignment.query_indices[i] = M31::from(entry.query_indices[i]);
            for j in 0..POSEIDON_M31X16_RATE {
                assignment.query_validator_hashes[i][j] =
                    M31::from(entry.query_validator_hashes[i][j]);
            }
        }
        for i in 0..POSEIDON_M31X16_RATE {
            assignment.active_validator_bits_hash[i] =
                M31::from(entry.active_validator_bits_hash[i]);
        }
        assignment
    }
}

pub fn distribute_sub_assignment2(
    hashtable_bits: Vec<Vec<u8>>,
    raw_query_bits: Vec<Vec<u8>>,
    raw_query_indices: Vec<Vec<u64>>,
    valid_validator_list: Vec<u64>,
    raw_shuffle_indices: Vec<u64>,
    raw_committee_indices: Vec<u64>,
    real_committee_size: Vec<u64>,
    padding_size: usize,
    slot: u64,
    validator_hashes: Vec<Vec<u32>>,
    active_validator_bits_hash: Vec<u32>,
) -> (Vec<PermutationHashCircuit<M31>>, PermutationIndicesValidatorHashesCircuit<M31>) {
    if raw_query_bits.is_empty() {
        panic!("rawQueryBits is empty");
    }

    let mut tran_query_bits: Vec<Vec<u8>> = (0..raw_query_bits[0].len())
    .map(|i| raw_query_bits.iter().map(|row| row[i]).collect())
    .collect();

    let mut tran_query_indices: Vec<Vec<u64>> = (0..raw_query_indices[0].len())
        .map(|i| raw_query_indices.iter().map(|row| row[i]).collect())
        .collect();


    // padding
    let last_committee_size = *real_committee_size
        .last()
        .expect("real_committee_size is not empty") as usize;
    let to_pad = padding_size - last_committee_size;
    for i in 0..tran_query_bits.len() {
        let pad_bits = vec![hashtable_bits[i][0]; to_pad];
        tran_query_bits[i].extend(pad_bits);
        tran_query_indices[i].extend(vec![0u64; to_pad]);
    }

    // chunks
    let mut query_bits: Vec<Vec<u8>> = Vec::with_capacity(tran_query_bits.len());
    let mut query_indices: Vec<Vec<u64>> = Vec::with_capacity(tran_query_indices.len());

    for i in 0..tran_query_bits.len() {
        let mut bits_vec = Vec::new();
        let mut indices_vec = Vec::new();
        let mut start = 0;
        for &real_size in &real_committee_size {
            let end = std::cmp::min(start + padding_size, tran_query_bits[i].len());
            bits_vec.extend_from_slice(&tran_query_bits[i][start..end]);
            indices_vec.extend_from_slice(&tran_query_indices[i][start..end]);
            start += real_size as usize;
        }
        query_bits.push(bits_vec);
        query_indices.push(indices_vec);
    }

    // padding raw_shuffle_indices and raw_committee_indices
    let mut pad_shuffle_indices = raw_shuffle_indices.clone();
    let mut pad_committee_indices = raw_committee_indices.clone();
    pad_shuffle_indices.extend(vec![0u64; to_pad]);
    pad_committee_indices.extend(vec![0u64; to_pad]);
    // use the first element to pad
    for i in 0..to_pad {
        let idx_shuffle = raw_shuffle_indices.len() + i;
        pad_shuffle_indices[idx_shuffle] = raw_shuffle_indices[0];

        let idx_committee = raw_committee_indices.len() + i;
        pad_committee_indices[idx_committee] = raw_committee_indices[0];
    }

    // construct shuffle_indices and committee_indices
    let mut shuffle_indices: Vec<u64> = Vec::new();
    let mut committee_indices: Vec<u64> = Vec::new();
    let mut start = 0;
    for &real_size in &real_committee_size {
        let end_shuffle = std::cmp::min(start + padding_size, pad_shuffle_indices.len());
        let end_committee = std::cmp::min(start + padding_size, pad_committee_indices.len());
        shuffle_indices.extend_from_slice(&pad_shuffle_indices[start..end_shuffle]);
        committee_indices.extend_from_slice(&pad_committee_indices[start..end_committee]);
        start += real_size as usize;
    }

    let hashtable_num = query_bits.len();
    if query_indices.len() != hashtable_num {
        panic!("queryIndices length is not equal to queryBits length");
    }

    let hashtable_size = query_bits[0].len();
    // ensure each has the same size.
    for (i, (bits, indices)) in query_bits.iter().zip(query_indices.iter()).enumerate() {
        if bits.len() != hashtable_size || indices.len() != hashtable_size {
            panic!(
                "The length of queryBits[{}] or queryIndices[{}] is not equal to hashtableSize",
                i, i
            );
        }
    }

    // Check if the hashtable size exceeds the allowed TABLE_SIZE.
    if hashtable_size > TABLE_SIZE {
        panic!("hashtableSize length is larger than the circuit TableSize, please adjust the circuit TableSize");
    }
    let mut assignments = vec![];
    for i in 0..hashtable_num {
        for j in 0..query_bits[i].len() {
            // Cast query_indices[i][j] to usize, as it is used as an index.
            if hashtable_bits[i][query_indices[i][j] as usize] != query_bits[i][j] {
                println!("wrong query");
            }
        }
        let assignment = PermutationHashCircuit::from_entry(
            &hashtable_bits,
            &query_indices,
            &query_bits,
            hashtable_size,
            i,
        );
        assignments.push(assignment);
    }

    // -- permutationIndicesValidatorHashesCircuit --
    let mut active_validator_bits = vec![0u64; VALIDATOR_COUNT];

    // Mark validators present in valid_validator_list by setting their index to 1.
    for &idx in valid_validator_list.iter() {
        active_validator_bits[idx as usize] = 1;
    }

    // Prepare realKeys
    let mut real_keys = vec![0u64; VALIDATOR_COUNT];
    let mut cur_key: i64 = -1;

    // For each validator index, assign a real key based on whether the validator is active.
    for i in 0..VALIDATOR_COUNT {
        if active_validator_bits[i] == 1 {
            cur_key += 1;
            real_keys[i] = cur_key as u64;
        } else {
            real_keys[i] = (cur_key as u64) + (VALIDATOR_COUNT as u64);
        }
    }

    // Debug, can be removed
    let debug = true;
    if debug {
        let mut key_hash_map: HashMap<u64, Vec<u32>> = HashMap::new();
        for i in 0..validator_hashes.len() {
            key_hash_map.insert(real_keys[i], validator_hashes[i].clone());
        }
        for &shuffle_index in shuffle_indices.iter() {
            // Look up the query hashes using the current shuffle index as the key.
            if let Some(query_hashes) = key_hash_map.get(&shuffle_index) {
                // In the Go code, the expected validator hash is found at:
                //   validatorHashes[validValidatorList[shuffleIndices[i]]][j]
                // In Rust, we first convert the shuffle_index into an index for valid_validator_list.
                let valid_idx = valid_validator_list[shuffle_index as usize] as usize;
                for j in 0..query_hashes.len() {
                    if query_hashes[j] != validator_hashes[valid_idx][j] {
                        panic!(
                            "query_hashes[{}] != validator_hashes[{}][{}]",
                            j, shuffle_index, j
                        );
                    }
                }
            }
        }
    }
    let assignment = PermutationIndicesValidatorHashesCircuit::from_entry(
        &real_keys,
        &active_validator_bits_hash,
        &active_validator_bits,
        &validator_hashes,
        &shuffle_indices,
        &committee_indices,
        &valid_validator_list,
    );
    (assignments, assignment)
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermutationHashEntry {
    #[serde(rename = "QueryIndices")]
    pub query_indices: Vec<u32>,
    #[serde(rename = "QueryValidatorHashes")]
    pub query_validator_hashes: Vec<Vec<u32>>,
    #[serde(rename = "ActiveValidatorBitsHash")]
    pub active_validator_bits_hash: Vec<u32>,
    #[serde(rename = "ActiveValidatorBits")]
    pub active_validator_bits: Vec<u32>,
    #[serde(rename = "TableValidatorHashes")]
    pub table_validator_hashes: Vec<Vec<u32>>,
    #[serde(rename = "RealKeys")]
    pub real_keys: Vec<u32>,
}

impl GenericDefine<M31Config> for PermutationIndicesValidatorHashesCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let zero_var = builder.constant(0);
        let neg_one_count = builder.sub(1, VALIDATOR_COUNT as u32);
        //check the activeValidatorBitsHash
        if self.active_validator_bits.len() % 16 != 0 {
            panic!("activeValidatorBits length must be multiple of 16")
        }
        let mut active_validator_16_bits = vec![];
        for i in 0..VALIDATOR_COUNT / 16 {
            active_validator_16_bits.push(from_binary(
                builder,
                self.active_validator_bits[i * 16..(i + 1) * 16].to_vec(),
            ));
        }
        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        let active_validator_hash = params.hash_to_state_flatten(builder, &active_validator_16_bits);
        for (i, active_validator_hashbit) in active_validator_hash
            .iter()
            .enumerate()
            .take(POSEIDON_M31X16_RATE)
        {
            builder.assert_is_equal(active_validator_hashbit, self.active_validator_bits_hash[i]);
        }
        //move inactive validators to the end
        let mut sorted_table_key = [Variable::default(); VALIDATOR_COUNT];
        sorted_table_key[..VALIDATOR_COUNT].copy_from_slice(&self.real_keys[..VALIDATOR_COUNT]); //if active, use curKey, else use curInactiveKey
        //for the first one, if active, use 0, else use -ValidatorCount
        let shift = simple_select(
            builder,
            self.active_validator_bits[0],
            zero_var,
            neg_one_count,
        );
        let shift_key = builder.add(sorted_table_key[0], shift);
        let shift_key_zero = builder.is_zero(shift_key);
        builder.assert_is_equal(shift_key_zero, 1); //the first key must be 0 or ValidatorCount-1
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
            let previous_plus_one = builder.add(sorted_table_key[i - 1], 1);
            let previous_minus_count_plus_one =
                builder.sub(previous_plus_one, VALIDATOR_COUNT as u32);
            let previous_plus_count = builder.add(sorted_table_key[i - 1], VALIDATOR_COUNT as u32);
            let expected_key = simple_lookup2(
                builder,
                self.active_validator_bits[i - 1],
                self.active_validator_bits[i],
                sorted_table_key[i - 1],
                previous_plus_count,
                previous_minus_count_plus_one,
                previous_plus_one,
            );
            //if current one is active, the diff must be 1. Otherwise, the diff must be 0. That is, always equal to activeValidatorBits[i]
            let diff = builder.sub(expected_key, sorted_table_key[i]);
            let diff_zero = builder.is_zero(diff);
            builder.assert_is_equal(diff_zero, 1);
        }
        //logup
        let mut logup = LogUpSingleKeyTable::new(8);
        let mut table_values = vec![];
        for i in 0..VALIDATOR_COUNT {
            table_values.push(self.table_validator_hashes[i].to_vec());
        }
        //build a table with sorted key, i.e., the inactive validators have been moved to the end
        logup.new_table(sorted_table_key.to_vec(), table_values);
        //logup
        let mut query_values = vec![];
        for i in 0..QUERY_SIZE {
            query_values.push(self.query_validator_hashes[i].to_vec());
        }
        logup.batch_query(self.query_indices.to_vec(), query_values);
        logup.final_check(builder);
        // logup.final_check(builder);
        // logup.final_check(builder);
    }
}
//seperate PermutationIndicesValidatorHashesCircuit to 8 sub-circuits, leveraging avx512
declare_circuit!(PermutationIndicesValidatorHashBitCircuit {
    query_indices: [Variable; QUERY_SIZE],
    query_validator_hashes: [Variable; QUERY_SIZE],
    active_validator_bits_hash: [Variable; POSEIDON_M31X16_RATE],
    active_validator_bits: [Variable; VALIDATOR_COUNT],
    table_validator_hashes: [Variable; VALIDATOR_COUNT],
    real_keys: [Variable; VALIDATOR_COUNT],
});
impl PermutationIndicesValidatorHashBitCircuit<M31> {
    pub fn from_assignment(entry: &PermutationIndicesValidatorHashesCircuit<M31>) -> Vec<Self> {
        let mut assignment = PermutationIndicesValidatorHashBitCircuit {
            query_indices: [M31::from(0); QUERY_SIZE],
            query_validator_hashes: [M31::from(0); QUERY_SIZE],
            active_validator_bits_hash: [M31::from(0); POSEIDON_M31X16_RATE],
            active_validator_bits: [M31::from(0); VALIDATOR_COUNT],
            table_validator_hashes: [M31::from(0); VALIDATOR_COUNT],
            real_keys: [M31::from(0); VALIDATOR_COUNT],
        };
        assignment.query_indices.copy_from_slice(&entry.query_indices);
        assignment.active_validator_bits_hash.copy_from_slice(&entry.active_validator_bits_hash);
        assignment.active_validator_bits.copy_from_slice(&entry.active_validator_bits);
        assignment.real_keys.copy_from_slice(&entry.real_keys);
        let mut assignments = vec![];
        for i in 0..POSEIDON_M31X16_RATE {
            for j in 0..QUERY_SIZE {
                assignment.query_validator_hashes[j] = entry.query_validator_hashes[j][i];
            }
            for j in 0..VALIDATOR_COUNT {
                assignment.table_validator_hashes[j] = entry.table_validator_hashes[j][i];
            }
            assignments.push(assignment.clone());
        }
        assignments
    }
}

impl GenericDefine<M31Config> for PermutationIndicesValidatorHashBitCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let zero_var = builder.constant(0);
        let neg_one_count = builder.sub(1, VALIDATOR_COUNT as u32);
        //check the activeValidatorBitsHash
        if self.active_validator_bits.len() % 16 != 0 {
            panic!("activeValidatorBits length must be multiple of 16")
        }
        let mut active_validator_16_bits = vec![];
        for i in 0..VALIDATOR_COUNT / 16 {
            active_validator_16_bits.push(from_binary(
                builder,
                self.active_validator_bits[i * 16..(i + 1) * 16].to_vec(),
            ));
        }
        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        let active_validator_hash = params.hash_to_state_flatten(builder, &active_validator_16_bits);
        for (i, active_validator_hashbit) in active_validator_hash
            .iter()
            .enumerate()
            .take(POSEIDON_M31X16_RATE)
        {
            builder.assert_is_equal(active_validator_hashbit, self.active_validator_bits_hash[i]);
        }
        //move inactive validators to the end
        let mut sorted_table_key = [Variable::default(); VALIDATOR_COUNT];
        sorted_table_key[..VALIDATOR_COUNT].copy_from_slice(&self.real_keys[..VALIDATOR_COUNT]); //if active, use curKey, else use curInactiveKey
        //for the first one, if active, use 0, else use -ValidatorCount
        let shift = simple_select(
            builder,
            self.active_validator_bits[0],
            zero_var,
            neg_one_count,
        );
        let shift_key = builder.add(sorted_table_key[0], shift);
        let shift_key_zero = builder.is_zero(shift_key);
        builder.assert_is_equal(shift_key_zero, 1); //the first key must be 0 or ValidatorCount-1
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
            let previous_plus_one = builder.add(sorted_table_key[i - 1], 1);
            let previous_minus_count_plus_one =
                builder.sub(previous_plus_one, VALIDATOR_COUNT as u32);
            let previous_plus_count = builder.add(sorted_table_key[i - 1], VALIDATOR_COUNT as u32);
            let expected_key = simple_lookup2(
                builder,
                self.active_validator_bits[i - 1],
                self.active_validator_bits[i],
                sorted_table_key[i - 1],
                previous_plus_count,
                previous_minus_count_plus_one,
                previous_plus_one,
            );
            //if current one is active, the diff must be 1. Otherwise, the diff must be 0. That is, always equal to activeValidatorBits[i]
            let diff = builder.sub(expected_key, sorted_table_key[i]);
            let diff_zero = builder.is_zero(diff);
            builder.assert_is_equal(diff_zero, 1);
        }
        //logup
        let mut logup = LogUpSingleKeyTable::new(8);
        let mut table_values = vec![];
        for i in 0..VALIDATOR_COUNT {
            table_values.push(vec![self.table_validator_hashes[i]]);
        }
        //build a table with sorted key, i.e., the inactive validators have been moved to the end
        logup.new_table(sorted_table_key.to_vec(), table_values);
        //logup
        let mut query_values = vec![];
        for i in 0..QUERY_SIZE {
            query_values.push(vec![self.query_validator_hashes[i]]);
        }
        logup.batch_query(self.query_indices.to_vec(), query_values);
        logup.final_check(builder);
        // logup.final_check(builder);
        // logup.final_check(builder);
    }
}
pub fn generate_permutation_hashes_witness(dir: &str) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        println!("preparing solver...");
        ensure_directory_exists("./witnesses/permutationhashes");
        let file_name = format!("solver_permutationhashes_{}.txt", VALIDATOR_COUNT);
        let w_s = if std::fs::metadata(&file_name).is_ok() {
            println!("The solver exists!");
            let file = std::fs::File::open(&file_name).unwrap();
            let reader = std::io::BufReader::new(file);
            witness_solver::WitnessSolver::deserialize_from(reader).unwrap()
        } else {
            println!("The solver does not exist.");
            let compile_result =
                compile_generic(&PermutationIndicesValidatorHashesCircuit::default(), CompileOptions::default()).unwrap();

            let file = std::fs::File::create(&file_name).unwrap();
            let writer = std::io::BufWriter::new(file);
            compile_result
                .witness_solver
                .serialize_into(writer)
                .unwrap();
            let CompileResult {
                witness_solver,
                layered_circuit,
            } = compile_result;
            let circuit_name = format!("circuit_permutationhashes_{}.txt", VALIDATOR_COUNT);
            let file = std::fs::File::create(&circuit_name).unwrap();
            let writer = std::io::BufWriter::new(file);
            layered_circuit.serialize_into(writer).unwrap();
            witness_solver
        };

        let witness_solver = Arc::new(w_s);

        println!("Start generating permutationhash witnesses...");
        let start_time = std::time::Instant::now();
        let file_path = format!("{}/permutationhash_assignment.json", dir);

        let permutation_hash_data: Vec<PermutationHashEntry> =
            read_from_json_file(&file_path).unwrap();
        let permutation_hash_data = &permutation_hash_data[0];
        let end_time = std::time::Instant::now();
        println!(
            "loaded permutationhash data time: {:?}",
            end_time.duration_since(start_time)
        );

        let mut hint_registry = HintRegistry::<M31>::new();
        register_hint(&mut hint_registry);
        let assignment = PermutationIndicesValidatorHashesCircuit::from_assignment(permutation_hash_data);
        let mut assignments = vec![];
        for _i in 0..16 {
            assignments.push(assignment.clone());
        }
        let assignment_chunks: Vec<Vec<PermutationIndicesValidatorHashesCircuit<M31>>> =
            assignments.chunks(16).map(|x| x.to_vec()).collect();

        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                let witness_solver = Arc::clone(&witness_solver);
                thread::spawn(move || {
                    let mut hint_registry1 = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry1);
                    // let witness = witness_solver
                    //     .solve_witness_with_hints(&assignments[0], &mut hint_registry1)
                    //     .unwrap();
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry1)
                        .unwrap();
                    let file_name = format!("./witnesses/permutationhashes/witness_{}.txt", i);
                    let file = std::fs::File::create(file_name).unwrap();
                    let writer = std::io::BufWriter::new(file);
                    witness.serialize_into(writer).unwrap();
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
        let end_time = std::time::Instant::now();
        println!(
            "Generate permutationhash witness Time: {:?}",
            end_time.duration_since(start_time)
        );
    });
}


pub fn generate_permutation_hashbit_witness(dir: &str) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        println!("preparing solver...");
        ensure_directory_exists("./witnesses/permutationhashbit");
        let file_name = format!("solver_permutationhashbit_{}.txt", VALIDATOR_COUNT);
        let w_s = if std::fs::metadata(&file_name).is_ok() {
            println!("The solver exists!");
            let file = std::fs::File::open(&file_name).unwrap();
            let reader = std::io::BufReader::new(file);
            witness_solver::WitnessSolver::deserialize_from(reader).unwrap()
        } else {
            println!("The solver does not exist.");
            let compile_result =
                compile_generic(&PermutationIndicesValidatorHashBitCircuit::default(), CompileOptions::default()).unwrap();
            let file = std::fs::File::create(&file_name).unwrap();
            let writer = std::io::BufWriter::new(file);
            compile_result
                .witness_solver
                .serialize_into(writer)
                .unwrap();
            let CompileResult {
                witness_solver,
                layered_circuit,
            } = compile_result;
            let circuit_name = format!("circuit_permutationhashbit_{}.txt", VALIDATOR_COUNT);
            let file = std::fs::File::create(&circuit_name).unwrap();
            let writer = std::io::BufWriter::new(file);
            layered_circuit.serialize_into(writer).unwrap();
            witness_solver
        };

        let witness_solver = Arc::new(w_s);

        println!("Start generating permutationhashbit witnesses...");
        let start_time = std::time::Instant::now();
        let file_path = format!("{}/permutationhash_assignment.json", dir);

        let permutation_hash_data: Vec<PermutationHashEntry> =
            read_from_json_file(&file_path).unwrap();
        let permutation_hash_data = &permutation_hash_data[0];
        let end_time = std::time::Instant::now();
        println!(
            "loaded permutationhash data time: {:?}",
            end_time.duration_since(start_time)
        );

        let mut hint_registry = HintRegistry::<M31>::new();
        register_hint(&mut hint_registry);
        let assignment = PermutationIndicesValidatorHashesCircuit::from_assignment(permutation_hash_data);
        let target_assignments = PermutationIndicesValidatorHashBitCircuit::from_assignment(&assignment);
        let mut assignments = vec![];
        for _i in 0..2 {
            assignments.extend(target_assignments.clone());
        }
        let assignment_chunks: Vec<Vec<PermutationIndicesValidatorHashBitCircuit<M31>>> =
            assignments.chunks(16).map(|x| x.to_vec()).collect();

        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                let witness_solver = Arc::clone(&witness_solver);
                thread::spawn(move || {
                    let mut hint_registry1 = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry1);
                    // let witness = witness_solver
                    //     .solve_witness_with_hints(&assignments[0], &mut hint_registry1)
                    //     .unwrap();
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry1)
                        .unwrap();
                    let file_name = format!("./witnesses/permutationhashbit/witness_{}.txt", i);
                    let file = std::fs::File::create(file_name).unwrap();
                    let writer = std::io::BufWriter::new(file);
                    witness.serialize_into(writer).unwrap();
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
        let end_time = std::time::Instant::now();
        println!(
            "Generate permutationhash witness Time: {:?}",
            end_time.duration_since(start_time)
        );
    });
}

#[test]
fn test_permutation_hashes(){
    let dir = "./data";
    generate_permutation_hashes_witness(dir);
}
#[test]
fn test_permutation_hashbit(){
    let dir = "./data";
    generate_permutation_hashbit_witness(dir);
}

#[test]
fn eval_permutation_hashbit(){
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let dir = "./data";
        println!("Start generating permutationhashbit witnesses...");
        let start_time = std::time::Instant::now();
        let file_path = format!("{}/permutationhash_assignment.json", dir);

        let permutation_hash_data: Vec<PermutationHashEntry> =
            read_from_json_file(&file_path).unwrap();
        let permutation_hash_data = &permutation_hash_data[0];
        let end_time = std::time::Instant::now();
        println!(
            "loaded permutationhash data time: {:?}",
            end_time.duration_since(start_time)
        );

        let assignment = PermutationIndicesValidatorHashesCircuit::from_assignment(permutation_hash_data);
        println!("Start permutationhashbit witnesses...");
        let target_assignments = PermutationIndicesValidatorHashBitCircuit::from_assignment(&assignment);
        println!("Start evaluating permutationhashbit witnesses...");
        for assignment in target_assignments {
            let mut hint_registry = HintRegistry::<M31>::new();
            register_hint(&mut hint_registry);
            debug_eval(&PermutationIndicesValidatorHashBitCircuit::default(), &assignment, hint_registry);
        }
    });
}