use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::{Expander, ExpanderNoOverSubscribe, ParallelizedExpander, ProvingSystem,};
use expander_compiler::zkcuda::shape::Reshape;
use expander_compiler::zkcuda::proving_system::expander::config::{ZKCudaBN254Hyrax, ZKCudaBN254HyraxBatchPCS, ZKCudaBN254KZG, ZKCudaBN254KZGBatchPCS,};
use expander_compiler::zkcuda::{context::*, kernel::*};
use gkr::BN254ConfigSha2Hyrax;
use gkr_engine::FieldEngine;

const SIZE:usize = 2;
#[kernel]
fn compare_macro<C: Config>(
	api: &mut API<C>,
	query: &[InputVariable; SIZE],
) {
    let target = api.constant(7);
	 for i in 0..SIZE {
		api.assert_is_equal(target, query[i]);
	}
}

#[test]
fn zkcuda_div(){
    let kernel_comp: KernelPrimitive<BN254Config> = compile_compare_macro().unwrap();

    let mut ctx = Context::<BN254Config>::default();
    let parallel_count = 6; //would fail
    // let parallel_count = 8; //would pass
    let query: Vec<Vec<BN254Fr>> = vec![vec![BN254Fr::from(7u32);SIZE];parallel_count];
    
    let query = ctx.copy_to_device(&query);
    call_kernel!(ctx, kernel_comp, parallel_count, query).unwrap();

    // type P = Expander<M31Config>;
    let computation_graph = ctx.compile_computation_graph().unwrap();
    ctx.solve_witness().unwrap();
    let (prover_setup, _) = ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::setup(&computation_graph);
    ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::prove(
        &prover_setup,
        &computation_graph,
        ctx.export_device_memories(),
    );
    ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::post_process();
}
