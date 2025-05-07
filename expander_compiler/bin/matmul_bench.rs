use expander_compiler::frontend::*;
use expander_compiler::zkcuda::{context::*, kernel::*};

/// N * M matrix times M *K matrix
const N: usize = 16;
const M: usize = 32;
const K: usize = 64;

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

fn zkcuda_matmul_sum() {
    let kernel_mul_line: Kernel<M31Config> = compile_mul_line().unwrap();

    let mut ctx: Context<M31Config> = Context::default();

    let mut mat_a: Vec<Vec<M31>> = vec![];
    for i in 0..N {
        mat_a.push(vec![]);
        for j in 0..M {
            mat_a[i].push(M31::from((i * 233 + j + 1) as u32));
        }
    }
    let mut mat_b: Vec<Vec<M31>> = vec![];
    for i in 0..M {
        mat_b.push(vec![]);
        for j in 0..K {
            mat_b[i].push(M31::from((i * 2333 + j + 1) as u32));
        }
    }
    let mut expected_result = vec![vec![M31::zero(); K]; N];
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
    let result: Vec<Vec<M31>> = ctx.copy_to_host(c);
    assert_eq!(result, expected_result);

    let computation_graph = ctx.to_computation_graph();
    let (prover_setup, verifier_setup) = ctx.proving_system_setup(&computation_graph);
    let proof = ctx.to_proof(&prover_setup);
    assert!(computation_graph.verify(&proof, &verifier_setup));
}

fn main() {
    zkcuda_matmul_sum();
}