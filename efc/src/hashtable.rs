use crate::beacon;
use crate::utils::{
    ensure_directory_exists, get_solver, read_from_json_file, write_witness_to_file,
};
use ark_std::primitive::u8;
use circuit_std_rs::sha256::m31::check_sha256_37bytes_256batch_compress;
use circuit_std_rs::sha256::m31_utils::{big_array_add_reduce, bytes_to_bits};
use circuit_std_rs::utils::register_hint;
use expander_compiler::circuit::layered::witness::Witness;
use expander_compiler::frontend::extra::*;
use expander_compiler::frontend::*;
use serde::Deserialize;
use std::sync::Arc;
use std::thread;

pub const SHA256LEN: usize = 32;
pub const HASHTABLESIZE: usize = 256;
#[derive(Clone, Copy, Debug)]
pub struct HashTableParams {
    pub table_size: usize,
    pub hash_len: usize,
}
#[derive(Debug, Deserialize)]
pub struct HashTableJson {
    #[serde(rename = "Seed")]
    pub seed: Vec<u8>,
    #[serde(rename = "ShuffleRound")]
    pub shuffle_round: u8,
    #[serde(rename = "StartIndex")]
    pub start_index: Vec<u8>,
    #[serde(rename = "HashOutputs")]
    pub hash_outputs: Vec<Vec<u8>>,
}
#[derive(Debug, Deserialize)]
pub struct HashTablesJson {
    pub tables: Vec<HashTableJson>,
}

declare_circuit!(HASHTABLECircuit {
    shuffle_round: Variable,
    start_index: [Variable; 4],
    seed: [PublicVariable; SHA256LEN],
    output: [[Variable; SHA256LEN]; HASHTABLESIZE],
});
pub type HashtableAssignmentChunks = Vec<Vec<HASHTABLECircuit<M31>>>;
impl HASHTABLECircuit<M31> {
    pub fn from_entry(&mut self, entry: &HashTableJson) {
        for i in 0..SHA256LEN {
            self.seed[i] = M31::from(entry.seed[i] as u32);
        }
        self.shuffle_round = M31::from(entry.shuffle_round as u32);
        for i in 0..4 {
            self.start_index[i] = M31::from(entry.start_index[i] as u32);
        }
        for i in 0..HASHTABLESIZE {
            for j in 0..SHA256LEN {
                self.output[i][j] = M31::from(entry.hash_outputs[i][j] as u32);
            }
        }
    }
    pub fn get_assignments_from_data(hashtable_data: Vec<HashTableJson>) -> Vec<Self> {
        let mut assignments = vec![];
        for cur_hashtable_data in &hashtable_data {
            let mut hash_assignment = HASHTABLECircuit::default();
            hash_assignment.from_entry(cur_hashtable_data);
            assignments.push(hash_assignment);
        }
        assignments
    }
    pub fn from_beacon(
        &mut self,
        seed: &[u8],
        shuffle_round: usize,
        start_index: usize,
        output: &[[u8; 32]],
    ) {
        self.seed
            .iter_mut()
            .zip(seed.iter())
            .for_each(|(a, &b)| *a = M31::from(b as u32));

        self.shuffle_round = M31::from(shuffle_round as u32);
        let start_index_bytes_le = (start_index as u32).to_le_bytes();
        self.start_index
            .iter_mut()
            .zip(start_index_bytes_le.iter())
            .for_each(|(a, &b)| *a = M31::from(b as u32));

        (0..HASHTABLESIZE).for_each(|i| {
            (0..SHA256LEN).for_each(|j| self.output[i][j] = M31::from(output[i][j] as u32))
        });
    }
    pub fn get_assignments_from_beacon_data(
        seed: &[u8],
        output: &[[u8; 32]],
        subcircuit_count: usize,
    ) -> Vec<Self> {
        let mut assignments = vec![];
        let size_per_round = output.len() / beacon::SHUFFLEROUND;
        for i in 0..subcircuit_count {
            let current_round = i * HASHTABLESIZE / size_per_round;
            let start_index = (i * HASHTABLESIZE) % size_per_round;
            let mut hash_assignment = HASHTABLECircuit::default();
            hash_assignment.from_beacon(
                seed,
                current_round,
                start_index,
                &output[i * HASHTABLESIZE..(i + 1) * HASHTABLESIZE],
            );
            assignments.push(hash_assignment);
        }
        assignments
    }
    pub fn get_assignments_from_json(dir: &str) -> Vec<Self> {
        let file_path = format!("{}/hash_assignment.json", dir);
        let hashtable_data: Vec<HashTableJson> = read_from_json_file(&file_path).unwrap();
        HASHTABLECircuit::get_assignments_from_data(hashtable_data)
    }
}
impl GenericDefine<M31Config> for HASHTABLECircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut seed_bits: Vec<Variable> = vec![];
        for i in 0..8 {
            seed_bits.extend_from_slice(&bytes_to_bits(builder, &self.seed[i * 4..(i + 1) * 4]));
        }
        let mut indices = vec![];
        let var0 = builder.constant(0);
        for i in 0..HASHTABLESIZE {
            //assume HASHTABLESIZE is less than 2^8
            let var_i = builder.constant(i as u32);
            let index =
                big_array_add_reduce(builder, &self.start_index, &[var_i, var0, var0, var0], 8);
            indices.push(bytes_to_bits(builder, &index));
        }
        let mut round_bits = vec![];
        round_bits.extend_from_slice(&bytes_to_bits(builder, &[self.shuffle_round]));
        let mut inputs = vec![];
        let mut outputs = vec![];
        for (i, index) in indices.iter().enumerate().take(HASHTABLESIZE) {
            let mut cur_input = Vec::<Variable>::new();
            cur_input.extend_from_slice(&seed_bits);
            cur_input.extend_from_slice(&index[8..]);
            cur_input.extend_from_slice(&round_bits);
            cur_input.extend_from_slice(&index[..8]);
            inputs.push(cur_input);
            outputs.push(self.output[i].to_vec());
        }
        check_sha256_37bytes_256batch_compress(builder, &inputs, &outputs);
    }
}

pub fn generate_hash_witnesses(dir: &str) {
    let circuit_name = &format!("hashtable{}", HASHTABLESIZE);

    //get solver
    log::debug!("preparing {} solver...", circuit_name);
    let witnesses_dir = format!("./witnesses/{}", circuit_name);
    let w_s = get_solver(&witnesses_dir, circuit_name, HASHTABLECircuit::default());

    //get assignments
    let start_time = std::time::Instant::now();
    let assignments = HASHTABLECircuit::get_assignments_from_json(dir);
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned {:} assignments time: {:?}",
        circuit_name,
        end_time.duration_since(start_time)
    );
    let assignment_chunks: HashtableAssignmentChunks =
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
}

pub fn end2end_hashtable_witnesses(
    w_s: WitnessSolver<M31Config>,
    hashtable_data: Vec<HashTableJson>,
) {
    let circuit_name = &format!("hashtable{}", HASHTABLESIZE);

    let witnesses_dir = format!("./witnesses/{}", circuit_name);
    ensure_directory_exists(&witnesses_dir);

    //get assignments
    let start_time = std::time::Instant::now();
    let assignments = HASHTABLECircuit::get_assignments_from_data(hashtable_data);
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned {:} assignments time: {:?}",
        circuit_name,
        end_time.duration_since(start_time)
    );
    let assignment_chunks: HashtableAssignmentChunks =
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
}

pub fn end2end_hashtable_witnesses_with_assignments(
    w_s: WitnessSolver<M31Config>,
    assignment_chunks: HashtableAssignmentChunks,
    witnesses_dir: String,
) {
    let start_time = std::time::Instant::now();
    //generate witnesses (multi-thread)
    log::debug!("Start generating witnesses on {}...", witnesses_dir);
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
        "Generate witness Time: {:?} on {}",
        end_time.duration_since(start_time),
        witnesses_dir
    );
}

pub fn end2end_hashtable_witnesses_with_assignments_chunk16(
    w_s: WitnessSolver<M31Config>,
    assignment_chunks: HashtableAssignmentChunks,
    witnesses_dir: String,
) {
    let start_time = std::time::Instant::now();
    //generate witnesses (multi-thread)
    log::debug!("Start generating witnesses on {}...", witnesses_dir);
    let witness_solver = Arc::new(w_s);
    let handles = assignment_chunks
        .into_iter()
        .enumerate()
        .map(|(i, assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            let witnesses_dir_clone = witnesses_dir.clone();
            thread::spawn(move || {
                let assignment_chunks: Vec<Vec<HASHTABLECircuit<M31>>> = assignments.chunks(16).map(|x| x.to_vec()).collect();
                let handles = assignment_chunks
                    .into_iter()
                    .enumerate()
                    .map(|(j, assignments)| {
                        let witness_solver = Arc::clone(&witness_solver);
                        thread::spawn(move || {
                            let mut hint_registry = HintRegistry::<M31>::new();
                            register_hint(&mut hint_registry);
                            (j, witness_solver
                                .solve_witnesses_with_hints(&assignments, &mut hint_registry)
                                .unwrap())
                        }
                        )
                    })
                    .collect::<Vec<_>>();
                let mut results = Vec::new();
                for handle in handles {
                    results.push(handle.join().unwrap());
                }
                let num_inputs_per_witness = results[0].1.num_inputs_per_witness;
                let num_public_inputs_per_witness = results[0].1.num_public_inputs_per_witness;
                results.sort_by_key(|(j, _)| *j);
                let new_values = results.into_iter().map(|(_, witness)| witness.values).flatten().collect::<Vec<M31>>();
                let new_witness: Witness<M31Config> = Witness::<M31Config> {
                    num_witnesses: assignments.len(),
                    num_inputs_per_witness,
                    num_public_inputs_per_witness,
                    values: new_values
                };
                write_witness_to_file(
                    &format!("{}/witness_{}.txt", witnesses_dir_clone, i),
                    new_witness,
                )
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
    let end_time = std::time::Instant::now();
    log::debug!(
        "Generate witness Time: {:?} on {}",
        end_time.duration_since(start_time),
        witnesses_dir
    );
}

pub fn end2end_hashtable_assignments_with_beacon_data(
    seed: &[u8],
    hash_bytes: Vec<[u8; 32]>,
    mpi_size: usize,
) -> HashtableAssignmentChunks {
    let subcircuit_count = hash_bytes.len() / HASHTABLESIZE;
    //get assignments
    let start_time = std::time::Instant::now();
    let assignments =
        HASHTABLECircuit::get_assignments_from_beacon_data(seed, &hash_bytes, subcircuit_count);
    let end_time = std::time::Instant::now();
    log::debug!(
        "assigned hahtable assignments time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: HashtableAssignmentChunks =
        assignments.chunks(16*mpi_size).map(|x| x.to_vec()).collect();
    assignment_chunks
}

pub fn debug_hashtable_all_assignments(assignment_chunks: HashtableAssignmentChunks) {
    stacker::grow(32 * 1024 * 1024 * 1024, || {
        let circuit_name = format!("hashtable{}", HASHTABLESIZE);

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
                            debug_eval(&HASHTABLECircuit::default(), &assignment, hint_registry);
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
// fn test_end2end_hashtable_assignments() {
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
//     let assignments = end2end_hashtable_assignments_with_beacon_data(&seed, hash_bytes);
// }

// #[test]
// fn test_hashtable_witnesses_end() {
//     stacker::grow(128 * 1024 * 1024 * 1024, || {
//         let epoch = 290000;
//         let slot = epoch * 32;
//         let hashtable_handle = thread::spawn(|| {
//             let circuit_name = format!("hashtable{}", HASHTABLESIZE);
//             let circuit = HASHTABLECircuit::default();
//             let witnesses_dir = format!("./witnesses/{}", circuit_name);
//             get_solver(&witnesses_dir, &circuit_name, circuit)
//         });
//         let solver_hashtable = hashtable_handle.join().unwrap();
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
//         let assignments = end2end_hashtable_assignments_with_beacon_data(&seed, hash_bytes);
//         end2end_hashtable_witnesses_with_assignments(solver_hashtable, assignments);
//     });
// }
