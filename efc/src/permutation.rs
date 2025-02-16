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
    index: [Variable; TABLE_SIZE],
    value: [Variable; TABLE_SIZE],
    table: [Variable; TABLE_SIZE],
});
impl PermutationQueryCircuit<M31> {
    pub fn from_entry(entry: &PermutationQueryEntry) -> Self {
        let mut assignment = PermutationQueryCircuit {
            index: [M31::from(0); TABLE_SIZE],
            value: [M31::from(0); TABLE_SIZE],
            table: [M31::from(0); TABLE_SIZE],
        };

        for j in 0..TABLE_SIZE {
            assignment.table[j] = M31::from(entry.table[j]);
        }
        for j in 0..TABLE_SIZE {
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
}

impl GenericDefine<M31Config> for PermutationQueryCircuit<Variable> {
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
pub fn end2end_permutation_query_witness(
    w_s: WitnessSolver<M31Config>,
    permutation_query_data: Vec<PermutationQueryEntry>,
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let witness_solver = Arc::new(w_s);

        println!("Start generating permutation query witnesses...");
        let start_time = std::time::Instant::now();

        let mut hint_registry = HintRegistry::<M31>::new();
        register_hint(&mut hint_registry);
        let assignments = PermutationQueryCircuit::from_entries(&permutation_query_data);
        let assignment_chunks: Vec<Vec<PermutationQueryCircuit<M31>>> =
            assignments.chunks(16).map(|x| x.to_vec()).collect();

        let handles = assignment_chunks
            .into_iter()
            .enumerate()
            .map(|(i, assignments)| {
                let witness_solver = Arc::clone(&witness_solver);
                thread::spawn(move || {
                    let mut hint_registry = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry);
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                        .unwrap();
                    let file_name = format!("./witnesses/permutationquery/witness_{}.txt", i);
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
            "Generate permutation query witness Time: {:?}",
            end_time.duration_since(start_time)
        );
    });
}

#[test]
fn test_permutationquery() {
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = PermutationQueryCircuit::<M31> {
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
        &PermutationQueryCircuit::default(),
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
            assignment.active_validator_bits[i] = M31::from(entry.active_validator_bits[i]);
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
pub fn generate_permutation_hashes_witnesses(dir: &str) {
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
            let compile_result = compile_generic(
                &PermutationIndicesValidatorHashesCircuit::default(),
                CompileOptions::default(),
            )
            .unwrap();

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
        let assignment =
            PermutationIndicesValidatorHashesCircuit::from_assignment(permutation_hash_data);
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
                    let mut hint_registry = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry);
                    // let witness = witness_solver
                    //     .solve_witness_with_hints(&assignments[0], &mut hint_registry)
                    //     .unwrap();
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry)
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

pub fn generate_permutation_hashbit_witnesses(dir: &str) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        println!("preparing solver...");
        let initial_time = std::time::Instant::now();
        ensure_directory_exists("./witnesses/permutationhashbit");
        let file_name = format!("solver_permutationhashbit_{}.txt", VALIDATOR_COUNT);
        let w_s = if std::fs::metadata(&file_name).is_ok() {
            println!("The solver exists!");
            let file = std::fs::File::open(&file_name).unwrap();
            let reader = std::io::BufReader::new(file);
            witness_solver::WitnessSolver::deserialize_from(reader).unwrap()
        } else {
            println!("The solver does not exist.");
            let compile_result = compile_generic(
                &PermutationIndicesValidatorHashBitCircuit::default(),
                CompileOptions::default(),
            )
            .unwrap();
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
        let assignment =
            PermutationIndicesValidatorHashesCircuit::from_assignment(permutation_hash_data);
        let target_assignments =
            PermutationIndicesValidatorHashBitCircuit::from_assignment(&assignment);
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
                    let mut hint_registry = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry);
                    // let witness = witness_solver
                    //     .solve_witness_with_hints(&assignments[0], &mut hint_registry)
                    //     .unwrap();
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry)
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
        println!("total Time: {:?}", end_time.duration_since(initial_time));
    });
}
pub fn end2end_permutation_hashbit_witness(
    w_s: WitnessSolver<M31Config>,
    permutation_hash_data: Vec<PermutationHashEntry>,
) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let witness_solver = Arc::new(w_s);

        println!("Start generating permutationhashbit witnesses...");
        let start_time = std::time::Instant::now();
        let permutation_hash_data = &permutation_hash_data[0];

        let mut hint_registry = HintRegistry::<M31>::new();
        register_hint(&mut hint_registry);
        let assignment =
            PermutationIndicesValidatorHashesCircuit::from_assignment(permutation_hash_data);
        let target_assignments =
            PermutationIndicesValidatorHashBitCircuit::from_assignment(&assignment);
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
                    let mut hint_registry = HintRegistry::<M31>::new();
                    register_hint(&mut hint_registry);
                    let witness = witness_solver
                        .solve_witnesses_with_hints(&assignments, &mut hint_registry)
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
fn test_permutation_hashes() {
    let dir = "./data";
    generate_permutation_hashes_witnesses(dir);
}
#[test]
fn test_permutation_hashbit() {
    let dir = "./data";
    generate_permutation_hashbit_witnesses(dir);
}

#[test]
fn eval_permutation_hashbit() {
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

        let assignment =
            PermutationIndicesValidatorHashesCircuit::from_assignment(permutation_hash_data);
        println!("Start permutationhashbit witnesses...");
        let target_assignments =
            PermutationIndicesValidatorHashBitCircuit::from_assignment(&assignment);
        println!("Start evaluating permutationhashbit witnesses...");
        for assignment in target_assignments {
            let mut hint_registry = HintRegistry::<M31>::new();
            register_hint(&mut hint_registry);
            debug_eval(
                &PermutationIndicesValidatorHashBitCircuit::default(),
                &assignment,
                hint_registry,
            );
        }
    });
}
