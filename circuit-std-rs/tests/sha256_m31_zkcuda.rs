use circuit_std_rs::sha256::m31::check_sha256_37bytes_256batch_compress;
use expander_compiler::frontend::*;
use expander_compiler::utils::serde::Serde;
use expander_compiler::zkcuda::proving_system::{DummyProvingSystem, ExpanderGKRProvingSystem};
use expander_compiler::zkcuda::{context::*, kernel::*};
use sha2::{Digest, Sha256};
use circuit_std_rs::{sha256::m31_zkcuda::*, sha256::m31_utils_zkcuda::*};
use std::time;
use std::fs::File;
use std::io::{self, BufRead};


#[kernel]
fn sha256_37bytes<C: Config>(
    builder: &mut API<C>,
    orign_data: &[InputVariable; 37],
    output_data: &mut [OutputVariable; SHA256LEN],
) -> Vec<Variable> {
    println!("sha256_37bytes");
    let mut hint_idx = GLOBAL_HINT_IDX.lock().unwrap();
    *hint_idx = 0;
    std::mem::drop(hint_idx);
    let mut data = orign_data.to_vec();
    // for _ in 32..37 {
    //     data.push(builder.constant(255));
    // }
    let n = data.len();
    if n != 32 + 1 + 4 {
        panic!("len(orignData) !=  32+1+4")
    }
    let mut pre_pad = vec![builder.constant(0); 64 - 37];
    pre_pad[0] = builder.constant(128); //0x80
    pre_pad[64 - 37 - 2] = builder.constant((37) * 8 / 256); //length byte
    pre_pad[64 - 37 - 1] = builder.constant((32 + 1 + 4) * 8 - 256); //length byte
    data.append(&mut pre_pad); //append padding
    let mut d = MyDigest::new(builder);
    println!("pass");
    d.chunk_write(builder, &data);
    println!("write");
    let res = d.return_sum(builder).to_vec();
    for (i, val) in res.iter().enumerate() {
        output_data[i] = *val;
    }
}


#[test]
fn zkcuda_sha256_37bytes_globalhint() {
    println!("Global hints:");
    let filename = "log.txt";
    let file = File::open(filename).unwrap();
    let reader = io::BufReader::new(file);
    let mut matrix: Vec<Vec<u32>> = Vec::new();
    let mut reading_numbers = false;
    for line in reader.lines() {
        if let Ok(content) = line {
            let trimmed = content.trim();
            if trimmed == "tobinary:" {
                reading_numbers = true;
                continue;
            }
            if reading_numbers && !trimmed.is_empty() {
                let no_comma = trimmed.trim_end_matches(',');
                let nums: Vec<u32> = no_comma
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u32>().ok())
                    .collect();
                if !nums.is_empty() {
                    matrix.push(nums);
                }
                reading_numbers = false;
            }
        }
    }

    let mut hints = GLOBAL_HINTS.lock().unwrap();
    for i in 0..matrix.len() {
        let mut hint_res = vec![];
        for j in 0..matrix[i].len() {
            hint_res.push(matrix[i][j]);
        }
        hints.push(hint_res);
    }
    std::mem::drop(hints);
    println!("Global hints");
    let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
    println!("compile_sha256_37bytes() done");
    let data = [255; 37];
    let repeat_time = 64;
    let mut hash = Sha256::new();
    hash.update(data);
    let output = hash.finalize();
    let mut input_vars = vec![];
    let mut output_vars = vec![];
    for i in 0..37 {
        input_vars.push(M31::from(data[i] as u32));
    }
    for i in 0..32 {
        output_vars.push(M31::from(output[i] as u32));
    }
    let mut new_input_vars = vec![];
    for _ in 0..repeat_time {
        new_input_vars.push(input_vars.clone());
    }
    let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>> = Context::default();

    let a = ctx.copy_to_device(&new_input_vars, false);
    let mut c = None;
    let start_time = time::Instant::now();
    call_kernel!(ctx, kernel_check_sha256_37bytes, a, mut c);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
    // let c = c.reshape(&[repeat_time, 32]);
    let result: Vec<Vec<M31>> = ctx.copy_to_host(c);
    for i in 0..repeat_time {
        assert_eq!(result[i], output_vars);
    }
}



// #[test]
// fn zkcuda_sha256_37bytes_hint() {
//     let mut hint_registry = HintRegistry::<M31>::new();
//     hint_registry.register("myhint.tobinary", to_binary_hint);
//     let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
//     println!("compile_sha256_37bytes() done");
//     let data = [255; 37];
//     let repeat_time = 64;
//     let mut hash = Sha256::new();
//     hash.update(data);
//     let output = hash.finalize();
//     println!("output: {:?}", output);
//     let mut input_vars = vec![];
//     let mut output_vars = vec![];
//     for i in 0..37 {
//         input_vars.push(M31::from(data[i] as u32));
//     }
//     for i in 0..32 {
//         output_vars.push(M31::from(output[i] as u32));
//     }
//     let mut new_input_vars = vec![];
//     for _ in 0..repeat_time {
//         new_input_vars.push(input_vars.clone());
//     }
//     let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> = Context::new(hint_registry);

//     let a = ctx.copy_to_device(&new_input_vars, false);
//     let mut c = None;
//     let start_time = time::Instant::now();
//     call_kernel!(ctx, kernel_check_sha256_37bytes, a, mut c);
//     let elapsed = start_time.elapsed();
//     println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
//     // let c = c.reshape(&[repeat_time, 32]);
//     let result: Vec<Vec<M31>> = ctx.copy_to_host(c);
//     for i in 0..repeat_time {
//         assert_eq!(result[i], output_vars);
//     }
// }



// #[test]
// fn zkcuda_sha256_37bytes_simd_hint() {
//     use arith::SimdField;
//     let mut hint_registry = HintRegistry::<M31>::new();
//     hint_registry.register("myhint.tobinary", to_binary_hint);
//     let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
//     println!("compile_sha256_37bytes() done");
//     let data = [255; 37];
//     let repeat_time = 1;
//     let mut hash = Sha256::new();
//     hash.update(data);
//     let output = hash.finalize();
//     let mut input_vars: Vec<mersenne31::M31x16> = vec![];
//     let mut output_vars = vec![];
//     for i in 0..37 {
//         let mut tmp = Vec::new();
//         for j in 0..16 {
//             tmp.push(M31::from(data[i] as u32));
//         }
//         input_vars.push(mersenne31::M31x16::pack(&tmp));
//     }
//     for i in 0..32 {
//         output_vars.push(M31::from(output[i] as u32));
//     }
//     let mut new_input_vars: Vec<Vec<mersenne31::M31x16>> = vec![];
//     for _ in 0..repeat_time {
//         new_input_vars.push(input_vars.clone());
//     }
//     let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> = Context::new(hint_registry);

//     let a = ctx.copy_simd_to_device(&new_input_vars, false);
//     let mut c = None;
//     let start_time = time::Instant::now();
//     call_kernel!(ctx, kernel_check_sha256_37bytes, a, mut c);
//     let elapsed = start_time.elapsed();
//     println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
//     // let c = c.reshape(&[repeat_time, 32]);
//     let result: Vec<Vec<mersenne31::M31x16>> = ctx.copy_simd_to_host(c);
//     for i in 0..32 {
//         let unpack_output = result[0][i].unpack();
//         for j in 0..8 {
//             assert_eq!(unpack_output[j], output_vars[i]);
//         }
//     }
// }

// #[kernel]
// fn check_hashtable<C: Config>(
//     builder: &mut API<C>,
//     input: &[InputVariable; 37],
//     output: &mut [OutputVariable; 64*32],
// ) {
//     let mut seed_bits: Vec<Variable> = vec![];
//     for i in 0..8{
//         seed_bits.extend_from_slice(&bytes_to_bits(builder, &input[i*4..(i+1)*4]));
//     }
//     let mut indices = vec![];
//     let var0 = builder.constant(0);
//     for i  in 0..64 {
//         //assume HASHTABLESIZE is less than 2^8
//         let var_i = builder.constant(i as u32);
//         let index = big_array_add_reduce(builder, &input[33..37], &[var_i, var0, var0, var0], 8);
//         indices.push(bytes_to_bits(builder, &index));
//     }
//     let mut round_bits = vec![];
//     round_bits.extend_from_slice(&bytes_to_bits(builder, &[input[32]]));
//     let mut inputs = vec![];
//     for (i, index) in indices.iter().enumerate().take(64) {
//         let mut cur_input = Vec::<Variable>::new();
//         cur_input.extend_from_slice(&seed_bits);
//         cur_input.extend_from_slice(&index[8..]);
//         cur_input.extend_from_slice(&round_bits);
//         cur_input.extend_from_slice(&index[..8]);
//         inputs.push(cur_input);
//     }
//     println!("len of inputs: {}", inputs.len());
//     let res = sha256_37bytes_256batch_compress(builder, &inputs);
//     println!("len of res: {}", res.len());
//     for i in 0..64 {
//         for j in 0..32 {
//             output[i*32+j] = res[i][j];
//         }
//     }
// }




// #[test]
// fn zkcuda_hashtable_hint() {
//     let mut hint_registry = HintRegistry::<M31>::new();
//     hint_registry.register("myhint.tobinary", to_binary_hint);
//     let kernel_check_hashtable: Kernel<M31Config> = compile_check_hashtable().unwrap();
//     println!("compile_check_hashtable() done");
//     let data = [255; 32];
//     let mut expected_output = vec![];
//     let repeat_time = 64;
//     for i in 0..repeat_time {
//         let mut hash = Sha256::new();
//         let mut new_data = vec![];
//         new_data.extend_from_slice(&data);
//         new_data.push(0);
//         new_data.push(i);
//         new_data.extend_from_slice(&vec![0,0,0]);
//         hash.update(new_data);
//         let output = hash.finalize();
//         expected_output.push(output);
//     }
//     let mut input_vars = vec![];
//     let mut output_vars = vec![];
//     for i in 0..32 {
//         input_vars.push(M31::from(data[i] as u32));
//     }
//     for i in 32..37{
//         input_vars.push(M31::from(0 as u32));
//     }
//     for i in 0..repeat_time {
//         for j in 0..32 {
//             output_vars.push(M31::from(expected_output[i as usize][j] as u32));
//         }
//     }
//     let mut new_input_vars = vec![];
//     for _ in 0..repeat_time {
//         new_input_vars.push(input_vars.clone());
//     }
//     let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> = Context::new(hint_registry);

//     let a = ctx.copy_to_device(&new_input_vars, false);
//     let mut c = None;
//     let start_time = time::Instant::now();
//     call_kernel!(ctx, kernel_check_hashtable, a, mut c);
//     let elapsed = start_time.elapsed();
//     println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
//     // let c = c.reshape(&[repeat_time, 32]);
//     let result: Vec<M31> = ctx.copy_to_host(c);
//     assert_eq!(result, output_vars);
// }
const HASHTABLESIZE: usize = 64;
declare_circuit!(HASHTABLECircuit {
    shuffle_round: Variable,
    start_index: [Variable; 4],
    seed: [PublicVariable; SHA256LEN],
    output: [[Variable; SHA256LEN]; HASHTABLESIZE],
});
impl<C: Config> Define<C> for HASHTABLECircuit<Variable> {
    fn define(&self, builder: &mut API<C>) {
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
//Where C: Config = M31Config
fn hashtable_big_field<C: Config, const N_WITNESSES: usize>(){
    let compile_result: CompileResult<C> = compile(&HASHTABLECircuit::default()).unwrap();
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
    let (simd_input, simd_public_input) = witness.to_simd::<C::DefaultSimdField>();
    println!("{} {}", simd_input.len(), simd_public_input.len());
    expander_circuit.layers[0].input_vals = simd_input;
    expander_circuit.public_input = simd_public_input.clone();

    expander_circuit.evaluate();
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
// #[test]
// fn test_hashtable(){
//     let seed = [255; 32];
//     let start_index = [0, 0, 0, 0];
//     let mut output = vec![];
//     let repeat_time = 64;
//     for i in 0..repeat_time {
//         let mut hash = Sha256::new();
//         let mut new_data = vec![];
//         new_data.extend_from_slice(&seed);
//         new_data.push(0);
//         new_data.push(i);
//         new_data.extend_from_slice(&vec![0,0,0]);
//         hash.update(new_data);
//         let output_data = hash.finalize();
//         output.push(output_data);
//     }
//     let mut assignment = HASHTABLECircuit::default();
//     for i in 0..32 {
//         assignment.seed[i] = M31::from(seed[i] as u32);
//     }
//     for i in 0..4 {
//         assignment.start_index[i] = M31::from(start_index[i] as u32);
//     }
//     for i in 0..repeat_time {
//         for j in 0..32 {
//             assignment.output[i as usize][j] = M31::from(output[i as usize][j] as u32);
//         }
//     }
//     let compile_result = compile(&HASHTABLECircuit::default()).unwrap();
//     let mut hint_registry = HintRegistry::<M31>::new();
//     hint_registry.register("myhint.tobinary", to_binary_hint);
//     let witness = compile_result
//         .witness_solver
//         .solve_witness_with_hints(&assignment, &mut hint_registry)
//         .unwrap();
//     let start_time = std::time::Instant::now();
//     let output = compile_result.layered_circuit.run(&witness);
//     assert_eq!(output, vec![true]);
//     let elapsed = start_time.elapsed();
//     println!("Time elapsed in run() is: {:?}", elapsed);
// }

