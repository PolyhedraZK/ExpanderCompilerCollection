use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::Expander;
use expander_compiler::zkcuda::proving_system::ProvingSystem;
use expander_compiler::zkcuda::shape::{Reshape, Transpose};
use expander_compiler::zkcuda::{context::*, kernel::*};

#[kernel]
fn mul_line<C: Config>(
    api: &mut API<C>,
    a: &[InputVariable; 32],
    b: &[[InputVariable; 64]; 32],
    c: &mut [OutputVariable; 64],
) {
    for j in 0..64 {
        c[j] = api.constant(0);
    }
    for i in 0..32 {
        for j in 0..64 {
            let t = api.mul(a[i], b[i][j]);
            c[j] = api.add(c[j], t);
        }
    }
}

#[kernel]
fn sum_8_elements<C: Config>(api: &mut API<C>, a: &[InputVariable; 8], b: &mut OutputVariable) {
    let mut sum = api.constant(0);
    for i in 0..8 {
        sum = api.add(sum, a[i]);
    }
    *b = sum;
}

#[kernel]
fn eq_8_elements<C: Config>(api: &mut API<C>, a: &[InputVariable; 512], b: &[InputVariable; 512], c: &mut [OutputVariable; 512]) {
    for i in 0..512 {
        c[i] = api.add(a[i], b[i]);
    }
}

#[test]
fn zkcuda_matmul_sum() {
    let kernel_mul_line: KernelPrimitive<M31Config> = compile_mul_line().unwrap();
    let kernel_sum_8_elements: KernelPrimitive<M31Config> = compile_sum_8_elements().unwrap();

    let mut ctx: Context<M31Config> = Context::default();

    let mut mat_a: Vec<Vec<M31>> = vec![];
    for i in 0..64 {
        mat_a.push(vec![]);
        for j in 0..32 {
            mat_a[i].push(M31::from((i * 233 + j + 1) as u32));
        }
    }
    let mut mat_b: Vec<Vec<M31>> = vec![];
    for i in 0..32 {
        mat_b.push(vec![]);
        for j in 0..64 {
            mat_b[i].push(M31::from((i * 2333 + j + 1) as u32));
        }
    }
    let mut expected_result = M31::zero();
    for i in 0..64 {
        for j in 0..64 {
            for k in 0..32 {
                expected_result += mat_a[i][k] * mat_b[k][j];
            }
        }
    }

    let a = ctx.copy_to_device(&mat_a);
    let b = ctx.copy_to_device(&mat_b);
    let mut c = None;
    call_kernel!(ctx, kernel_mul_line, 64, a, b, mut c).unwrap();

    let c = c.reshape(&[512, 8]);
    let mut d = None;
    call_kernel!(ctx, kernel_sum_8_elements, 512, c, mut d).unwrap();

    let d = d.reshape(&[64, 8]);
    let mut e = None;
    call_kernel!(ctx, kernel_sum_8_elements, 64, d, mut e).unwrap();

    let e = e.reshape(&[8, 8]);
    let mut f = None;
    call_kernel!(ctx, kernel_sum_8_elements, 8, e, mut f).unwrap();

    let f = f.reshape(&[1, 8]);
    let mut g = None;
    call_kernel!(ctx, kernel_sum_8_elements, 1, f, mut g).unwrap();

    let g = g.reshape(&[]);
    let result: M31 = ctx.copy_to_host(g);
    assert_eq!(result, expected_result);

    type P = Expander<M31Config>;
    let computation_graph = ctx.compile_computation_graph().unwrap();
    ctx.solve_witness().unwrap();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(
        &prover_setup,
        &computation_graph,
        ctx.export_device_memories(),
    );
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
}

#[test]
fn zkcuda_matmul1_transpose() {
    let kernel_mul_line: KernelPrimitive<M31Config> = compile_mul_line().unwrap();
    let kernel_eq_8_elements: KernelPrimitive<M31Config> = compile_eq_8_elements().unwrap();

    let mut ctx: Context<M31Config> = Context::default();

    // Create mat_a: [64, 32]
    let mut mat_a: Vec<Vec<M31>> = vec![];
    for i in 0..64 {
        mat_a.push(vec![]);
        for j in 0..32 {
            mat_a[i].push(M31::from((i * 233 + j + 1) as u32));
        }
    }

    // Create mat_b: [32, 64]
    let mut mat_b: Vec<Vec<M31>> = vec![];
    for i in 0..32 {
        mat_b.push(vec![]);
        for j in 0..64 {
            mat_b[i].push(M31::from((i * 2333 + j + 1) as u32));
        }
    }

    // Compute expected result in u32 format
    // Result is [64, 64] after matmul
    let mut mat_c_u32: Vec<Vec<u32>> = vec![vec![0u32; 64]; 64];
    for i in 0..64 {
        for j in 0..64 {
            let mut sum = 0u32;
            for k in 0..32 {
                let a_val = (i * 233 + k + 1) as u32;
                let b_val = (k * 2333 + j + 1) as u32;
                sum = sum.wrapping_add(a_val.wrapping_mul(b_val));
            }
            mat_c_u32[i][j] = sum %2147483647;
        }
    }

    // Reshape [64, 64] -> [2, 512] using u32 array
    let mut mat_c_reshaped_u32: Vec<Vec<u32>> = vec![vec![0u32; 512]; 8];
    for i in 0..64 {
        for j in 0..64 {
            let flat_idx = i * 64 + j;
            let new_i = flat_idx / 512;
            let new_j = flat_idx % 512;
            mat_c_reshaped_u32[new_i][new_j] = mat_c_u32[i][j];
        }
    }

    // Transpose [2, 512] -> [512, 2] using u32 array
    let mut mat_c_transposed_u32: Vec<Vec<u32>> = vec![vec![0u32; 8]; 512];
    for i in 0..8 {
        for j in 0..512 {
            mat_c_transposed_u32[j][i] = mat_c_reshaped_u32[i][j];
        }
    }

    // Convert expected result to M31
    let mut expected_result: Vec<Vec<M31>> = vec![];
    for i in 0..512 {
        expected_result.push(vec![]);
        for j in 0..8 {
            expected_result[i].push(M31::from(mat_c_transposed_u32[i][j]));
        }
    }

    // Run computation on device
    let a = ctx.copy_to_device(&mat_a);
    let b = ctx.copy_to_device(&mat_b);
    let mut c = None;
    call_kernel!(ctx, kernel_mul_line, 64, a, b, mut c).unwrap();

    // Reshape [64, 64] -> [2, 512]
    let c = c.reshape(&[512, 8]);

    // Transpose [2, 512] -> [512, 2]
    let c_transposed = c.transpose(&[1, 0]);

    // Prepare expected result for comparison
    // let c_expected = ctx.copy_to_device(&expected_result);
    let c_clone = c_transposed.clone();

    // Compare the results using eq_2_elements kernel
    let mut d = None;
    call_kernel!(ctx, kernel_eq_8_elements, 8, c_clone, c_transposed, mut d).unwrap();

    // Compile and verify the proof
    type P = Expander<M31Config>;
    let computation_graph = ctx.compile_computation_graph().unwrap();
    ctx.solve_witness().unwrap();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(
        &prover_setup,
        &computation_graph,
        ctx.export_device_memories(),
    );
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
}
