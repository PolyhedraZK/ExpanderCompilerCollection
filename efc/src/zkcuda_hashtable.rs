use crate::hashtable::{HASHTABLESIZE, SHA256LEN};
use circuit_std_rs::sha256::m31::sha256_37bytes;
use circuit_std_rs::sha256::m31_utils::{big_array_add, to_binary_hint};
use expander_compiler::frontend::*;
use expander_compiler::zkcuda::context::{call_kernel, Context};
use expander_compiler::zkcuda::kernel::*;
use expander_compiler::zkcuda::proving_system::ExpanderGKRProvingSystem;

#[allow(dead_code)]
fn compute_hashtable_inner<C: Config>(api: &mut API<C>, p: &Vec<Variable>) -> Vec<Variable> {
    let shuffle_round = p[0];
    let start_index = &p[1..5];
    let seed = &p[5..];
    let mut output = vec![];

    let mut indices = vec![Vec::<Variable>::new(); HASHTABLESIZE];
    if HASHTABLESIZE > 256 {
        panic!("HASHTABLESIZE > 256")
    }
    let var0 = api.constant(0);
    for (i, cur_index) in indices.iter_mut().enumerate().take(HASHTABLESIZE) {
        //assume HASHTABLESIZE is less than 2^8
        let var_i = api.constant(i as u32);
        let index = big_array_add(api, &start_index, &[var_i, var0, var0, var0], 8);
        *cur_index = index.to_vec();
    }
    for (_, index) in indices
        .iter()
        .enumerate()
        .take(crate::hashtable::HASHTABLESIZE)
    {
        let mut cur_input = Vec::<Variable>::new();
        cur_input.extend_from_slice(&seed);
        cur_input.push(shuffle_round);
        cur_input.extend_from_slice(index);
        let data_hash = sha256_37bytes(api, &cur_input);
        output.extend(data_hash);
    }

    return output;
}
#[kernel]
fn compute_hashtable<C: Config>(
    api: &mut API<C>,
    input: &[InputVariable; 1 + 4 + SHA256LEN],
    output: &mut [OutputVariable; SHA256LEN * HASHTABLESIZE],
) {
    let outc = api.memorized_simple_call(compute_hashtable_inner, input);
    for i in 0..HASHTABLESIZE {
        for j in 0..SHA256LEN {
            output[i * SHA256LEN + j] = outc[i * SHA256LEN + j]
        }
    }
}

//#[test]
pub fn test_zkcuda_hashtable() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);

    let mut ctx: Context<M31Config, ExpanderGKRProvingSystem<M31Config>, _> =
        Context::new(hint_registry);

    let shuffle_round = 100;
    let start_index = vec![1, 0, 0, 0];
    let mut seed: Vec<u32> = vec![];
    for i in 0..SHA256LEN {
        seed.push(i as u32);
    }

    let mut p: Vec<M31> = vec![];
    p.push(M31::from(shuffle_round));
    for i in 0..4 {
        p.push(M31::from(start_index[i]))
    }
    for i in 0..SHA256LEN {
        p.push(M31::from(seed[i]));
    }

    println!("prepare data ok");
    let p = ctx.copy_to_device(&vec![p], false);

    println!("copy to device ok");
    // println!("p: {:?}", p.clone().unwrap().shape.unwrap());

    let start_time = std::time::Instant::now();
    let kernel: Kernel<M31Config> = compile_compute_hashtable().unwrap();

    let t2 = std::time::Instant::now();
    println!("compile ok, time {:?}", t2.duration_since(start_time));

    let mut out = None;
    call_kernel!(ctx, kernel, p, mut out);
    let t3 = std::time::Instant::now();
    println!("call kernel ok, time {:?}", t3.duration_since(t2));

    // println!("out shape: {:?}", out.clone().unwrap().shape.unwrap());
    // let out = out.reshape(&[SHA256LEN*HASHTABLESIZE]);
    // println!("out shape: {:?}", out.clone().unwrap().shape.unwrap());

    //let out: Vec<M31> = ctx.copy_to_host(out);
    //println!("copy to host ok");
    //println!("out: {:?}", out);

    let computation_graph = ctx.to_computation_graph();

    let proof = ctx.to_proof();

    assert!(computation_graph.verify(&proof));

    let t4 = std::time::Instant::now();

    println!("verify ok, time {:?}", t4.duration_since(t3));
}
