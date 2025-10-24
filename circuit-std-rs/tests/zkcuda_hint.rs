use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::expander::config::{ZKCudaBN254Hyrax, ZKCudaBN254HyraxBatchPCS, ZKCudaBN254KZG, ZKCudaBN254KZGBatchPCS,};
use expander_compiler::zkcuda::proving_system::{Expander, ExpanderNoOverSubscribe, ParallelizedExpander, ProvingSystem,};
use expander_compiler::zkcuda::{context::*, kernel::*};
use gkr::BN254ConfigSha2Hyrax;
use gkr_engine::FieldEngine;
use expander_compiler::zkcuda::shape::Reshape;
use std::fs;
use std::fs::File;
use serdes::serdes::ExpSerde;
use circuit_std_rs::logup::{LogUpRangeProofTable, query_count_hint, rangeproof_hint};
#[kernel]
fn div_112_1_macro<C: Config>(
	api: &mut API<C>,
	y: &[InputVariable; 11222],
	a: &[InputVariable; 1],
	floor: &[InputVariable; 11222],
) {
	let mut table = LogUpRangeProofTable::new(12);
	table.initial(api);
	for i in 0..11222 {
		let tmp = api.mul(floor[i], a[i%1]);
		let rem = api.sub(y[i], tmp);
		let diff = api.sub(a[i%1], rem);
		table.rangeproof(api, diff, 48);
	}
	table.final_check(api);
}
// #[kernel]
// fn div_112_1_without_hint_macro<C: Config>(
// 	api: &mut API<C>,
// 	y: &[InputVariable; 112],
// 	a: &[InputVariable; 1],
// 	floor: &[InputVariable; 112],
//     query: &[InputVariable; 112*4],
//     query_count: &[InputVariable; 4096],
// ) {
// 	let mut table = LogUpRangeProofTable::new(12);
// 	table.initial(api);
// 	for i in 0..112 {
// 		let tmp = api.mul(floor[i], a[i%1]);
// 		let rem = api.sub(y[i], tmp);
// 		let diff = api.sub(a[i%1], rem);
//         let mut compose = query[i*4+3];
//         let shift = api.constant(4096);
//         for j in (0..3).rev() {
//             compose = api.mul(compose, shift);
//             compose = api.add(compose, query[i*4+j]);
//         }
//         api.assert_is_equal(diff, compose);
// 	}
//     for i in 0..112*4 {
//         table.query_range(query[i]);
//     }
// 	table.final_check(api);
// }
// #[kernel]
// fn div_112_1_without_hint_macro<C: Config>(
// 	api: &mut API<C>,
// 	y: &[InputVariable; 11222],
// 	a: &[InputVariable; 1],
// 	floor: &[InputVariable; 11222],
//     query: &[InputVariable; 11222*4],
//     query_count: &[InputVariable; 4096],
// ) {
// 	let mut table = LogUpRangeProofTable::new(12);
// 	table.initial(api);
// 	for i in 0..11222 {
// 		let tmp = api.mul(floor[i], a[i%1]);
// 		let rem = api.sub(y[i], tmp);
// 		let diff = api.sub(a[i%1], rem);
//         let mut compose = query[i*4+3];
//         let shift = api.constant(4096);
//         for j in (0..3).rev() {
//             compose = api.mul(compose, shift);
//             compose = api.add(compose, query[i*4+j]);
//         }
//         api.assert_is_equal(diff, compose);
// 	}
// 	table.final_check_with_query(api, query, query_count);
// }
#[test]
fn zkcuda_hint() {
    let mut hint_registry = HintRegistry::<BN254Fr>::new();
    hint_registry.register("myhint.querycounthint", query_count_hint);
    hint_registry.register("myhint.rangeproofhint", rangeproof_hint);
    let mut ctx: Context<BN254Config, _> = Context::new(hint_registry);
    let kernel_div_112_1: KernelPrimitive<BN254Config> = compile_div_112_1_macro().unwrap();
    let mut y_data: Vec<Vec<BN254Fr>> = vec![];
    let mut a_data: Vec<BN254Fr> = vec![];
    let mut floor_data: Vec<Vec<BN254Fr>> = vec![];
    for i in 0..71 {
        y_data.push(vec![]);
        floor_data.push(vec![]);
        for j in 0..11222 {
            y_data[i].push(BN254Fr::from(((i * 11222 + j) * 7 + 1) as u32));
            floor_data[i].push(BN254Fr::from((i * 11222 + j) as u32));
        }
    }
    a_data.push(BN254Fr::from(7u32));
    let y_var = ctx.copy_to_device(&y_data);
    let a_var = ctx.copy_to_device(&a_data);
    let floor_var = ctx.copy_to_device(&floor_data);
    call_kernel!(ctx, kernel_div_112_1, 71, y_var, a_var, floor_var).unwrap();
    let computation_graph = ctx.compile_computation_graph().unwrap();
    ctx.solve_witness().unwrap();
    let file = std::fs::File::create("graph.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    computation_graph.serialize_into(writer);
    let (prover_setup, _) = ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::setup(&computation_graph);
    let proof = ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::prove(&prover_setup, &computation_graph, ctx.export_device_memories());
    let file = std::fs::File::create("proof.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    proof.serialize_into(writer);
    <ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax> as ProvingSystem<BN254Config>>::post_process();
}

#[test]
fn zkcuda_without_hint() {
    // let mut ctx: Context<BN254Config, _> = Context::default();
    let mut hint_registry = HintRegistry::<BN254Fr>::new();
    hint_registry.register("myhint.querycounthint", query_count_hint);
    // hint_registry.register("myhint.rangeproofhint", rangeproof_hint);
    let mut ctx: Context<BN254Config, _> = Context::new(hint_registry);
    let kernel_div_112_1_without_hint: KernelPrimitive<BN254Config> = compile_div_112_1_without_hint_macro().unwrap();
    let mut y_data: Vec<Vec<BN254Fr>> = vec![];
    let mut a_data: Vec<BN254Fr> = vec![];
    let mut floor_data: Vec<Vec<BN254Fr>> = vec![];
    let mut query_data: Vec<Vec<BN254Fr>> = vec![];
    let mut query_count_data: Vec<Vec<BN254Fr>> = vec![];
    for i in 0..71 {
        y_data.push(vec![]);
        floor_data.push(vec![]);
        query_data.push(vec![]);
        query_count_data.push(vec![]);
        for j in 0..11222 {
            y_data[i].push(BN254Fr::from(((i * 11222 + j) * 7 + 1) as u32));
            floor_data[i].push(BN254Fr::from((i * 11222 + j) as u32));
            let diff = BN254Fr::from(7u32 - 1u32);
            query_data[i].push(diff);
            for k in 0..3{
                query_data[i].push(BN254Fr::from(0u32));
            }
        }
        for k in 0..4096 {
            query_count_data[i].push(BN254Fr::from(0u32));
        }
        query_count_data[i][0] = BN254Fr::from(336u32);
        query_count_data[i][6] = BN254Fr::from(112u32);
    }
    a_data.push(BN254Fr::from(7u32));
    let y_var = ctx.copy_to_device(&y_data);
    let a_var = ctx.copy_to_device(&a_data);
    let floor_var = ctx.copy_to_device(&floor_data);
    let query_var = ctx.copy_to_device(&query_data);
    let query_count_var = ctx.copy_to_device(&query_count_data);
    call_kernel!(ctx, kernel_div_112_1_without_hint, 71, y_var, a_var, floor_var, query_var,query_count_var).unwrap();
    let computation_graph = ctx.compile_computation_graph().unwrap();
    ctx.solve_witness().unwrap();
    let file = std::fs::File::create("graph.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    computation_graph.serialize_into(writer);
    let (prover_setup, _) = ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::setup(&computation_graph);
    let proof = ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax>::prove(&prover_setup, &computation_graph, ctx.export_device_memories());
    let file = std::fs::File::create("proof.txt").unwrap();
    let writer = std::io::BufWriter::new(file);
    proof.serialize_into(writer);
    <ExpanderNoOverSubscribe::<ZKCudaBN254Hyrax> as ProvingSystem<BN254Config>>::post_process();
}