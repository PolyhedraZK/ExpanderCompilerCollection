use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::{Expander, ProvingSystem};
use expander_compiler::zkcuda::{context::*, kernel::*};

const SIZE: usize = 2;
#[kernel]
fn compare_macro<C: Config>(api: &mut API<C>, query: &[InputVariable; SIZE]) {
    let target = api.constant(7);
    for i in 0..SIZE {
        api.assert_is_equal(target, query[i]);
    }
}

// Data should be padded correctly to avoid failure
#[test]
fn zkcuda_padding() {
    let kernel_comp: KernelPrimitive<BN254Config> = compile_compare_macro().unwrap();

    let mut ctx = Context::<BN254Config>::default();
    let parallel_count = 6;
    let query: Vec<Vec<BN254Fr>> = vec![vec![BN254Fr::from(7u32); SIZE]; parallel_count];

    let query = ctx.copy_to_device(&query);
    call_kernel!(ctx, kernel_comp, parallel_count, query).unwrap();

    let computation_graph = ctx.compile_computation_graph().unwrap();
    ctx.solve_witness().unwrap();
    let (prover_setup, _) = Expander::<BN254Config>::setup(&computation_graph);
    Expander::<BN254Config>::prove(
        &prover_setup,
        &computation_graph,
        ctx.export_device_memories(),
    );
}
