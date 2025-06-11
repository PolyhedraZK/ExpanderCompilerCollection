#![allow(clippy::needless_range_loop)]
#![allow(clippy::ptr_arg)]

use expander_compiler::frontend::{
    BN254Config, BasicAPI, CircuitField, Config, Error, FieldArith, Variable, API,
};
use expander_compiler::zkcuda::proving_system::{
    ExpanderGKRProvingSystem, ParallelizedExpanderGKRProvingSystem, ProvingSystem,
};
use expander_compiler::zkcuda::shape::Reshape;
use expander_compiler::zkcuda::{context::*, kernel::*};
use gkr::BN254ConfigSha2Hyrax;

/// N * M matrix times M *K matrix
const N: usize = 8;
const M: usize = 512;
const K: usize = 512;

#[kernel]
fn mul_line<C: Config>(
    api: &mut API<C>,
    a: &[InputVariable; M],
    b: &[[InputVariable; K]; M],
    c: &mut [OutputVariable; K],
) {
    for j in 0..K {
        c[j] = api.constant(0);
    }

    for i in 0..M {
        for j in 0..K {
            let t = api.mul(a[i], b[i][j]);
            c[j] = api.add(c[j], t);
        }
    }
}

fn zkcuda_matmul<C: Config, P: ProvingSystem<C>>() {
    let kernel_mul_line: KernelPrimitive<C> = compile_mul_line().unwrap();

    let mut ctx: Context<C> = Context::default();

    let mut mat_a: Vec<Vec<CircuitField<C>>> = vec![];
    for i in 0..N {
        mat_a.push(vec![]);
        for j in 0..M {
            mat_a[i].push(CircuitField::<C>::from((i * 233 + j + 1) as u32));
        }
    }
    let mut mat_b: Vec<Vec<CircuitField<C>>> = vec![];
    for i in 0..M {
        mat_b.push(vec![]);
        for j in 0..K {
            mat_b[i].push(CircuitField::<C>::from((i * 2333 + j + 1) as u32));
        }
    }
    let mut expected_result = vec![vec![CircuitField::<C>::zero(); K]; N];
    for i in 0..N {
        for j in 0..K {
            for k in 0..M {
                expected_result[i][j] += mat_a[i][k] * mat_b[k][j];
            }
        }
    }

    let a = ctx.copy_to_device(&mat_a);
    let b = ctx.copy_to_device(&mat_b);
    let mut c = None;
    call_kernel!(ctx, kernel_mul_line, N, a, b, mut c).unwrap();

    let c = c.reshape(&[N, K]);
    let result: Vec<Vec<CircuitField<C>>> = ctx.copy_to_host(c);
    assert_eq!(result, expected_result);

    let computation_graph = ctx.compile_computation_graph().unwrap();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(
        &prover_setup,
        &computation_graph,
        &ctx.export_device_memories(),
    );
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
}

fn main() {
    zkcuda_matmul::<BN254Config, ExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>>();
    zkcuda_matmul::<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>>();
}
