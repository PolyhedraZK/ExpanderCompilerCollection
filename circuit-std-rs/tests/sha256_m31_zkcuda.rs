use circuit_std_rs::sha256::m31::check_sha256_37bytes_256batch_compress;
use expander_compiler::frontend::*;
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



#[test]
fn zkcuda_sha256_37bytes_hint() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);
    let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
    println!("compile_sha256_37bytes() done");
    let data = [255; 37];
    let repeat_time = 64;
    let mut hash = Sha256::new();
    hash.update(data);
    let output = hash.finalize();
    println!("output: {:?}", output);
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
    let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> = Context::new(hint_registry);

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



#[test]
fn zkcuda_sha256_37bytes_simd_hint() {
    use arith::SimdField;
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);
    let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
    println!("compile_sha256_37bytes() done");
    let data = [255; 37];
    let repeat_time = 1;
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
    let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> = Context::new(hint_registry);

    let a = ctx.copy_simd_to_device(&new_input_vars, false);
    let mut c = None;
    let start_time = time::Instant::now();
    call_kernel!(ctx, kernel_check_sha256_37bytes, a, mut c);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
    // let c = c.reshape(&[repeat_time, 32]);
    let result: Vec<Vec<mersenne31::M31x16>> = ctx.copy_simd_to_host(c);
    for i in 0..32 {
        let unpack_output = result[0][i].unpack();
        for j in 0..8 {
            assert_eq!(unpack_output[j], output_vars[i]);
        }
    }
}

#[kernel]
fn check_hashtable<C: Config>(
    builder: &mut API<C>,
    input: &[InputVariable; 37],
    output: &mut [OutputVariable; 64*32],
) {
    let mut seed_bits: Vec<Variable> = vec![];
    for i in 0..8{
        seed_bits.extend_from_slice(&bytes_to_bits(builder, &input[i*4..(i+1)*4]));
    }
    let mut indices = vec![];
    let var0 = builder.constant(0);
    for i  in 0..64 {
        //assume HASHTABLESIZE is less than 2^8
        let var_i = builder.constant(i as u32);
        let index = big_array_add_reduce(builder, &input[33..37], &[var_i, var0, var0, var0], 8);
        indices.push(bytes_to_bits(builder, &index));
    }
    let mut round_bits = vec![];
    round_bits.extend_from_slice(&bytes_to_bits(builder, &[input[32]]));
    let mut inputs = vec![];
    for (i, index) in indices.iter().enumerate().take(64) {
        let mut cur_input = Vec::<Variable>::new();
        cur_input.extend_from_slice(&seed_bits);
        cur_input.extend_from_slice(&index[8..]);
        cur_input.extend_from_slice(&round_bits);
        cur_input.extend_from_slice(&index[..8]);
        inputs.push(cur_input);
    }
    println!("len of inputs: {}", inputs.len());
    let res = sha256_37bytes_256batch_compress(builder, &inputs);
    println!("len of res: {}", res.len());
    for i in 0..64 {
        for j in 0..32 {
            output[i*32+j] = res[i][j];
        }
    }
}




#[test]
fn zkcuda_hashtable_hint() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);
    let kernel_check_hashtable: Kernel<M31Config> = compile_check_hashtable().unwrap();
    println!("compile_check_hashtable() done");
    let data = [255; 32];
    let mut expected_output = vec![];
    let repeat_time = 64;
    for i in 0..64 {
        let mut hash = Sha256::new();
        let mut new_data = vec![];
        new_data.extend_from_slice(&data);
        new_data.push(0);
        new_data.push(i);
        new_data.extend_from_slice(&vec![0,0,0]);
        hash.update(new_data);
        let output = hash.finalize();
        expected_output.push(output);
    }
    let mut input_vars = vec![];
    let mut output_vars = vec![];
    for i in 0..32 {
        input_vars.push(M31::from(data[i] as u32));
    }
    for i in 32..37{
        input_vars.push(M31::from(0 as u32));
    }
    for i in 0..64 {
        for j in 0..32 {
            output_vars.push(M31::from(expected_output[i][j] as u32));
        }
    }
    let mut new_input_vars = vec![];
    for _ in 0..repeat_time {
        new_input_vars.push(input_vars.clone());
    }
    let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> = Context::new(hint_registry);

    let a = ctx.copy_to_device(&new_input_vars, false);
    let mut c = None;
    let start_time = time::Instant::now();
    call_kernel!(ctx, kernel_check_hashtable, a, mut c);
    let elapsed = start_time.elapsed();
    println!("Time elapsed in call_kernel!() is: {:?}", elapsed);
    // let c = c.reshape(&[repeat_time, 32]);
    let result: Vec<M31> = ctx.copy_to_host(c);
    assert_eq!(result, output_vars);
}