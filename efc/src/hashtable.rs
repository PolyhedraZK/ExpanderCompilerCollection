use std::thread;
use std::sync::Arc;
use circuit_std_rs::gnark::hints::register_hint;
use expander_compiler::circuit::ir::hint_normalized::witness_solver;
use expander_compiler::frontend::*;
use expander_config::M31ExtConfigSha2;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use circuit_std_rs::big_int::{to_binary_hint, big_array_add};
use circuit_std_rs::sha2_m31::check_sha256;
use expander_compiler::frontend::extra::*;
use ark_std::primitive::u8;
use crate::utils::{ensure_directory_exists, read_from_json_file};
use crate::utils::run_circuit;

const SHA256LEN: usize = 32;
const HASHTABLESIZE: usize = 64;
#[derive(Clone, Copy, Debug)]
pub struct HashTableParams {
    pub table_size: usize,
    pub hash_len: usize,
}
#[derive(Debug, Deserialize)]
pub struct HashTableJson{
	pub Seed: Vec<u8>,
	pub ShuffleRound: u8,
	pub StartIndex: Vec<u8>,
	pub HashOutputs: Vec<Vec<u8>>
}
#[derive(Debug, Deserialize)]
pub struct HashTablesJson{
	pub tables: Vec<HashTableJson>,
}

declare_circuit!(HASHTABLECircuit {
	shuffle_round: Variable,
	start_index:   [Variable;4],
	seed:      [PublicVariable; SHA256LEN],
	output:  [[Variable;SHA256LEN];HASHTABLESIZE],
});
impl GenericDefine<M31Config> for HASHTABLECircuit<Variable> {
	fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
		let mut indices = vec![Vec::<Variable>::new(); HASHTABLESIZE];
		if HASHTABLESIZE > 256 {
			panic!("HASHTABLESIZE > 256")
		}
		let var0 = builder.constant(0);
		for i in 0..HASHTABLESIZE {
			//assume HASHTABLESIZE is less than 2^8
			let var_i = builder.constant(i as u32);
			let index = big_array_add(builder, &self.start_index, &[var_i, var0, var0, var0], 8);
			indices[i] = index.to_vec();
		}
		for i in 0..HASHTABLESIZE {
			let mut cur_input = Vec::<Variable>::new();
			cur_input.extend_from_slice(&self.seed);
			cur_input.push(self.shuffle_round);
			cur_input.extend_from_slice(&indices[i]);
			let mut data = cur_input;
			data.append(&mut self.output[i].to_vec());
			builder.memorized_simple_call(check_sha256, &data);
		}
	}
}



#[test]
fn test_hashtable(){
	let seed = [0 as u8;32];
	let round = 0 as u8;
	let start_index =  [0 as u8;4];
	let mut assignment:HASHTABLECircuit<M31> = HASHTABLECircuit::default();
	for i in 0..32 {
		assignment.seed[i] = M31::from(seed[i] as u32);
	}
	
	assignment.shuffle_round = M31::from(round as u32);
	for i in 0..4 {
		assignment.start_index[i] = M31::from(start_index[i] as u32);
	}
	let mut inputs = vec![];
	let mut cur_index = start_index;
	for i in 0..HASHTABLESIZE{
		let mut input = vec![];
		input.extend_from_slice(&seed);
		input.push(round);
		input.extend_from_slice(&cur_index);
		if cur_index[0] == 255 {
			cur_index[0] = 0;
			cur_index[1] += 1;
		} else {
			cur_index[0] += 1;
		}
		inputs.push(input);
	}
	for i in 0..HASHTABLESIZE{
		let data = inputs[i].to_vec();
		let mut hash = Sha256::new();
		hash.update(&data);
		let output = hash.finalize();
		for j in 0..32 {
			assignment.output[i][j] = M31::from(output[j] as u32);
		}
	}
	let test_time = 1024;
	let mut handles = vec![];
    let mut assignments = vec![];
    for i in 0..test_time {
        assignments.push(assignment.clone());
    }
	let compile_result = compile_generic(&HASHTABLECircuit::default(), CompileOptions::default()).unwrap();
	let witness_solver = compile_result.witness_solver.clone();
    let start_time = std::time::Instant::now();
    for i in 0..test_time {
			let w_s = witness_solver.clone();
			let assignment_clone = assignments[i].clone();
			handles.push(thread::spawn(move || { 
				let mut hint_registry = HintRegistry::<M31>::new();
						hint_registry.register("myhint.tobinary", to_binary_hint);
				w_s
            .solve_witness_with_hints(&assignment_clone, &mut hint_registry)
            .unwrap();
			}));
    }
	for handle in handles {
		handle.join().unwrap();
	}
    let end_time = std::time::Instant::now();
    println!("Generate witness Time: {:?}", end_time.duration_since(start_time));
}

#[test]
fn run_expander_hashtable(){
	let seed = [0 as u8;32];
	let round = 0 as u8;
	let start_index =  [0 as u8;4];
	let mut assignment:HASHTABLECircuit<M31> = HASHTABLECircuit::default();
	for i in 0..32 {
		assignment.seed[i] = M31::from(seed[i] as u32);
	}
	
	assignment.shuffle_round = M31::from(round as u32);
	for i in 0..4 {
		assignment.start_index[i] = M31::from(start_index[i] as u32);
	}
	let mut inputs = vec![];
	let mut cur_index = start_index;
	for i in 0..HASHTABLESIZE{
		let mut input = vec![];
		input.extend_from_slice(&seed);
		input.push(round);
		input.extend_from_slice(&cur_index);
		if cur_index[0] == 255 {
			cur_index[0] = 0;
			cur_index[1] += 1;
		} else {
			cur_index[0] += 1;
		}
		inputs.push(input);
	}
	for i in 0..HASHTABLESIZE{
		let data = inputs[i].to_vec();
		let mut hash = Sha256::new();
		hash.update(&data);
		let output = hash.finalize();
		for j in 0..32 {
			assignment.output[i][j] = M31::from(output[j] as u32);
		}
	}
	let test_time = 16;
    let mut assignments = vec![];
    for i in 0..test_time {
        assignments.push(assignment.clone());
    }

	let compile_result = compile_generic(&HASHTABLECircuit::default(), CompileOptions::default()).unwrap();
	let mut hint_registry = HintRegistry::<M31>::new();
	hint_registry.register("myhint.tobinary", to_binary_hint);
    let start_time = std::time::Instant::now();
	let witness = compile_result
	.witness_solver
	.solve_witnesses_with_hints(&assignments, &mut hint_registry)
	.unwrap();
	let end_time = std::time::Instant::now();
	println!("Generate witness Time: {:?}", end_time.duration_since(start_time));
	run_circuit::<M31Config, M31ExtConfigSha2>(&compile_result, witness);
	let end_time = std::time::Instant::now();
	println!("Generate witness Time: {:?}", end_time.duration_since(start_time));
}

#[test]
fn run_multi_hashtable(){
	let seed = [0 as u8;32];
	let round = 0 as u8;
	let start_index =  [0 as u8;4];
	let mut assignment:HASHTABLECircuit<M31> = HASHTABLECircuit::default();
	for i in 0..32 {
		assignment.seed[i] = M31::from(seed[i] as u32);
	}
	
	assignment.shuffle_round = M31::from(round as u32);
	for i in 0..4 {
		assignment.start_index[i] = M31::from(start_index[i] as u32);
	}
	let mut inputs = vec![];
	let mut cur_index = start_index;
	for i in 0..HASHTABLESIZE{
		let mut input = vec![];
		input.extend_from_slice(&seed);
		input.push(round);
		input.extend_from_slice(&cur_index);
		if cur_index[0] == 255 {
			cur_index[0] = 0;
			cur_index[1] += 1;
		} else {
			cur_index[0] += 1;
		}
		inputs.push(input);
	}
	for i in 0..HASHTABLESIZE{
		let data = inputs[i].to_vec();
		let mut hash = Sha256::new();
		hash.update(&data);
		let output = hash.finalize();
		for j in 0..32 {
			assignment.output[i][j] = M31::from(output[j] as u32);
		}
	}
	let test_time = 2880;
    let mut assignments = vec![];
    for i in 0..test_time {
        assignments.push(assignment.clone());
    }

	let assignment_chunks: Vec<Vec<HASHTABLECircuit<M31>>> =
        assignments.chunks(16).map(|x| x.to_vec()).collect();
    let mut w_s: witness_solver::WitnessSolver::<M31Config>;
    if std::fs::metadata("hashtable.witness").is_ok() {
        println!("The file exists!");
        w_s = witness_solver::WitnessSolver::deserialize_from(std::fs::File::open("hashtable.witness").unwrap()).unwrap();
    } else {
        println!("The file does not exist.");
        let compile_result = compile_generic(&HASHTABLECircuit::default(), CompileOptions::default()).unwrap();
        compile_result.witness_solver.serialize_into(std::fs::File::create("hashtable.witness").unwrap()).unwrap();
        w_s = compile_result.witness_solver;
    }
    let witness_solver = Arc::new(w_s);
    let start_time = std::time::Instant::now();
    let handles = assignment_chunks
        .into_iter()
        .map(|(assignments)| {
            let witness_solver = Arc::clone(&witness_solver);
            thread::spawn(move || {
				println!("start");
                let mut hint_registry1 = HintRegistry::<M31>::new();
                register_hint(&mut hint_registry1);
                witness_solver.solve_witnesses_with_hints(&assignments, &mut hint_registry1).unwrap();
            }
            )
        })
        .collect::<Vec<_>>();
    // let handles = assignment_chunks
    //     .into_iter()
    //     .map(|assignments| {
    //         let witness_solver = Arc::clone(&witness_solver);
    //         let hint_register = Arc::clone(&share_hint_registry);
    //         thread::spawn(move || witness_solver.solve_witnesses_with_hints(&assignments, &mut ).unwrap())
    //     })
    //     .collect::<Vec<_>>();
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.join().unwrap());
    }
    let end_time = std::time::Instant::now();
    println!("Generate witness Time: {:?}", end_time.duration_since(start_time));
}

pub fn generate_hash_witnesses(dir: &str){
	println!("preparing solver...");
	ensure_directory_exists("./witnesses/hashtable");
    let mut w_s: witness_solver::WitnessSolver::<M31Config>;
    if std::fs::metadata("hashtable.witness").is_ok() {
        println!("The file exists!");
        w_s = witness_solver::WitnessSolver::deserialize_from(std::fs::File::open("hashtable.witness").unwrap()).unwrap();
    } else {
        println!("The file does not exist.");
        let compile_result = compile_generic(&HASHTABLECircuit::default(), CompileOptions::default()).unwrap();
        compile_result.witness_solver.serialize_into(std::fs::File::create("hashtable.witness").unwrap()).unwrap();
        w_s = compile_result.witness_solver;
    }
    let witness_solver = Arc::new(w_s);

	println!("generating witnesses...");
    let start_time = std::time::Instant::now();

	let file_path = format!("{}/hash_assignment.json",dir);

	let hashtable_data: Vec<HashTableJson> = read_from_json_file(&file_path).unwrap();
	let mut assignments = vec![];
	for i in 0..hashtable_data.len(){
		let mut hash_assignment = HASHTABLECircuit::default();
		for j in 0..32 {
			hash_assignment.seed[j] = M31::from(hashtable_data[i].Seed[j] as u32);
		}
		hash_assignment.shuffle_round = M31::from(hashtable_data[i].ShuffleRound as u32);
		for j in 0..4 {
			hash_assignment.start_index[j] = M31::from(hashtable_data[i].StartIndex[j] as u32);
		}
		for j in 0..HASHTABLESIZE{
			for k in 0..32 {
				hash_assignment.output[j][k] = M31::from(hashtable_data[i].HashOutputs[j][k] as u32);
			}
		}
		assignments.push(hash_assignment);
	}

    let end_time = std::time::Instant::now();
    println!("assigned assignments time: {:?}", end_time.duration_since(start_time));
	let assignment_chunks: Vec<Vec<HASHTABLECircuit<M31>>> =
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
				let file_name = format!("./witnesses/hashtable/witness_{}.txt", i);
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
}

#[test]
fn test_read_hash_assignment(){
	generate_hash_witnesses("");
}
