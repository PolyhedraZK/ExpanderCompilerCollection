use circuit_std_rs::sha256::{m31::{check_sha256_37bytes_256batch_compress, sha256_37bytes}, m31_utils_zkcuda::*, m31_zkcuda::*};
use expander_compiler::frontend::*;
use extra::*;
use sha2::{Digest, Sha256};

declare_circuit!(SHA25637BYTESCircuit {
    input: [Variable; 37],
    output: [Variable; 32],
});

pub fn check_sha256<C: Config, B: RootAPI<C>>(
    builder: &mut B,
    origin_data: &Vec<Variable>,
) -> Vec<Variable> {
    let output = origin_data[37..].to_vec();
    let result = sha256_37bytes(builder, &origin_data[..37]);
    for i in 0..32 {
        builder.assert_is_equal(result[i], output[i]);
    }
    result
}

impl Define<M31Config> for SHA25637BYTESCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        for _ in 0..8 {
            let mut data = self.input.to_vec();
            data.append(&mut self.output.to_vec());
            builder.memorized_simple_call(check_sha256, &data);
        }
    }
}

// #[test]
// fn test_sha256_37bytes() {
//     let mut hint_registry = HintRegistry::<M31>::new();
//     hint_registry.register("myhint.tobinary", to_binary_hint);
//     let compile_result =
//         compile(&SHA25637BYTESCircuit::default(), CompileOptions::default()).unwrap();
//     for i in 0..1 {
//         let data = [i; 37];
//         let mut hash = Sha256::new();
//         hash.update(data);
//         let output = hash.finalize();
//         let mut assignment = SHA25637BYTESCircuit::default();
//         for i in 0..37 {
//             assignment.input[i] = M31::from(data[i] as u32);
//         }
//         for i in 0..32 {
//             assignment.output[i] = M31::from(output[i] as u32);
//         }
//         let witness = compile_result
//             .witness_solver
//             .solve_witness_with_hints(&assignment, &mut hint_registry)
//             .unwrap();
//         let output = compile_result.layered_circuit.run(&witness);
//         assert_eq!(output, vec![true]);
//     }
// }

// #[test]
// fn debug_sha256_37bytes() {
//     let mut hint_registry = HintRegistry::<M31>::new();
//     hint_registry.register("myhint.tobinary", to_binary_hint);
//     let data = [255; 37];
//     let mut hash = Sha256::new();
//     hash.update(data);
//     let output = hash.finalize();
//     let mut assignment = SHA25637BYTESCircuit::default();
//     for i in 0..37 {
//         assignment.input[i] = M31::from(data[i] as u32);
//     }
//     for i in 0..32 {
//         assignment.output[i] = M31::from(output[i] as u32);
//     }
//     debug_eval(&SHA25637BYTESCircuit::default(), &assignment, hint_registry);
// }


const HASHTABLESIZE: usize = 64;
declare_circuit!(HASHTABLECircuit {
    shuffle_round: Variable,
    start_index: [Variable; 4],
    seed: [PublicVariable; SHA256LEN],
    output: [[Variable; SHA256LEN]; HASHTABLESIZE],
});
impl<C: Config> Define<C> for HASHTABLECircuit<Variable> {
    fn define<Builder: RootAPI<C>>(&self, builder: &mut Builder) {
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
fn hashtable_big_field<C: Config, const N_WITNESSES: usize>(){
    let compile_result: CompileResult<C> = compile(&HASHTABLECircuit::default(), CompileOptions::default()).unwrap();
    let CompileResult {
        witness_solver,
        layered_circuit,
    } = compile_result;
    let circuit_name = format!("circuit_{}.txt", "hashtablem31");
    let file = std::fs::File::create(&circuit_name).unwrap();
    let writer = std::io::BufWriter::new(file);
    layered_circuit.serialize_into(writer).unwrap();

    let seed = [255; 32];
    let start_index = [0, 0, 0, 0];
    let mut output = vec![];
    let repeat_time = 64;
    for i in 0..repeat_time {
        let mut hash = Sha256::new();
        let mut new_data = vec![];
        new_data.extend_from_slice(&seed);
        new_data.push(0);
        new_data.push(i);
        new_data.extend_from_slice(&vec![0,0,0]);
        hash.update(new_data);
        let output_data = hash.finalize();
        output.push(output_data);
    }
    let mut assignment = HASHTABLECircuit::default();
    for i in 0..32 {
        assignment.seed[i] = C::CircuitField::from(seed[i] as u32);
    }
    for i in 0..4 {
        assignment.start_index[i] = C::CircuitField::from(start_index[i] as u32);
    }
    for i in 0..repeat_time {
        for j in 0..32 {
            assignment.output[i as usize][j] = C::CircuitField::from(output[i as usize][j] as u32);
        }
    }

    // let mut hint_registry = HintRegistry::<C::CircuitField>::new();
    // hint_registry.register("myhint.tobinary", to_binary_hint::<C>);
    // let witness = witness_solver.solve_witness_with_hints(&HASHTABLECircuit::default(), &mut hint_registry).unwrap();
    // let res = layered_circuit.run(&witness);
    // assert_eq!(res, vec![true]);
    // println!("test 1 passed");
    let mut assignments = vec![];
    for _ in 0..N_WITNESSES {
        assignments.push(assignment.clone());
    }

    let mut expander_circuit = layered_circuit
        .export_to_expander::<C::DefaultGKRFieldConfig>()
        .flatten();
    let config = expander_config::Config::<C::DefaultGKRConfig>::new(
        expander_config::GKRScheme::Vanilla,
        mpi_config::MPIConfig::new(),
    );
    let mut hint_registry = HintRegistry::<C::CircuitField>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint::<C>);
    let start = std::time::Instant::now();
    let witness = witness_solver.solve_witnesses_with_hints(&assignments, &mut hint_registry).unwrap();
    let file_name = format!("witness_{}.txt", "hashtablem31");
    let file = std::fs::File::create(file_name).unwrap();
    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();
    println!("time: {} ms", start.elapsed().as_millis());
    let start = std::time::Instant::now();
    let (simd_input, simd_public_input) = witness.to_simd::<C::DefaultSimdField>();
    println!("{} {}", simd_input.len(), simd_public_input.len());
    expander_circuit.layers[0].input_vals = simd_input;
    expander_circuit.public_input = simd_public_input.clone();

    expander_circuit.evaluate();
    println!("time: {} ms", start.elapsed().as_millis());
    let start = std::time::Instant::now();
    let (claimed_v, proof) = gkr::executor::prove(&mut expander_circuit, &config);
    println!("time: {} ms", start.elapsed().as_millis());

    let start = std::time::Instant::now();
    assert!(gkr::executor::verify(
        &mut expander_circuit,
        &config,
        &proof,
        &claimed_v
    ));
    println!("time: {} ms", start.elapsed().as_millis());

    /*let assignments_correct: Vec<Keccak256Circuit<C::CircuitField>> = (0..N_WITNESSES)
        .map(|i| assignments[i * 2].clone())
        .collect();
    let witness = witness_solver
        .solve_witnesses(&assignments_correct)
        .unwrap();

    let file = match field_name {
        "m31" => std::fs::File::create("circuit_m31.txt").unwrap(),
        "bn254" => std::fs::File::create("circuit_bn254.txt").unwrap(),
        _ => panic!("unknown field"),
    };
    let writer = std::io::BufWriter::new(file);
    layered_circuit.serialize_into(writer).unwrap();

    let file = match field_name {
        "m31" => std::fs::File::create("witness_m31.txt").unwrap(),
        "bn254" => std::fs::File::create("witness_bn254.txt").unwrap(),
        _ => panic!("unknown field"),
    };

    let writer = std::io::BufWriter::new(file);
    witness.serialize_into(writer).unwrap();

    let file = match field_name {
        "m31" => std::fs::File::create("witness_m31_solver.txt").unwrap(),
        "bn254" => std::fs::File::create("witness_bn254_solver.txt").unwrap(),
        _ => panic!("unknown field"),
    };
    let writer = std::io::BufWriter::new(file);
    witness_solver.serialize_into(writer).unwrap();*/

    println!("dumped to files");
}
#[test]
fn test_hashtable(){
    hashtable_big_field::<M31Config, 16>();
}