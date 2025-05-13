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

declare_circuit!(SHA25637BYTESCircuit {
    input: [Variable; 37],
    output: [Variable; 32],
});

impl Define<M31Config> for SHA25637BYTESCircuit<Variable> {
    fn define<Builder: RootAPI<M31Config>>(&self, builder: &mut Builder) {
        let mut data = self.input.to_vec();
        let n = data.len();
        println!("n: {}", n);
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
            builder.assert_is_equal(*val, self.output[i]);
        }
    }
}

#[test]
fn test_sha256_37bytes_call_expander() {
    let compile_result =
        compile(&SHA25637BYTESCircuit::default(), CompileOptions::default()).unwrap();
    for i in 0..1 {
        let data = [i; 37];
        let mut hash = Sha256::new();
        hash.update(data);
        let output = hash.finalize();
        let mut assignment = SHA25637BYTESCircuit::default();
        for i in 0..37 {
            assignment.input[i] = M31::from(data[i] as u32);
        }
        for i in 0..32 {
            assignment.output[i] = M31::from(output[i] as u32);
        }
        let witness = compile_result
            .witness_solver
            .solve_witness_with_hints(&assignment, &mut EmptyHintCaller)
            .unwrap();
        let output = compile_result.layered_circuit.run(&witness);
        assert_eq!(output, vec![true]);

        let mut expander_circuit = compile_result.layered_circuit.export_to_expander_flatten();
    
        let mpi_config = MPIConfig::prover_new();
    
        let (simd_input, simd_public_input) = witness.to_simd();
        println!("{} {}", simd_input.len(), simd_public_input.len());
        expander_circuit.layers[0].input_vals = simd_input;
        expander_circuit.public_input = simd_public_input.clone();
    
        // prove
        let start_time = std::time::Instant::now();
        expander_circuit.evaluate();
        let elapsed = start_time.elapsed();
        println!("Time elapsed in evaluate!() is: {:?}", elapsed);
        let (claimed_v, proof) = gkr::executor::prove::<M31Config>(&mut expander_circuit, mpi_config.clone());
        let elapsed = start_time.elapsed();
        println!("Time elapsed in prove!() is: {:?}", elapsed);
    
        // verify
        assert!(gkr::executor::verify::<M31Config>(
            &mut expander_circuit,
            mpi_config,
            &proof,
            &claimed_v
        ));
        let elapsed = start_time.elapsed();
        println!("Time elapsed in verify!() is: {:?}", elapsed);
    }
}

#[test]
fn debug_sha256_37bytes() {
    let data = [255; 37];
    let mut hash = Sha256::new();
    hash.update(data);
    let output = hash.finalize();
    let mut assignment = SHA25637BYTESCircuit::default();
    for i in 0..37 {
        assignment.input[i] = M31::from(data[i] as u32);
    }
    for i in 0..32 {
        assignment.output[i] = M31::from(output[i] as u32);
    }
    debug_eval(
        &SHA25637BYTESCircuit::default(),
        &assignment,
        EmptyHintCaller,
    );
}

#[kernel]
fn sha256_37bytes<C: Config>(
    builder: &mut API<C>,
    orign_data: &[InputVariable; 37],
    output_data: &mut [OutputVariable; 32],
) -> Vec<Variable> {
    let mut data = orign_data.to_vec();
    for i in 0..8 {
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
    let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
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
    let kernel_check_sha256_37bytes: Kernel<M31Config> = compile_sha256_37bytes().unwrap();
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
