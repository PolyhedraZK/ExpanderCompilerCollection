use crate::beacon;
use crate::utils::{
    ensure_directory_exists, get_solver, read_from_json_file, write_witness_to_file,
};
use circuit_std_rs::logup::LogUpSingleKeyTable;
use circuit_std_rs::poseidon::poseidon_m31::*;
use circuit_std_rs::poseidon::poseidon_u32::PoseidonParams;
use circuit_std_rs::poseidon::utils::*;
use circuit_std_rs::sha256::m31_utils::*;
use circuit_std_rs::utils::{register_hint, simple_lookup2, simple_select};
use expander_compiler::frontend::extra::*;
use expander_compiler::frontend::*;
use serde::Deserialize;
use std::sync::Arc;
use std::thread;
pub const QUERY_TABLE_SIZE: usize = 1024 * 1024;
#[derive(Debug, Clone, Deserialize)]
pub struct PermutationQueryEntry {
    #[serde(rename = "Index")]
    pub index: Vec<u32>,
    #[serde(rename = "Value")]
    pub value: Vec<u32>,
    #[serde(rename = "Table")]
    pub table: Vec<u32>,
}
declare_circuit!(PermutationQueryCircuit {
    index: [Variable; QUERY_TABLE_SIZE],
    value: [Variable; QUERY_TABLE_SIZE],
    table: [Variable; QUERY_TABLE_SIZE],
});
pub type PermutationQueryAssignmentChunks = Vec<Vec<PermutationQueryCircuit<M31>>>;
impl PermutationQueryCircuit<M31> {
    pub fn from_entry(entry: &PermutationQueryEntry) -> Self {
        let mut assignment = PermutationQueryCircuit {
            index: [M31::from(0); QUERY_TABLE_SIZE],
            value: [M31::from(0); QUERY_TABLE_SIZE],
            table: [M31::from(0); QUERY_TABLE_SIZE],
        };

        for j in 0..QUERY_TABLE_SIZE {
            assignment.table[j] = M31::from(entry.table[j]);
        }
        for j in 0..QUERY_TABLE_SIZE {
            assignment.index[j] = M31::from(entry.index[j]);
            assignment.value[j] = M31::from(entry.value[j]);
        }
        assignment
    }
    pub fn from_entries(entries: &[PermutationQueryEntry]) -> Vec<Self> {
        let mut assignments = vec![];
        for entry in entries {
            assignments.push(PermutationQueryCircuit::from_entry(entry));
        }
        assignments
    }

    pub fn get_assignments_from_data(
        hashtable_bits: &[Vec<u8>],
        query_bits: Vec<Vec<u8>>,
        query_indices: Vec<Vec<u64>>,
    ) -> Vec<Self> {
        let mut assignments = vec![];
        for i in 0..hashtable_bits.len() {
            let mut assignment = PermutationQueryCircuit::default();
            for j in 0..QUERY_TABLE_SIZE {
                assignment.table[j] =
                    M31::from(*hashtable_bits.get(i).and_then(|v| v.get(j)).unwrap_or(&0) as u32);
                assignment.index[j] =
                    M31::from(*query_indices.get(j).and_then(|v| v.get(i)).unwrap_or(&0) as u32);
                assignment.value[j] =
                    M31::from(*query_bits.get(j).and_then(|v| v.get(i)).unwrap_or(&0) as u32);
            }
            assignments.push(assignment);
        }
        assignments
    }
}

impl GenericDefine<M31Config> for PermutationQueryCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut table = LogUpSingleKeyTable::new(8);
        let mut table_key = vec![];
        for i in 0..QUERY_TABLE_SIZE {
            table_key.push(builder.constant(i as u32));
        }
        let mut table_values = vec![];
        for i in 0..QUERY_TABLE_SIZE {
            table_values.push(vec![self.table[i]]);
        }
        table.new_table(table_key, table_values);
        let mut query_values = vec![];
        for i in 0..QUERY_TABLE_SIZE {
            query_values.push(vec![self.value[i]]);
        }
        table.batch_query(self.index.to_vec(), query_values);
        //m31 field, repeat 3 times
        table.final_check(builder);
        table.final_check(builder);
        table.final_check(builder);
    }
}
pub fn end2end_permutation_query_witness(
    w_s: WitnessSolver<M31Config>,
    permutation_query_data: Vec<PermutationQueryEntry>,
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let circuit_name = "permutationquery";

        let witnesses_dir = format!("./witnesses/{}", circuit_name);
        ensure_directory_exists(&witnesses_dir);

        //get assignments
        let start_time = std::time::Instant::now();
        let assignments = PermutationQueryCircuit::from_entries(&permutation_query_data);
        let end_time = std::time::Instant::now();
        log::debug!(
            "assigned permutation_query assignments time: {:?}",
            end_time.duration_since(start_time)
        );
        let assignment_chunks: PermutationQueryAssignmentChunks =
            assignments.chunks(16).map(|x| x.to_vec()).collect();

        //generate witnesses (multi-thread)
        log::debug!("Start generating  {} witnesses...", circuit_name);
        let witness_solver = Arc::new(w_s);
        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                let witness_solver = Arc::clone(&witness_solver);
                let witnesses_dir_clone = witnesses_dir.clone();
                thread::spawn(move || {
                    //TODO: hint_registry cannot be shared/cloned
                    let mut hint_registry = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry);
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                        .unwrap();
                    write_witness_to_file(
                        &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                        witness,
                    )
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
        let end_time = std::time::Instant::now();
        log::debug!(
            "Generate {} witness Time: {:?}",
            circuit_name,
            end_time.duration_since(start_time)
        );
    });
}
pub fn end2end_permutation_query_witnesses_with_assignments(
    w_s: WitnessSolver<M31Config>,
    assignment_chunks: PermutationQueryAssignmentChunks,
) {
    let circuit_name = "permutationquery";

    let witnesses_dir = format!("./witnesses/{}", circuit_name);
    ensure_directory_exists(&witnesses_dir);

    let start_time = std::time::Instant::now();
    //generate witnesses (multi-thread)
    log::debug!("Start generating  {} witnesses...", circuit_name);
    let witness_solver = Arc::new(w_s);
    let handles = assignment_chunks
        .into_iter()
        .enumerate()
        .map(|(i, assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            let witnesses_dir_clone = witnesses_dir.clone();
            thread::spawn(move || {
                //TODO: hint_registry cannot be shared/cloned
                let mut hint_registry = HintRegistry::<M31>::new();
                register_hint(&mut hint_registry);
                let witness = witness_solver
                    .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                    .unwrap();
                write_witness_to_file(
                    &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                    witness,
                )
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
    let end_time = std::time::Instant::now();
    log::debug!(
        "Generate {} witness Time: {:?}",
        circuit_name,
        end_time.duration_since(start_time)
    );
}

pub fn end2end_permutation_query_assignments(
    permutation_query_data: Vec<PermutationQueryEntry>,
) -> PermutationQueryAssignmentChunks {
    //get assignments
    let start_time = std::time::Instant::now();
    let assignments = PermutationQueryCircuit::from_entries(&permutation_query_data);
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned permutation_query assignments time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: PermutationQueryAssignmentChunks =
        assignments.chunks(16).map(|x| x.to_vec()).collect();
    assignment_chunks
}

#[test]
fn test_permutationquery() {
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = PermutationQueryCircuit::<M31> {
        index: [M31::from(0); QUERY_TABLE_SIZE],
        value: [M31::from(0); QUERY_TABLE_SIZE],
        table: [M31::from(0); QUERY_TABLE_SIZE],
    };
    for i in 0..QUERY_TABLE_SIZE {
        assignment.index[i] = M31::from(i as u32);
        assignment.value[i] = M31::from((i as u32 + 571) * 79);
        assignment.table[i] = M31::from((i as u32 + 571) * 79);
    }
    debug_eval(
        &PermutationQueryCircuit::default(),
        &assignment,
        hint_registry,
    );
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
    pub fn from_entry(entry: &PermutationHashEntry) -> Self {
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
                    M31::from(entry.table_validator_hashes[i % QUERY_SIZE][j]);
            }
            assignment.real_keys[i] = M31::from(entry.real_keys[i % QUERY_SIZE]);
            assignment.active_validator_bits[i] =
                M31::from(entry.active_validator_bits[i % QUERY_SIZE]);
        }
        for i in 0..QUERY_SIZE {
            assignment.query_indices[i] = M31::from(entry.query_indices[i % (QUERY_SIZE / 2)]);
            for j in 0..POSEIDON_M31X16_RATE {
                assignment.query_validator_hashes[i][j] =
                    M31::from(entry.query_validator_hashes[i % (QUERY_SIZE / 2)][j]);
            }
        }
        for i in 0..POSEIDON_M31X16_RATE {
            assignment.active_validator_bits_hash[i] =
                M31::from(entry.active_validator_bits_hash[i]);
        }
        assignment
    }
    pub fn get_assignments_from_data(
        permutation_hash_data: Vec<PermutationHashEntry>,
    ) -> Vec<Self> {
        let mut assignments = vec![];
        let assignment =
            PermutationIndicesValidatorHashesCircuit::from_entry(&permutation_hash_data[0]);
        for _ in 0..16 {
            assignments.push(assignment.clone());
        }
        assignments
    }
    pub fn get_assignments_from_json(dir: &str) -> Vec<Self> {
        let file_path = format!("{}/permutationhash_assignment.json", dir);
        let permutation_hash_data: Vec<PermutationHashEntry> =
            read_from_json_file(&file_path).unwrap();
        PermutationIndicesValidatorHashesCircuit::get_assignments_from_data(permutation_hash_data)
    }
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
                &self.active_validator_bits[i * 16..(i + 1) * 16],
            ));
        }
        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        let active_validator_hash =
            params.hash_to_state_flatten(builder, &active_validator_16_bits);
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
    query_indices: [Variable; QUERY_SIZE], //PCS: share with shuffle circuit
    query_validator_hashes: [Variable; QUERY_SIZE], //PCS: share with shuffle circuit
    active_validator_bits_hash: [Variable; POSEIDON_M31X16_RATE], //PUBLIC
    active_validator_bits: [Variable; VALIDATOR_COUNT], //HINT
    table_validator_hashes: [Variable; VALIDATOR_COUNT], //PCS: share with validatortree circuit
    real_keys: [Variable; VALIDATOR_COUNT], //HINT
});
pub type PermutationIndicesValidatorHashBitAssignmentChunks =
    Vec<Vec<PermutationIndicesValidatorHashBitCircuit<M31>>>;
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
        assignment
            .query_indices
            .copy_from_slice(&entry.query_indices);
        assignment
            .active_validator_bits_hash
            .copy_from_slice(&entry.active_validator_bits_hash);
        assignment
            .active_validator_bits
            .copy_from_slice(&entry.active_validator_bits);
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
    pub fn get_assignments_from_data(
        valid_validator_list: &[u64],
        validator_hashes: Vec<Vec<u32>>,
        shuffle_indices: Vec<u64>,
    ) -> Vec<Self> {
        // Real key mapping and active bit hashing
        let mut active_validator_bits = vec![0u32; VALIDATOR_COUNT];
        for &idx in valid_validator_list {
            active_validator_bits[idx as usize] = 1;
        }

        let mut real_keys = vec![0u64; VALIDATOR_COUNT];
        let mut cur_key: i64 = -1;
        for i in 0..VALIDATOR_COUNT {
            if active_validator_bits[i] == 1 {
                cur_key += 1;
                real_keys[i] = cur_key as u64;
            } else {
                real_keys[i] = (cur_key + QUERY_SIZE as i64) as u64;
            }
        }

        let params = PoseidonParams::new(
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        let mut compact_elements = vec![0; VALIDATOR_COUNT / 16];
        for i in 0..compact_elements.len() {
            let mut cur16 = 0u32;
            for j in 0..16 {
                cur16 |= active_validator_bits[i * 16 + j] << j;
            }
            compact_elements[i] = cur16;
        }
        let active_hash = params.hash_to_state(&compact_elements);
        let mut assignments = vec![];
        for i in 0..POSEIDON_M31X16_RATE {
            let mut assignment = PermutationIndicesValidatorHashBitCircuit::default();
            for (j, &v) in active_hash.iter().enumerate().take(POSEIDON_M31X16_RATE) {
                assignment.active_validator_bits_hash[j] = M31::from(v);
            }
            for j in 0..VALIDATOR_COUNT {
                assignment.real_keys[j] = M31::from(real_keys[j] as u32);
                assignment.active_validator_bits[j] = M31::from(active_validator_bits[j]);
                assignment.table_validator_hashes[j] =
                    M31::from(*validator_hashes.get(j).and_then(|h| h.get(i)).unwrap_or(&0));
            }
            for j in 0..QUERY_SIZE {
                assignment.query_indices[j] =
                    M31::from(*shuffle_indices.get(j).unwrap_or(&0) as u32);
                assignment.query_validator_hashes[j] = M31::from(
                    *valid_validator_list
                        .get(*shuffle_indices.get(j).unwrap_or(&0) as usize)
                        .and_then(|vid| validator_hashes.get(*vid as usize))
                        .and_then(|v| v.get(i))
                        .unwrap_or(&0),
                );
            }
            assignments.push(assignment);
        }
        assignments
    }
}

impl GenericDefine<M31Config> for PermutationIndicesValidatorHashBitCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let zero_var = builder.constant(0);
        let neg_one_count = builder.sub(1, QUERY_SIZE as u32);
        //check the activeValidatorBitsHash
        if self.active_validator_bits.len() % 16 != 0 {
            panic!("activeValidatorBits length must be multiple of 16")
        }
        let mut active_validator_16_bits = vec![];
        for i in 0..VALIDATOR_COUNT / 16 {
            active_validator_16_bits.push(from_binary(
                builder,
                &self.active_validator_bits[i * 16..(i + 1) * 16],
            ));
        }
        let params = PoseidonM31Params::new(
            builder,
            POSEIDON_M31X16_RATE,
            16,
            POSEIDON_M31X16_FULL_ROUNDS,
            POSEIDON_M31X16_PARTIAL_ROUNDS,
        );
        let active_validator_hash =
            params.hash_to_state_flatten(builder, &active_validator_16_bits);
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
            let previous_minus_count_plus_one = builder.sub(previous_plus_one, QUERY_SIZE as u32);
            let previous_plus_count = builder.add(sorted_table_key[i - 1], QUERY_SIZE as u32);
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
pub fn generate_permutation_hashbit_witnesses(dir: &str) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let circuit_name = &format!("permutationhashbit_{}", VALIDATOR_COUNT);

        //get solver
        log::debug!("preparing {} solver...", circuit_name);
        let witnesses_dir = format!("./witnesses/{}", circuit_name);
        let w_s = get_solver(
            &witnesses_dir,
            circuit_name,
            PermutationIndicesValidatorHashBitCircuit::default(),
        );

        let start_time = std::time::Instant::now();
        let assignments = PermutationIndicesValidatorHashesCircuit::get_assignments_from_json(dir);
        let target_assignments =
            PermutationIndicesValidatorHashBitCircuit::from_assignment(&assignments[0]);
        let mut assignments = vec![];
        for _ in 0..2 {
            assignments.extend(target_assignments.clone());
        }
        let end_time = std::time::Instant::now();
        log::debug!(
            "assigned permutation hashbit assignments time: {:?}",
            end_time.duration_since(start_time)
        );
        let assignment_chunks: PermutationIndicesValidatorHashBitAssignmentChunks =
            assignments.chunks(16).map(|x| x.to_vec()).collect();

        //generate witnesses (multi-thread)
        log::debug!("Start generating witnesses...");
        let witness_solver = Arc::new(w_s);
        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                let witness_solver = Arc::clone(&witness_solver);
                let witnesses_dir_clone = witnesses_dir.clone();
                thread::spawn(move || {
                    let mut hint_registry = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry);
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                        .unwrap();
                    write_witness_to_file(
                        &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                        witness,
                    )
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
        let end_time = std::time::Instant::now();
        log::debug!(
            "Generate {} witness Time: {:?}",
            circuit_name,
            end_time.duration_since(start_time)
        );
    });
}
pub fn end2end_permutation_hashbit_witness(
    w_s: WitnessSolver<M31Config>,
    permutation_hash_data: Vec<PermutationHashEntry>,
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let circuit_name = &format!("permutationhashbit_{}", VALIDATOR_COUNT);

        let witnesses_dir = format!("./witnesses/{}", circuit_name);
        let start_time = std::time::Instant::now();
        let assignment =
            PermutationIndicesValidatorHashesCircuit::from_entry(&permutation_hash_data[0]);
        let target_assignments =
            PermutationIndicesValidatorHashBitCircuit::from_assignment(&assignment);
        let mut assignments = vec![];
        for _ in 0..2 {
            assignments.extend(target_assignments.clone());
        }
        let end_time = std::time::Instant::now();
        log::debug!(
            "assigned permutation hashbit assignments time: {:?}",
            end_time.duration_since(start_time)
        );
        let assignment_chunks: PermutationIndicesValidatorHashBitAssignmentChunks =
            assignments.chunks(16).map(|x| x.to_vec()).collect();

        let witness_solver = Arc::new(w_s);
        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                let witness_solver = Arc::clone(&witness_solver);
                let witnesses_dir_clone = witnesses_dir.clone();
                thread::spawn(move || {
                    let mut hint_registry = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry);
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                        .unwrap();
                    write_witness_to_file(
                        &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                        witness,
                    )
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
        let end_time = std::time::Instant::now();
        log::debug!(
            "Generate permutationhash witness Time: {:?}",
            end_time.duration_since(start_time)
        );
    });
}
pub fn end2end_permutation_hashbit_witnesses_with_assignments(
    w_s: WitnessSolver<M31Config>,
    assignment_chunks: PermutationIndicesValidatorHashBitAssignmentChunks,
) {
    let circuit_name = &format!("permutationhashbit_{}", VALIDATOR_COUNT);

    let start_time = std::time::Instant::now();
    let witnesses_dir = format!("./witnesses/{}", circuit_name);

    let witness_solver = Arc::new(w_s);
    let handles = assignment_chunks
        .into_iter()
        .enumerate()
        .map(|(i, assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            let witnesses_dir_clone = witnesses_dir.clone();
            thread::spawn(move || {
                let mut hint_registry = HintRegistry::<M31>::new();
                register_hint(&mut hint_registry);
                let witness = witness_solver
                    .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                    .unwrap();
                write_witness_to_file(
                    &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                    witness,
                )
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
    let end_time = std::time::Instant::now();
    log::debug!(
        "Generate permutationhash witness Time: {:?}",
        end_time.duration_since(start_time)
    );
}
pub fn end2end_permutation_assignments_with_beacon_data(
    hashtable_bits: &[Vec<u8>],
    shuffle_data: &beacon::ShuffleData,
    valid_validator_list: &[u64],
    committee_data: &beacon::CommitteeData,
    padding_size: usize,
    validator_hashes: &[Vec<u32>],
    mpi_size1: usize,
    mpi_size2: usize,
) -> (
    PermutationQueryAssignmentChunks,
    PermutationIndicesValidatorHashBitAssignmentChunks,
) {
    let raw_committee_indices = committee_data.committee_indices.to_vec();
    let real_committee_size = &committee_data.real_committee_size;
    let raw_query_bits = shuffle_data.flip_bits.to_vec();
    let raw_query_indices = shuffle_data.positions.to_vec();
    let raw_shuffle_indices = shuffle_data.shuffle_indices.to_vec();
    let to_pad = padding_size - real_committee_size.last().copied().unwrap_or(0) as usize;
    let bit_len = raw_query_bits[0].len();

    let mut pad_bits = vec![vec![0u8; bit_len]; to_pad];
    let pad_indices = vec![vec![0u64; raw_query_indices[0].len()]; to_pad];

    (0..to_pad).for_each(|i| (0..bit_len).for_each(|j| pad_bits[i][j] = hashtable_bits[j][0]));

    let mut copy_query_bits = raw_query_bits.to_vec();
    copy_query_bits.extend(pad_bits);

    let mut copy_query_indices = raw_query_indices.to_vec();
    copy_query_indices.extend(pad_indices);

    let mut query_bits: Vec<Vec<u8>> = Vec::new();
    let mut query_indices: Vec<Vec<u64>> = Vec::new();
    let mut start = 0;

    for &real_size in real_committee_size {
        let end = (start + padding_size).min(copy_query_bits.len());
        query_bits.extend_from_slice(&copy_query_bits[start..end]);
        query_indices.extend_from_slice(&copy_query_indices[start..end]);
        start += real_size as usize;
    }

    let mut pad_shuffle_indices = raw_shuffle_indices.to_vec();
    pad_shuffle_indices.extend(vec![raw_shuffle_indices[0]; to_pad]);
    let mut pad_committee_indices = raw_committee_indices.to_vec();
    pad_committee_indices.extend(vec![raw_committee_indices[0]; to_pad]);

    let mut shuffle_indices = vec![];
    let mut committee_indices = vec![];
    let mut start = 0;

    for &real_size in real_committee_size {
        let end = (start + padding_size).min(pad_shuffle_indices.len());
        shuffle_indices.extend_from_slice(&pad_shuffle_indices[start..end]);
        committee_indices.extend_from_slice(&pad_committee_indices[start..end]);
        start += real_size as usize;
    }

    let permutation_query_assignments = PermutationQueryCircuit::get_assignments_from_data(
        hashtable_bits,
        query_bits,
        query_indices,
    );

    let permutation_query_assignment_chunks: PermutationQueryAssignmentChunks =
        permutation_query_assignments
            .chunks(16*mpi_size1)
            .map(|x| x.to_vec())
            .collect();

    let permutation_hashbit_assignments =
        PermutationIndicesValidatorHashBitCircuit::get_assignments_from_data(
            valid_validator_list,
            validator_hashes.to_vec(),
            shuffle_indices,
        );
    //copy permutation_hashbit_assignments to 16
    let mut permutation_hashbit_assignment_chunks: Vec<
        Vec<PermutationIndicesValidatorHashBitCircuit<M31>>,
    > = vec![permutation_hashbit_assignments.clone(); 1];
    permutation_hashbit_assignment_chunks[0].extend(permutation_hashbit_assignments.clone());
    if mpi_size2 == 1 {
        permutation_hashbit_assignment_chunks.push(permutation_hashbit_assignment_chunks[0].clone());
    } else if mpi_size2 == 2 {
        permutation_hashbit_assignment_chunks[0].extend(permutation_hashbit_assignments.clone());
        permutation_hashbit_assignment_chunks[0].extend(permutation_hashbit_assignments.clone());
    } else {
        panic!("mpi_size2 must be 1 or 2");
    }

    (
        permutation_query_assignment_chunks,
        permutation_hashbit_assignment_chunks,
    )
}

pub fn end2end_permutation_witnesses_with_beacon_data(
    w_s_query: WitnessSolver<M31Config>,
    w_s_hashbit: WitnessSolver<M31Config>,
    hashtable_bits: &[Vec<u8>],
    shuffle_data: &beacon::ShuffleData,
    valid_validator_list: &[u64],
    committee_data: &beacon::CommitteeData,
    padding_size: usize,
    validator_hashes: &[Vec<u32>],
    mpi_size1: usize,
    mpi_size2: usize,
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let (permutation_query_assignment_chunks, permutation_hashbit_assignment_chunks) =
            end2end_permutation_assignments_with_beacon_data(
                hashtable_bits,
                shuffle_data,
                valid_validator_list,
                committee_data,
                padding_size,
                validator_hashes,
                mpi_size1,
                mpi_size2,
            );
        end2end_permutation_hashbit_witnesses_with_assignments(
            w_s_hashbit,
            permutation_hashbit_assignment_chunks,
        );
        end2end_permutation_query_witnesses_with_assignments(
            w_s_query,
            permutation_query_assignment_chunks,
        );
    });
}
// pub fn debug_shuffle_with_assignments(
//     assignment_chunks: ShuffleAssignmentChunks,
// ) {
//     stacker::grow(32 * 1024 * 1024 * 1024, || {
//         let circuit_name = &format!("shuffle_{}", VALIDATOR_CHUNK_SIZE);

//         let start_time = std::time::Instant::now();
//         let mut hint_registry = HintRegistry::<M31>::new();
//         register_hint(&mut hint_registry);
//         debug_eval(&ShuffleCircuit::default(), &assignment_chunks[0][0], hint_registry);
//         // let witness = w_s
//         //             .solve_witnesses_with_hints(&assignment_chunks[0], &mut hint_registry)
//         //             .unwrap();
//         let end_time = std::time::Instant::now();
//         log::debug!(
//             "Generate {} witness Time: {:?}",
//             circuit_name,
//             end_time.duration_since(start_time)
//         );
//     });
// }

pub fn debug_permutation_query_all_assignments(
    assignment_chunks: PermutationQueryAssignmentChunks,
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let circuit_name = &format!("permutationquery_{}", QUERY_TABLE_SIZE);

        let start_time = std::time::Instant::now();
        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                thread::Builder::new()
                    .name(format!("large stack thread {}", i))
                    .stack_size(2 * 1024 * 1024 * 1024)
                    .spawn(move || {
                        for assignment in assignments {
                            let mut hint_registry = HintRegistry::<M31>::new();
                            register_hint(&mut hint_registry);
                            debug_eval(
                                &PermutationQueryCircuit::default(),
                                &assignment,
                                hint_registry,
                            );
                        }
                    })
                    .expect("Failed to spawn thread")
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
        let end_time = std::time::Instant::now();
        log::debug!(
            "Debug_eval {} assingments Time: {:?}",
            circuit_name,
            end_time.duration_since(start_time)
        );
    });
}
pub fn debug_permutation_hashbit_all_assignments(
    assignment_chunks: PermutationIndicesValidatorHashBitAssignmentChunks,
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let circuit_name = &format!("permutationhashbit_{}", VALIDATOR_COUNT);

        let start_time = std::time::Instant::now();
        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                thread::Builder::new()
                    .name(format!("large stack thread {}", i))
                    .stack_size(2 * 1024 * 1024 * 1024)
                    .spawn(move || {
                        for assignment in assignments {
                            let mut hint_registry = HintRegistry::<M31>::new();
                            register_hint(&mut hint_registry);
                            debug_eval(
                                &PermutationIndicesValidatorHashBitCircuit::default(),
                                &assignment,
                                hint_registry,
                            );
                        }
                    })
                    .expect("Failed to spawn thread")
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
        let end_time = std::time::Instant::now();
        log::debug!(
            "Debug_eval {} assingments Time: {:?}",
            circuit_name,
            end_time.duration_since(start_time)
        );
    });
}
// #[test]
// fn test_end2end_permutation_assignments() {
//     let slot = 290000 * 32;
//     let (
//         seed,
//         shuffle_indices,
//         committee_indices,
//         pivots,
//         activated_indices,
//         flips,
//         positions,
//         flip_bits,
//         round_hash_bits,
//         attestations,
//         aggregated_pubkeys,
//         balance_list,
//         real_committee_size,
//         validator_tree,
//         hash_bytes,
//         plain_validators,
//     ) = beacon::prepare_assignment_data(slot, slot + 32);
//     let (permutation_query_assignment_chunks, permutation_hashbit_assignment_chunks) =
//         end2end_permutation_assignments_with_beacon_data(
//             &round_hash_bits,
//             &flip_bits,
//             &positions,
//             &activated_indices,
//             &shuffle_indices,
//             &committee_indices,
//             &real_committee_size,
//             shuffle::VALIDATOR_CHUNK_SIZE,
//             &validator_tree[validator_tree.len() - 1],
//         );
// }

// #[test]
// fn test_permutation_hashbit_witnesses_end() {
//     stacker::grow(128 * 1024 * 1024 * 1024, || {
//         let epoch = 290000;
//         let slot = epoch * 32;
//         let circuit_name = format!("permutationhashbit_{}", VALIDATOR_COUNT);
//         let circuit = PermutationIndicesValidatorHashBitCircuit::default();
//         let witnesses_dir = format!("./witnesses/{}", circuit_name);
//         let permutation_query_handle = thread::spawn(|| {
//             let circuit_name = "permutationquery";
//             let circuit = PermutationQueryCircuit::default();
//             let witnesses_dir = format!("./witnesses/{}", circuit_name);
//             get_solver(&witnesses_dir, circuit_name, circuit)
//         });
//         let solver_permutation_hash = get_solver(&witnesses_dir, &circuit_name, circuit);
//         let solver_permutation_query = permutation_query_handle.join().unwrap();
//         let (
//             seed,
//             shuffle_indices,
//             committee_indices,
//             pivots,
//             activated_indices,
//             flips,
//             positions,
//             flip_bits,
//             round_hash_bits,
//             attestations,
//             aggregated_pubkeys,
//             balance_list,
//             real_committee_size,
//             validator_tree,
//             hash_bytes,
//             plain_validators,
//         ) = beacon::prepare_assignment_data(slot, slot + 32);
//         let (permutation_query_assignment_chunks, permutation_hashbit_assignment_chunks) =
//             end2end_permutation_assignments_with_beacon_data(
//                 &round_hash_bits,
//                 &flip_bits,
//                 &positions,
//                 &activated_indices,
//                 &shuffle_indices,
//                 &committee_indices,
//                 &real_committee_size,
//                 shuffle::VALIDATOR_CHUNK_SIZE,
//                 &validator_tree[validator_tree.len() - 1],
//             );
//         end2end_permutation_query_witnesses_with_assignments(
//             solver_permutation_query,
//             permutation_query_assignment_chunks,
//         );
//         // end2end_permutation_hashbit_witnesses_with_assignments(
//         //     solver_permutation_hash,
//         //     permutation_hashbit_assignment_chunks,
//         // );
//     });
// }
