use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::expander::config::{
    ZKCudaBN254Hyrax, ZKCudaBN254HyraxBatchPCS, ZKCudaBN254KZG, ZKCudaBN254KZGBatchPCS,
};
use expander_compiler::zkcuda::proving_system::Expander;
use expander_compiler::zkcuda::proving_system::ProvingSystem;
use expander_compiler::zkcuda::proving_system::{ExpanderNoOverSubscribe, ParallelizedExpander};
use expander_compiler::zkcuda::shape::Reshape;
use expander_compiler::zkcuda::{context::*, kernel::*};
use serde::{Deserialize, Serialize};
use serdes::ExpSerde;
const SIZE: usize = 64;
const SIZE2: usize = 4096;
#[kernel]
fn mul_line<C: Config>(
    api: &mut API<C>,
    a: &[InputVariable; SIZE],
    b: &[InputVariable; SIZE],
    c: &mut [OutputVariable; 1],
) {
    let mut sum = api.constant(0);
    for i in 0..SIZE {
        let t = api.mul(a[i], b[i]);
        sum = api.add(sum, t);
    }
    c[0] = sum;
}

#[kernel]
fn sum_8_elements<C: Config>(api: &mut API<C>, a: &[InputVariable; SIZE2], b: &mut OutputVariable) {
    let mut sum = api.constant(0);
    for i in 0..SIZE2 {
        sum = api.add(sum, a[i]);
    }
    *b = sum;
}

// #[test]
// fn zkcuda_matmul_sum() {
//     let kernel_mul_line: KernelPrimitive<M31Config> = compile_mul_line().unwrap();
//     let kernel_sum_8_elements: KernelPrimitive<M31Config> = compile_sum_8_elements().unwrap();

//     let mut ctx: Context<M31Config> = Context::default();

//     let mut mat_a: Vec<Vec<M31>> = vec![];
//     for i in 0..64 {
//         mat_a.push(vec![]);
//         for j in 0..32 {
//             mat_a[i].push(M31::from((i * 233 + j + 1) as u32));
//         }
//     }
//     let mut mat_b: Vec<Vec<M31>> = vec![];
//     for i in 0..32 {
//         mat_b.push(vec![]);
//         for j in 0..64 {
//             mat_b[i].push(M31::from((i * 2333 + j + 1) as u32));
//         }
//     }
//     let mut expected_result = M31::zero();
//     for i in 0..64 {
//         for j in 0..64 {
//             for k in 0..32 {
//                 expected_result += mat_a[i][k] * mat_b[k][j];
//             }
//         }
//     }

//     let a = ctx.copy_to_device(&mat_a);
//     let b = ctx.copy_to_device(&mat_b);
//     let mut c = None;
//     call_kernel!(ctx, kernel_mul_line, 64, a, b, mut c).unwrap();

//     let c = c.reshape(&[512, 8]);
//     let mut d = None;
//     call_kernel!(ctx, kernel_sum_8_elements, 512, c, mut d).unwrap();

//     let d = d.reshape(&[64, 8]);
//     let mut e = None;
//     call_kernel!(ctx, kernel_sum_8_elements, 64, d, mut e).unwrap();

//     let e = e.reshape(&[8, 8]);
//     let mut f = None;
//     call_kernel!(ctx, kernel_sum_8_elements, 8, e, mut f).unwrap();

//     let f = f.reshape(&[1, 8]);
//     let mut g = None;
//     call_kernel!(ctx, kernel_sum_8_elements, 1, f, mut g).unwrap();

//     let g = g.reshape(&[]);
//     let result: M31 = ctx.copy_to_host(g);
//     assert_eq!(result, expected_result);

//     type P = Expander<M31Config>;
//     let computation_graph = ctx.compile_computation_graph().unwrap();
//     ctx.solve_witness().unwrap();
//     let (prover_setup, verifier_setup) = P::setup(&computation_graph);
//     let proof = P::prove(
//         &prover_setup,
//         &computation_graph,
//         ctx.export_device_memories(),
//     );
//     assert!(P::verify(&verifier_setup, &computation_graph, &proof));
// }

#[test]
fn zkcuda_matmul_sum() {
    let kernel_mul_line: KernelPrimitive<BN254Config> = compile_mul_line().unwrap();
    // println!("kernnel_mul_line: {:?}", kernel_mul_line);
    // let file = std::fs::File::create("kernel_mul_line_circuit.txt").unwrap();
    // let writer = std::io::BufWriter::new(file);
    // kernel_mul_line.serialize_into(writer);
    let parallel_count = 64;
    let mut ctx: Context<BN254Config> = Context::default();

    let mut mat_a: Vec<Vec<BN254Fr>> = vec![];
    for i in 0..parallel_count {
        mat_a.push(vec![]);
        for j in 0..64 {
            mat_a[i].push(BN254Fr::from((i * 233 + j + 1) as u32));
        }
    }
    let mut mat_b: Vec<Vec<BN254Fr>> = vec![];
    for i in 0..parallel_count / 4 {
        mat_b.push(vec![]);
        for j in 0..64 {
            mat_b[i].push(BN254Fr::from((i * 2333 + j + 11111) as u32));
        }
    }

    let a = ctx.copy_to_device(&mat_a);
    let b = ctx.copy_to_device(&mat_b);
    let mut c = None;
    call_kernel!(ctx, kernel_mul_line, parallel_count, a, b, mut c).unwrap();
    let computation_graph = ctx.compile_computation_graph().unwrap();
    ctx.solve_witness().unwrap();
    let (prover_setup, _) = ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::setup(&computation_graph);
    let proof = ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::prove(
        &prover_setup,
        &computation_graph,
        ctx.export_device_memories(),
    );
    // let file = std::fs::File::create("proof.txt").unwrap();
    // let writer = std::io::BufWriter::new(file);
    // proof.serialize_into(writer);
    <ExpanderNoOverSubscribe<ZKCudaBN254Hyrax> as ProvingSystem<BN254Config>>::post_process();
}
#[test]
fn zkcuda_sum() {
    let kernel_mul_line: KernelPrimitive<M31Config> = compile_sum_8_elements().unwrap();
    let file = std::fs::File::create("kernel_sum_8_elements.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    kernel_mul_line.serialize_into(writer);
}
