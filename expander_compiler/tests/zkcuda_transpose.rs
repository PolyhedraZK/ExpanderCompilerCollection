use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::DummyProvingSystem;
use expander_compiler::zkcuda::{context::*, kernel::*};

#[kernel]
fn sum<C: Config>(api: &mut API<C>, a: &[InputVariable; 16], b: &mut OutputVariable) {
    let mut sum = api.constant(0);
    for i in 0..16 {
        sum = api.add(sum, a[i]);
    }
    *b = sum;
}

#[test]
fn zkcuda_transpose() {
    let kernel_sum: Kernel<M31Config> = compile_sum().unwrap();

    let mut ctx: Context<M31Config, DummyProvingSystem<M31Config>> = Context::default();

    let mut mat: Vec<Vec<M31>> = vec![];
    for i in 0..16 {
        mat.push(vec![]);
        for _ in 0..16 {
            mat[i].push(M31::from((i == 0) as u32));
        }
    }

    // Let the matrix be 4x4x4x4 [a,b,c,d], it has value 1 only when a=b=0
    let mat = ctx.copy_to_device(&mat, false);
    let mat_clone = mat.clone();
    let mut res1 = None;
    call_kernel!(ctx, kernel_sum, mat_clone, mut res1);
    let res1: Vec<M31> = ctx.copy_to_host(res1);
    let expected_res1 = vec![16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    // Now it's [c,d,a,b]
    let mat = mat.reorder_bits(&[4, 5, 6, 7, 0, 1, 2, 3]);
    let mat_clone = mat.clone();
    let mut res2 = None;
    call_kernel!(ctx, kernel_sum, mat_clone, mut res2);
    let res2: Vec<M31> = ctx.copy_to_host(res2);
    let expected_res2 = vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];

    // Now it's [b,c,d,a]
    let mat = mat.reorder_bits(&[2, 3, 4, 5, 6, 7, 0, 1]);
    let mat_clone = mat.clone();
    let mut res3 = None;
    call_kernel!(ctx, kernel_sum, mat_clone, mut res3);
    let res3: Vec<M31> = ctx.copy_to_host(res3);
    let expected_res3 = vec![4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..16 {
        assert_eq!(res1[i], M31::from(expected_res1[i]));
        assert_eq!(res2[i], M31::from(expected_res2[i]));
        assert_eq!(res3[i], M31::from(expected_res3[i]));
    }

    let computation_graph = ctx.to_computation_graph();
    let (prover_setup, verifier_setup) = ctx.proving_system_setup(&computation_graph);
    let proof = ctx.to_proof(&prover_setup);
    assert!(computation_graph.verify(&proof, &verifier_setup));
}
