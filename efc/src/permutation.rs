use std::sync::Arc;
use std::thread;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ark_bls12_381::g2;
use circuit_std_rs::gnark::hints::register_hint;
use circuit_std_rs::logup::LogUpSingleKeyTable;
use circuit_std_rs::poseidon_m31_var::poseidon_variable_unsafe;
use circuit_std_rs::poseidon_m31::*;
use circuit_std_rs::utils::{simple_lookup2, simple_select};
use expander_compiler::circuit::ir::hint_normalized::witness_solver;
use expander_compiler::frontend::*;
use expander_config::M31ExtConfigSha2;
use num_bigint::BigInt;
use sha2::{Digest, Sha256};
use circuit_std_rs::big_int::{to_binary_hint, big_array_add};
use circuit_std_rs::sha2_m31::check_sha256;
use circuit_std_rs::gnark::emulated::field_bls12381::*;
use circuit_std_rs::gnark::emulated::field_bls12381::e2::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::pairing::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::g1::*;
use circuit_std_rs::gnark::emulated::sw_bls12381::g2::*;
use circuit_std_rs::gnark::element::*;
use expander_compiler::frontend::extra::*;
use circuit_std_rs::big_int::*;
use expander_compiler::{circuit::layered::InputType, frontend::*};

use crate::utils::{ensure_directory_exists, run_circuit};


const TableSize: usize = 1024;
declare_circuit!(PermutationHashCircuit {
    index: [Variable;TableSize],
    value: [Variable;TableSize],
    table: [Variable;TableSize],
});

impl GenericDefine<M31Config> for PermutationHashCircuit<Variable>  {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut table = LogUpSingleKeyTable::new(8);
        let mut table_key = vec![];
        for i in 0..TableSize {
            table_key.push(builder.constant(i as u32));
        }
        let mut table_values = vec![];
        for i in 0..TableSize {
            table_values.push(vec![self.table[i]]);
        }
        table.new_table(table_key, table_values);
        let mut query_values = vec![];
        for i in 0..TableSize {
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
        index: [M31::from(0); TableSize],
        value: [M31::from(0); TableSize],
        table: [M31::from(0); TableSize],
    };
    for i in 0..TableSize {
        assignment.index[i] = M31::from(i as u32);
        assignment.value[i] = M31::from((i as u32 + 571) * 79);
        assignment.table[i] = M31::from((i as u32 + 571) * 79);
    }
    debug_eval(&PermutationHashCircuit::default(), &assignment, hint_registry);
}

const QuerySize: usize = 1024*1024;
const ValidatorCount: usize = QuerySize*2;
declare_circuit!(PermutationIndicesValidatorHashesCircuit {
    query_indices: [Variable;QuerySize],
    query_validator_hashes: [[Variable;POSEIDON_HASH_LENGTH];QuerySize],
    active_validator_bits_hash: [Variable;POSEIDON_HASH_LENGTH],
    active_validator_bits: [Variable;ValidatorCount],
    table_validator_hashes: [[Variable;POSEIDON_HASH_LENGTH];ValidatorCount],
    real_keys: [Variable;ValidatorCount],
});

impl GenericDefine<M31Config> for PermutationIndicesValidatorHashesCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let zero_var = builder.constant(0);
        let count_var = builder.constant(ValidatorCount as u32);
        let neg_one_count = builder.sub(1, ValidatorCount as u32);
        //check the activeValidatorBitsHash
        if self.active_validator_bits.len() % 16 != 0 {
            panic!("activeValidatorBits length must be multiple of 16")
        }
        let mut active_validator_16_bits = vec![];
        for i in 0..ValidatorCount/16 {
            active_validator_16_bits.push(from_binary(builder, self.active_validator_bits[i*16..(i+1)*16].to_vec()));
        }
        let active_validator_hash = poseidon_variable_unsafe(builder, &PoseidonParams::new(), active_validator_16_bits, false);
        for i in 0..POSEIDON_HASH_LENGTH {
            builder.assert_is_equal(active_validator_hash[i], self.active_validator_bits_hash[i]);
        }
        //move inactive validators to the end
        let mut sorted_table_key = [Variable::default();ValidatorCount];
        for i in 0..ValidatorCount {
            sorted_table_key[i] = self.real_keys[i]; //if active, use curKey, else use curInactiveKey
        }
        //for the first one, if active, use 0, else use -ValidatorCount
        let shift = simple_select(builder, self.active_validator_bits[0], zero_var.clone(), neg_one_count);
        let shift_key = builder.add(sorted_table_key[0], shift);
        let shift_key_zero = builder.is_zero(shift_key);
        builder.assert_is_equal(shift_key_zero, 1); //the first key must be 0 or ValidatorCount-1
        for i in 1..ValidatorCount {
            //for every validator, its key can be
            //active and active: previous key + 1
            //active and inactive: previous key - ValidatorCount + 1
            //inactive and active: previous key + ValidatorCount
            //inactive and inactive: previous key
            //1 1 --> previous key + 1
            //1 0 --> previous key - ValidatorCount + 1
            //0 1 --> previous key + ValidatorCount
            //0 0 --> previous key
            let previous_plus_one = builder.add(sorted_table_key[i-1], 1);
            let previous_minus_count_plus_one = builder.sub(previous_plus_one, ValidatorCount as u32);
            let previous_plus_count = builder.add(sorted_table_key[i-1], ValidatorCount as u32);
            let expected_key = simple_lookup2(builder, self.active_validator_bits[i-1], self.active_validator_bits[i], sorted_table_key[i-1], previous_plus_count, previous_minus_count_plus_one, previous_plus_one);
            //if current one is active, the diff must be 1. Otherwise, the diff must be 0. That is, always equal to activeValidatorBits[i]
            let diff = builder.sub(expected_key, sorted_table_key[i]);
            let diff_zero = builder.is_zero(diff);
            builder.assert_is_equal(diff_zero, 1);
        }
        //logup
        let mut logup = LogUpSingleKeyTable::new(8);
        let mut table_values = vec![];
        for i in 0..ValidatorCount {
            table_values.push(self.table_validator_hashes[i].to_vec());
        }
        //build a table with sorted key, i.e., the inactive validators have been moved to the end
        logup.new_table(sorted_table_key.to_vec(), table_values);
        //logup
        let mut query_values = vec![];
        for i in 0..QuerySize {
            query_values.push(self.query_validator_hashes[i].to_vec());
        }
        logup.batch_query(self.query_indices.to_vec(), query_values);
        logup.final_check(builder);
        logup.final_check(builder);
        logup.final_check(builder);
    }
}

#[test]
fn test_permutation_indices_validator_hashes() {
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = PermutationIndicesValidatorHashesCircuit::<M31> {
        query_indices: [M31::from(0); QuerySize],
        query_validator_hashes: [[M31::from(0); POSEIDON_HASH_LENGTH]; QuerySize],
        active_validator_bits_hash: [M31::from(0); POSEIDON_HASH_LENGTH],
        active_validator_bits: [M31::from(0); ValidatorCount],
        table_validator_hashes: [[M31::from(0); POSEIDON_HASH_LENGTH]; ValidatorCount],
        real_keys: [M31::from(0); ValidatorCount],
    };
    let mut all_indices = vec![0; ValidatorCount];
    for i in 0..ValidatorCount {
        all_indices[i] = i;
    }
    let mut table_validator_hashes = vec![];
    for i in 0..ValidatorCount {
        let mut hashes = vec![];
        for j in 0..POSEIDON_HASH_LENGTH {
            hashes.push(all_indices[i]);
        }
        table_validator_hashes.push(hashes);
    }
    let mut query_indices = vec![0; QuerySize];
    for i in 0..QuerySize {
        query_indices[i] = i;
    }
    let mut active_validator_bits = vec![0; ValidatorCount];
    for i in 0..QuerySize {
        active_validator_bits[(i*3)%ValidatorCount] = 1;
    }
    let mut query_validator_hashes = vec![];
    for i in 0..ValidatorCount {
        if active_validator_bits[i] == 1 {
            query_validator_hashes.push(table_validator_hashes[i].clone());
        }
    }
    let bits = active_validator_bits.clone();
    let mut real_keys = vec![0 as i32; ValidatorCount];
    let bit = active_validator_bits[0].clone();
    let mut cur_key = -1;
    if bit == 1 {
        cur_key += 1;
        real_keys[0] = cur_key;
    } else {
        real_keys[0] = ValidatorCount as i32 + cur_key;
    }
    for i in 1..ValidatorCount {
        let bit = active_validator_bits[i].clone();
        if bit == 1 {
            cur_key += 1;
            real_keys[i] = cur_key;
        } else {
            real_keys[i] = cur_key + ValidatorCount as i32;
        }
    }
    let mut active_validator_16bits = vec![];
    for i in 0..ValidatorCount/16 {
        let mut bit16 = 0;
        for j in (0..16).rev() {
            bit16 = bit16 * 2 + active_validator_bits[i*16+j];
        }
        active_validator_16bits.push(bit16);
    }
    let active_validator_bits_hash = poseidon_elements_unsafe(&PoseidonParams::new(), active_validator_16bits, false);
    for i in 0..ValidatorCount {
        for j in 0..POSEIDON_HASH_LENGTH {
            assignment.table_validator_hashes[i][j] = M31::from(table_validator_hashes[i][j] as u32);
        }
        assignment.real_keys[i] = M31::from(real_keys[i] as u32);
        let bit = bits[i].clone();
        assignment.active_validator_bits[i] = M31::from(bit as u32);
    }
    for i in 0..QuerySize {
        assignment.query_indices[i] = M31::from(query_indices[i] as u32);
        for j in 0..POSEIDON_HASH_LENGTH {
            assignment.query_validator_hashes[i][j] = M31::from(query_validator_hashes[i][j] as u32);
        }
    }
    for i in 0..POSEIDON_HASH_LENGTH {
        assignment.active_validator_bits_hash[i] = M31::from(active_validator_bits_hash[i] as u32);
    }
    debug_eval(&PermutationIndicesValidatorHashesCircuit::default(), &assignment, hint_registry);
}

#[test]
fn generate_permutation_hashes_witness() {
    stacker::grow(32 * 1024 * 1024 * 1024, ||    {
        println!("preparing solver...");
	ensure_directory_exists("./witnesses/permutationhashes");
    let mut w_s: witness_solver::WitnessSolver::<M31Config>;
    let file_name = format!("permutationhashes_{}.witness", ValidatorCount);
    if std::fs::metadata(&file_name).is_ok() {
        println!("The file exists!");
        w_s = witness_solver::WitnessSolver::deserialize_from(std::fs::File::open(&file_name).unwrap()).unwrap();
    } else {
        println!("The file does not exist.");
        let compile_result = compile_generic(&PermutationIndicesValidatorHashesCircuit::default(), CompileOptions::default()).unwrap();
        compile_result.witness_solver.serialize_into(std::fs::File::create(&file_name).unwrap()).unwrap();
        w_s = compile_result.witness_solver;
    }
    let witness_solver = Arc::new(w_s);

	println!("generating witnesses...");
    let start_time = std::time::Instant::now();

    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    let mut assignment = PermutationIndicesValidatorHashesCircuit::<M31> {
        query_indices: [M31::from(0); QuerySize],
        query_validator_hashes: [[M31::from(0); POSEIDON_HASH_LENGTH]; QuerySize],
        active_validator_bits_hash: [M31::from(0); POSEIDON_HASH_LENGTH],
        active_validator_bits: [M31::from(0); ValidatorCount],
        table_validator_hashes: [[M31::from(0); POSEIDON_HASH_LENGTH]; ValidatorCount],
        real_keys: [M31::from(0); ValidatorCount],
    };
    let mut all_indices = vec![0; ValidatorCount];
    for i in 0..ValidatorCount {
        all_indices[i] = i;
    }
    let mut table_validator_hashes = vec![];
    for i in 0..ValidatorCount {
        let mut hashes = vec![];
        for j in 0..POSEIDON_HASH_LENGTH {
            hashes.push(all_indices[i]);
        }
        table_validator_hashes.push(hashes);
    }
    let mut query_indices = vec![0; QuerySize];
    for i in 0..QuerySize {
        query_indices[i] = i;
    }
    let mut active_validator_bits = vec![0; ValidatorCount];
    for i in 0..QuerySize {
        active_validator_bits[(i*3)%ValidatorCount] = 1;
    }
    let mut query_validator_hashes = vec![];
    for i in 0..ValidatorCount {
        if active_validator_bits[i] == 1 {
            query_validator_hashes.push(table_validator_hashes[i].clone());
        }
    }
    let bits = active_validator_bits.clone();
    let mut real_keys = vec![0 as i32; ValidatorCount];
    let bit = active_validator_bits[0].clone();
    let mut cur_key = -1;
    if bit == 1 {
        cur_key += 1;
        real_keys[0] = cur_key;
    } else {
        real_keys[0] = ValidatorCount as i32 + cur_key;
    }
        for i in 1..ValidatorCount {
            let bit = active_validator_bits[i].clone();
            if bit == 1 {
                cur_key += 1;
                real_keys[i] = cur_key;
            } else {
                real_keys[i] = cur_key + ValidatorCount as i32;
            }
        }
        let mut active_validator_16bits = vec![];
        for i in 0..ValidatorCount/16 {
            let mut bit16 = 0;
            for j in (0..16).rev() {
                bit16 = bit16 * 2 + active_validator_bits[i*16+j];
            }
            active_validator_16bits.push(bit16);
        }
        let active_validator_bits_hash = poseidon_elements_unsafe(&PoseidonParams::new(), active_validator_16bits, false);
        for i in 0..ValidatorCount {
            for j in 0..POSEIDON_HASH_LENGTH {
                assignment.table_validator_hashes[i][j] = M31::from(table_validator_hashes[i][j] as u32);
            }
            assignment.real_keys[i] = M31::from(real_keys[i] as u32);
            let bit = bits[i].clone();
            assignment.active_validator_bits[i] = M31::from(bit as u32);
        }
        for i in 0..QuerySize {
            assignment.query_indices[i] = M31::from(query_indices[i] as u32);
            for j in 0..POSEIDON_HASH_LENGTH {
                assignment.query_validator_hashes[i][j] = M31::from(query_validator_hashes[i][j] as u32);
            }
        }
        for i in 0..POSEIDON_HASH_LENGTH {
            assignment.active_validator_bits_hash[i] = M31::from(active_validator_bits_hash[i] as u32);
        }
        let mut assignments = vec![];
        for i in 0..16 {
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
                    let witness = witness_solver.solve_witnesses_with_hints(&assignments, &mut hint_registry1).unwrap();
                    let file_name = format!("./witnesses/permutationhashes/witness_{}.txt", i);
                    let file = std::fs::File::create(file_name).unwrap();
                    let writer = std::io::BufWriter::new(file);
                    witness.serialize_into(writer).unwrap();
                }
                )
            })
            .collect::<Vec<_>>();
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().unwrap());
        }
        let end_time = std::time::Instant::now();
        println!("Generate hashtable witness Time: {:?}", end_time.duration_since(start_time));
    });
}