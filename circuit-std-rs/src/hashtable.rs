use std::sync::{Arc, Mutex};
use std::thread;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use expander_compiler::frontend::*;
use sha2::{Digest, Sha256};
use crate::big_int::{to_binary_hint, big_array_add};
use crate::sha2_m31::check_sha256;

const SHA256LEN: usize = 32;
const HASHTABLESIZE: usize = 64;
#[derive(Clone, Copy, Debug)]
pub struct HashTableParams {
    pub table_size: usize,
    pub hash_len: usize,
}

declare_circuit!(HASHTABLECircuit {
	shuffle_round: Variable,
	start_index:   [Variable;4],
	seed:      [PublicVariable; SHA256LEN],
	output:  [[Variable;SHA256LEN];HASHTABLESIZE],
});
impl Define<M31Config> for HASHTABLECircuit<Variable> {
	fn define(&self, builder: &mut API<M31Config>) {
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
	let compile_result = compile(&HASHTABLECircuit::default()).unwrap();
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

