use circuit_std_rs::logup::LogUpSingleKeyTable;
use circuit_std_rs::utils::{register_hint, simple_lookup2, simple_select};
use expander_compiler::frontend::extra::debug_eval;
use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::{DummyProvingSystem, ExpanderGKRProvingSystem, ParallelizedExpanderGKRProvingSystem};
use expander_compiler::zkcuda::{context::*, kernel::*};
use circuit_std_rs::{sha256::m31::*, sha256::m31_utils::*};
use gkr_engine::{MPIConfig, MPIEngine};
use std::time;
use sha2::{Digest, Sha256};
use serdes::ExpSerde;


#[kernel]
fn sha256_37bytes<C: Config>(
    builder: &mut API<C>,
    orign_data: &[InputVariable; 37],
    output_data: &mut [OutputVariable; 32],
) -> Vec<Variable> {
    let mut data = orign_data.to_vec();
    for i in 0..256 {
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
        d.chunk_write(builder, &data);
        let res: Vec<Variable> = d.return_sum(builder).to_vec();
        data = vec![];
        for j in 0..32 {
            data.push(res[j]);
        }
        for j in 32..37 {
            data.push(orign_data[j]);
        }
    }
    for (i, val) in data.iter().enumerate() {
        if i >= 32 {
            break;
        }
        output_data[i] = *val;
    }
}



#[test]
fn zkcuda_hashtable_single_core() {
    let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
    println!("compile_sha256_37bytes() done");
    let data = [255; 37];
    let repeat_time = 16*4;
    let mut hash = Sha256::new();
    hash.update(data);
    let output = hash.finalize();
    // println!("output: {:?}", output);
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
    let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> = Context::default();

    let a = ctx.copy_to_device(&new_input_vars, false);
    let mut c = None;
    let start_time = time::Instant::now();
    call_kernel!(ctx, kernel_check_sha256_37bytes, a, mut c);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
    // let c = c.reshape(&[repeat_time, 32]);
    let result: Vec<Vec<M31>> = ctx.copy_to_host(c);
    // println!("result: {:?}", result);
    for i in 0..repeat_time {
        assert_eq!(result[i], output_vars);
    }

    let start_time = time::Instant::now();
    let computation_graph = ctx.to_computation_graph();
    let elapsed = start_time.elapsed();
    println!("Time elapsed in computation_graph!() is: {:?}", elapsed);
    let (prover_setup, verifier_setup) = ctx.proving_system_setup(&computation_graph);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in proving_system_setup!() is: {:?}", elapsed);
    let proof = ctx.to_proof(&prover_setup);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in to_proof!() is: {:?}", elapsed);
    assert!(computation_graph.verify(&proof, &verifier_setup));
    let elapsed = start_time.elapsed();
    println!("Time elapsed in assert!() is: {:?}", elapsed);
}


#[test]
fn zkcuda_hashtable_simd() {
    use arith::SimdField;
    let circuit_name = "kernel_sha256_256.txt";
    let kernel_check_sha256_37bytes = 
    if std::fs::metadata(&circuit_name).is_ok() {
        let file = std::fs::File::open(&circuit_name).unwrap();
        let reader = std::io::BufReader::new(file);
        Kernel::<M31Config>::deserialize_from(reader).unwrap()
    } else {
        let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
        let file = std::fs::File::create(&circuit_name).unwrap();
        let writer = std::io::BufWriter::new(file);
        kernel_check_sha256_37bytes.serialize_into(writer).unwrap();
        kernel_check_sha256_37bytes
    };
    println!("compile kernel_aggregated_pubkeys_cal_validator_hashes success");
    println!("compile_sha256_37bytes() done");
    let data = [255; 37];
    let repeat_time = 16;
    let mut hash = Sha256::new();
    hash.update(data);
    let output = hash.finalize();
    let mut input_vars: Vec<mersenne31::M31x16> = vec![];
    let mut output_vars = vec![];
    for i in 0..37 {
        let mut tmp = Vec::new();
        for j in 0..16 {
            tmp.push(M31::from(data[i] as u32));
        }
        input_vars.push(mersenne31::M31x16::pack(&tmp));
    }
    for i in 0..32 {
        output_vars.push(M31::from(output[i] as u32));
    }
    let mut new_input_vars: Vec<Vec<mersenne31::M31x16>> = vec![];
    for _ in 0..repeat_time {
        new_input_vars.push(input_vars.clone());
    }
    let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> = Context::default();

    let a = ctx.copy_simd_to_device(&new_input_vars, false);
    let mut c = None;
    let start_time = time::Instant::now();
    call_kernel!(ctx, kernel_check_sha256_37bytes, a, mut c);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
    // let c = c.reshape(&[repeat_time, 32]);
    // let result: Vec<Vec<mersenne31::M31x16>> = ctx.copy_simd_to_host(c);
    // for i in 0..32 {
    //     let unpack_output = result[0][i].unpack();
    //     for j in 0..8 {
    //         assert_eq!(unpack_output[j], output_vars[i]);
    //     }
    // }

    let start_time = time::Instant::now();
    let computation_graph = ctx.to_computation_graph();
    let elapsed = start_time.elapsed();
    println!("Time elapsed in computation_graph!() is: {:?}", elapsed);
    let (prover_setup, verifier_setup) = ctx.proving_system_setup(&computation_graph);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in proving_system_setup!() is: {:?}", elapsed);
    let proof = ctx.to_proof(&prover_setup);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in to_proof!() is: {:?}", elapsed);
    assert!(computation_graph.verify(&proof, &verifier_setup));
    let elapsed = start_time.elapsed();
    println!("Time elapsed in assert!() is: {:?}", elapsed);
}



#[test]
fn zkcuda_hashtable_multi_core_simd() {
    use arith::SimdField;
    let circuit_name = "kernel_sha256_256.txt";
    let kernel_check_sha256_37bytes = 
    if std::fs::metadata(&circuit_name).is_ok() {
        let file = std::fs::File::open(&circuit_name).unwrap();
        let reader = std::io::BufReader::new(file);
        Kernel::<M31Config>::deserialize_from(reader).unwrap()
    } else {
        let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
        let file = std::fs::File::create(&circuit_name).unwrap();
        let writer = std::io::BufWriter::new(file);
        kernel_check_sha256_37bytes.serialize_into(writer).unwrap();
        kernel_check_sha256_37bytes
    };
    println!("compile kernel_aggregated_pubkeys_cal_validator_hashes success");
    println!("compile_sha256_37bytes() done");
    let data = [255; 37];
    let repeat_time = 32;
    let mut hash = Sha256::new();
    hash.update(data);
    let output = hash.finalize();
    let mut input_vars: Vec<mersenne31::M31x16> = vec![];
    let mut output_vars = vec![];
    for i in 0..37 {
        let mut tmp = Vec::new();
        for j in 0..16 {
            tmp.push(M31::from(data[i] as u32));
        }
        input_vars.push(mersenne31::M31x16::pack(&tmp));
    }
    for i in 0..32 {
        output_vars.push(M31::from(output[i] as u32));
    }
    let mut new_input_vars: Vec<Vec<mersenne31::M31x16>> = vec![];
    for _ in 0..repeat_time {
        new_input_vars.push(input_vars.clone());
    }
    let mut ctx: Context<M31Config, ParallelizedExpanderGKRProvingSystem<M31Config>, _> = Context::default();

    let a = ctx.copy_simd_to_device(&new_input_vars, false);
    let mut c = None;
    let start_time = time::Instant::now();
    call_kernel!(ctx, kernel_check_sha256_37bytes, a, mut c);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
    // let c = c.reshape(&[repeat_time, 32]);
    let result: Vec<Vec<mersenne31::M31x16>> = ctx.copy_simd_to_host(c);
    // for i in 0..32 {
    //     let unpack_output = result[0][i].unpack();
    //     for j in 0..8 {
    //         assert_eq!(unpack_output[j], output_vars[i]);
    //     }
    // }

    let start_time = time::Instant::now();
    let computation_graph = ctx.to_computation_graph();
    let elapsed = start_time.elapsed();
    println!("Time elapsed in computation_graph!() is: {:?}", elapsed);
    let (prover_setup, verifier_setup) = ctx.proving_system_setup(&computation_graph);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in proving_system_setup!() is: {:?}", elapsed);
    let start_time = time::Instant::now();
    let proof = ctx.to_proof(&prover_setup);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in to_proof!() is: {:?}", elapsed);
    assert!(computation_graph.verify(&proof, &verifier_setup));
    let elapsed = start_time.elapsed();
    println!("Time elapsed in assert!() is: {:?}", elapsed);
}

fn read_slices_to_vec(data: &[mersenne31::M31x16]) -> Vec<mersenne31::M31x16> {
    let start_time = time::Instant::now();
    let result = data.to_vec();
    println!("read_slices_to_vec time: {:?}", start_time.elapsed());
    result
}
#[test]
fn test_to_vec(){
    use arith::SimdField;
    let start_time = time::Instant::now();
    let mut input_vars: Vec<mersenne31::M31x16> = vec![];
    for i in 0..67108864 {
        let mut tmp = Vec::new();
        for j in 0..16 {
            tmp.push(M31::from(i as u32));
        }
        input_vars.push(mersenne31::M31x16::pack(&tmp));
    }
    println!("init data time: {:?}", start_time.elapsed());
    let data_vec = read_slices_to_vec(&input_vars);
    println!("to_vec time: {:?}", start_time.elapsed());
}

fn read_slices_to_u32vec(data: &[u32]) -> Vec<u32> {
    let start_time = time::Instant::now();
    let result = data.to_vec();
    println!("read_slices_to_vec time: {:?}", start_time.elapsed());
    result
}
// #[test]
// fn test_to_u32vec(){
//     use arith::SimdField;
//     let start_time = time::Instant::now();
//     stacker::grow(128 * 1024 * 1024 * 1024, || {
//         let mut input_vars = [1u32; 1024*1024*1024];
//         println!("init data time: {:?}", start_time.elapsed());
//         let start_time = time::Instant::now();
//         let data_vec1 = read_slices_to_u32vec(&input_vars);
//         let data_vec2 = read_slices_to_u32vec(&input_vars);
//         println!("to_vec time: {:?}", start_time.elapsed());
//         println!("data_vec1: {:?}", data_vec1.len());
//         println!("data_vec2: {:?}", data_vec2.len());
//     });
// }