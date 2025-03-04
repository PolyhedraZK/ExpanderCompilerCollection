use crate::utils::{ensure_directory_exists, read_from_json_file};
use circuit_std_rs::logup::LogUpSingleKeyTable;
use circuit_std_rs::poseidon_m31::{
    PoseidonM31Params, POSEIDON_M31X16_FULL_ROUNDS, POSEIDON_M31X16_PARTIAL_ROUNDS,
    POSEIDON_M31X16_RATE,
};
use circuit_std_rs::sha256::m31_utils::from_binary;
use circuit_std_rs::utils::{register_hint, simple_lookup2, simple_select};
use expander_compiler::circuit::ir::hint_normalized::witness_solver;
use expander_compiler::frontend::extra::{debug_eval, HintRegistry, Serde};
use expander_compiler::frontend::{
    compile, declare_circuit, CompileOptions, Define, M31Config, RootAPI, Variable, M31,
};
use serde::Deserialize;
use std::sync::Arc;
use std::thread;

pub const TABLE_SIZE: usize = 1024;
declare_circuit!(PermutationHashCircuit {
    index: [Variable; TABLE_SIZE],
    value: [Variable; TABLE_SIZE],
    table: [Variable; TABLE_SIZE],
});

impl Define<M31Config> for PermutationHashCircuit<Variable> {
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

impl Define<M31Config> for PermutationIndicesValidatorHashesCircuit<Variable> {
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
        let active_validator_hash = params.hash_to_state(builder, &active_validator_16_bits);
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
        logup.final_check(builder);
        logup.final_check(builder);
    }
}

pub fn generate_permutation_hashes_witness(dir: &str) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        println!("preparing solver...");
        ensure_directory_exists("./witnesses/permutationhashes");
        let file_name = format!("permutationhashes_{}.witness", VALIDATOR_COUNT);
        let w_s = if std::fs::metadata(&file_name).is_ok() {
            println!("The solver exists!");
            witness_solver::WitnessSolver::deserialize_from(
                std::fs::File::open(&file_name).unwrap(),
            )
            .unwrap()
        } else {
            println!("The solver does not exist.");
            let compile_result = compile(
                &PermutationIndicesValidatorHashesCircuit::default(),
                CompileOptions::default(),
            )
            .unwrap();
            compile_result
                .witness_solver
                .serialize_into(std::fs::File::create(&file_name).unwrap())
                .unwrap();
            compile_result.witness_solver
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
        let mut assignment = PermutationIndicesValidatorHashesCircuit::<M31> {
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
                    M31::from(permutation_hash_data.table_validator_hashes[i][j]);
            }
            assignment.real_keys[i] = M31::from(permutation_hash_data.real_keys[i]);
            assignment.active_validator_bits[i] =
                M31::from(permutation_hash_data.active_validator_bits[i]);
        }
        for i in 0..QUERY_SIZE {
            assignment.query_indices[i] = M31::from(permutation_hash_data.query_indices[i]);
            for j in 0..POSEIDON_M31X16_RATE {
                assignment.query_validator_hashes[i][j] =
                    M31::from(permutation_hash_data.query_validator_hashes[i][j]);
            }
        }
        for i in 0..POSEIDON_M31X16_RATE {
            assignment.active_validator_bits_hash[i] =
                M31::from(permutation_hash_data.active_validator_bits_hash[i]);
        }
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
                    let witness = witness_solver
                        .solve_witness_with_hints(&assignments[0], &mut hint_registry1)
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
