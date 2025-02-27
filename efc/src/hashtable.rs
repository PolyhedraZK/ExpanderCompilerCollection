use crate::utils::{ensure_directory_exists, read_from_json_file};
use ark_std::primitive::u8;
use circuit_std_rs::sha256::m31::check_sha256_37bytes_256batch_compress;
use circuit_std_rs::sha256::m31_utils::{big_array_add_reduce, bytes_to_bits};
use circuit_std_rs::utils::register_hint;
use expander_compiler::circuit::ir::hint_normalized::witness_solver;
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
    println!("preparing solver...");
    ensure_directory_exists("./witnesses/hashtable256");
    let file_name = "solver_hashtable256.txt";
    let w_s = if std::fs::metadata(file_name).is_ok() {
        println!("The solver exists!");
        let file = std::fs::File::open(file_name).unwrap();
        let reader = std::io::BufReader::new(file);
        witness_solver::WitnessSolver::deserialize_from(reader).unwrap()
    } else {
        println!("The solver does not exist.");
        let compile_result =
            compile_generic(&HASHTABLECircuit::default(), CompileOptions::default()).unwrap();
        let file = std::fs::File::create(file_name).unwrap();
        let writer = std::io::BufWriter::new(file);
        compile_result
            .witness_solver
            .serialize_into(writer)
            .unwrap();
        let CompileResult {
            witness_solver,
            layered_circuit,
        } = compile_result;
        let file = std::fs::File::create("circuit_hashtable256.txt").unwrap();
        let writer = std::io::BufWriter::new(file);
        layered_circuit.serialize_into(writer).unwrap();
        witness_solver
    };
    let witness_solver = Arc::new(w_s);

    println!("generating witnesses...");
    let start_time = std::time::Instant::now();

    let file_path = format!("{}/hash_assignment.json", dir);

    let hashtable_data: Vec<HashTableJson> = read_from_json_file(&file_path).unwrap();
    let mut assignments = vec![];
    for cur_hashtable_data in &hashtable_data {
        let mut hash_assignment = HASHTABLECircuit::default();
        for j in 0..32 {
            hash_assignment.seed[j] = M31::from(cur_hashtable_data.seed[j] as u32);
        }
        hash_assignment.shuffle_round = M31::from(cur_hashtable_data.shuffle_round as u32);
        for j in 0..4 {
            hash_assignment.start_index[j] = M31::from(cur_hashtable_data.start_index[j] as u32);
        }
        for j in 0..HASHTABLESIZE {
            for k in 0..32 {
                hash_assignment.output[j][k] =
                    M31::from(cur_hashtable_data.hash_outputs[j%64][k] as u32);
            }
        }
        assignments.push(hash_assignment);
    }

    let end_time = std::time::Instant::now();
    println!(
        "assigned assignments time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: Vec<Vec<HASHTABLECircuit<M31>>> =
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
                let file_name = format!("./witnesses/hashtable256/witness_{}.txt", i);
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
        "Generate hashtable witness Time: {:?}",
        end_time.duration_since(start_time)
    );
}

pub fn end2end_hashtable_witnesses(
    w_s: WitnessSolver<M31Config>,
    hashtable_data: Vec<HashTableJson>,
) {
    let witness_solver = Arc::new(w_s);

    println!("generating witnesses...");
    let start_time = std::time::Instant::now();
    let mut assignments = vec![];
    for cur_hashtable_data in &hashtable_data {
        let mut hash_assignment = HASHTABLECircuit::default();
        for j in 0..32 {
            hash_assignment.seed[j] = M31::from(cur_hashtable_data.seed[j] as u32);
        }
        hash_assignment.shuffle_round = M31::from(cur_hashtable_data.shuffle_round as u32);
        for j in 0..4 {
            hash_assignment.start_index[j] = M31::from(cur_hashtable_data.start_index[j] as u32);
        }
        for j in 0..HASHTABLESIZE {
            for k in 0..32 {
                hash_assignment.output[j][k] =
                    M31::from(cur_hashtable_data.hash_outputs[j][k] as u32);
            }
        }
        assignments.push(hash_assignment);
    }

    let end_time = std::time::Instant::now();
    println!(
        "assigned assignments time: {:?}",
        end_time.duration_since(start_time)
    );
    let assignment_chunks: Vec<Vec<HASHTABLECircuit<M31>>> =
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
                let file_name = format!("./witnesses/hashtable/witness_{}.txt", i);
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
        "Generate hashtable witness Time: {:?}",
        end_time.duration_since(start_time)
    );
}
#[test]
fn test_generate_hash_witnesses() {
    generate_hash_witnesses("./data");
}

#[test]
fn test_hashtable() {
    let dir = "./data";
    let file_path = format!("{}/hash_assignment.json", dir);

    let hashtable_data: Vec<HashTableJson> = read_from_json_file(&file_path).unwrap();
    let mut assignments = vec![];
    for cur_hashtable_data in &hashtable_data {
        let mut hash_assignment = HASHTABLECircuit::default();
        for j in 0..32 {
            hash_assignment.seed[j] = M31::from(cur_hashtable_data.seed[j] as u32);
        }
        hash_assignment.shuffle_round = M31::from(cur_hashtable_data.shuffle_round as u32);
        for j in 0..4 {
            hash_assignment.start_index[j] = M31::from(cur_hashtable_data.start_index[j] as u32);
        }
        for j in 0..HASHTABLESIZE {
            for k in 0..32 {
                hash_assignment.output[j][k] =
                    M31::from(cur_hashtable_data.hash_outputs[j % 64][k] as u32);
            }
        }
        assignments.push(hash_assignment);
    }
    let mut hint_registry = HintRegistry::<M31>::new();
    register_hint(&mut hint_registry);
    debug_eval(&HASHTABLECircuit::default(), &assignments[0], hint_registry);
    println!("pass!");
}
