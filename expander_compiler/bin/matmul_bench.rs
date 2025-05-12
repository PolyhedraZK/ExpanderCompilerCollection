#![allow(clippy::needless_range_loop)]
#![allow(clippy::ptr_arg)]

use expander_compiler::frontend::{
    BasicAPI, CircuitField, Config, Error, FieldArith, M31Config, Variable, API,
};
use expander_compiler::zkcuda::proving_system::{
    ExpanderGKRProvingSystem, ParallelizedExpanderGKRProvingSystem, ProvingSystem,
};
use expander_compiler::zkcuda::{
    context::{call_kernel, Context, Reshape},
    kernel::{compile_with_spec_and_shapes, kernel, IOVecSpec, Kernel},
};

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
    let kernel_mul_line: Kernel<C> = compile_mul_line().unwrap();

    let mut ctx: Context<C, P> = Context::default();

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

    let a = ctx.copy_to_device(&mat_a, false);
    let b = ctx.copy_to_device(&mat_b, true);
    let mut c = None;
    call_kernel!(ctx, kernel_mul_line, a, b, mut c);

    let c = c.reshape(&[N, K]);
    let result: Vec<Vec<CircuitField<C>>> = ctx.copy_to_host(c);
    assert_eq!(result, expected_result);

    let computation_graph = ctx.to_computation_graph();
    let (prover_setup, verifier_setup) = ctx.proving_system_setup(&computation_graph);
    let proof = ctx.to_proof(&prover_setup);
    assert!(computation_graph.verify(&proof, &verifier_setup));
}

fn main() {
    zkcuda_matmul::<M31Config, ExpanderGKRProvingSystem<M31Config>>();
    zkcuda_matmul::<M31Config, ParallelizedExpanderGKRProvingSystem<M31Config>>();
}
