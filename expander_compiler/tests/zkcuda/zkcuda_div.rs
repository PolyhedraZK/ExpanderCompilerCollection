use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::{Expander, ExpanderNoOverSubscribe, ParallelizedExpander, ProvingSystem,};
use expander_compiler::zkcuda::shape::Reshape;
use expander_compiler::zkcuda::proving_system::expander::config::{ZKCudaBN254Hyrax, ZKCudaBN254HyraxBatchPCS, ZKCudaBN254KZG, ZKCudaBN254KZGBatchPCS,};
use expander_compiler::zkcuda::{context::*, kernel::*};
use gkr::BN254ConfigSha2Hyrax;
use gkr_engine::FieldEngine;



#[kernel]
fn div_49_macro<C: Config>(
	api: &mut API<C>,
	y: &[InputVariable; 49],
	a: &InputVariable,
	floor: &[InputVariable; 49],
	query: &[InputVariable; 49*4],
	query_count: &[InputVariable; 4096],
) {
	 for i in 0..49 {
		let tmp = api.mul(floor[i], a);
		let rem = api.sub(y[i], tmp);
		let diff = api.sub(a, rem);
		//check a - rem = compose(query[i*4:i*4+4])
		let mut compose = query[i*4+3];
		let shift = api.constant(4096);
		for j in (0..3).rev() {
			compose = api.mul(compose, shift);
			compose = api.add(compose, query[i*4+j]);
		}
		api.assert_is_equal(diff, compose);
	}
}

#[test]
fn zkcuda_div(){
    let kernel_div_49: KernelPrimitive<BN254Config> = compile_div_49_macro().unwrap();

    let mut ctx = Context::<BN254Config>::default();
    
    let mut y: Vec<Vec<BN254Fr>> = vec![];
    for i in 0..96 {
        y.push(vec![]);
        for j in 0..49 {
            y[i].push(BN254Fr::from((i * 77 + j * 7) as u32));
        }
    }
    let mut floor: Vec<Vec<BN254Fr>> = vec![];
    for i in 0..96 {
        floor.push(vec![]);
        for j in 0..49 {
            floor[i].push(BN254Fr::from((i * 11 + j) as u32));
        }
    }
    let a = BN254Fr::from(7u32);
    let mut query: Vec<Vec<BN254Fr>> = vec![];
    for i in 0..96 {
        query.push(vec![]);
        for j in 0..49*4 {
            if j % 4 == 0 {
                query[i].push(BN254Fr::from(7u32));
            } else {
                query[i].push(BN254Fr::zero());
            }
        }
    }
    let mut querycount: Vec<Vec<BN254Fr>> = vec![];
    for i in 0..96 {
        querycount.push(vec![]);
        for j in 0..4096 {
            if j == 7 {
                querycount[i].push(BN254Fr::from(49u32));
            } else {
                querycount[i].push(BN254Fr::from(49*3 as u32));
            }
        }
    }

    let y = ctx.copy_to_device(&y);
    let a = ctx.copy_to_device(&a);
    let floor = ctx.copy_to_device(&floor);
    let query = ctx.copy_to_device(&query);
    let querycount = ctx.copy_to_device(&querycount);
    call_kernel!(ctx, kernel_div_49, 96, y, a, floor, query, querycount).unwrap();

    // type P = Expander<M31Config>;
    let computation_graph = ctx.compile_computation_graph().unwrap();
    ctx.solve_witness().unwrap();
    let (prover_setup, verifier_setup) = ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::setup(&computation_graph);
    let proof = ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::prove(
        &prover_setup,
        &computation_graph,
        ctx.export_device_memories(),
    );
    assert!(ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::verify(&verifier_setup, &computation_graph, &proof));
}
