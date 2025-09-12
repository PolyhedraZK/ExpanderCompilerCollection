#![allow(clippy::all)] // disable all clippy warnings in temporary test code

use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::expander::config::{
    ZKCudaBN254KZG, ZKCudaBN254KZGBatchPCS, ZKCudaBN254MIMCKZGBatchPCS,
};
use expander_compiler::zkcuda::proving_system::expander_pcs_defered::BN254ConfigSha2UniKZG;
use expander_compiler::zkcuda::proving_system::{
    Expander, ExpanderNoOverSubscribe, ParallelizedExpander, ProvingSystem,
};
use expander_compiler::zkcuda::shape::Reshape;
use expander_compiler::zkcuda::{context::*, kernel::*};

use gkr::BN254ConfigSha2Hyrax;
use serdes::ExpSerde;

#[kernel]
fn add_2_macro<C: Config>(api: &mut API<C>, a: &[InputVariable; 2], b: &mut OutputVariable) {
    *b = api.add(a[0], a[1]);
}

fn zkcuda_test<C: Config, P: ProvingSystem<C>>() {
    let kernel_add_2: KernelPrimitive<C> = compile_add_2_macro().unwrap();
    println!("{:?}", kernel_add_2.io_shapes());

    let mut ctx: Context<C> = Context::default();
    let mut a: Vec<Vec<CircuitField<C>>> = vec![];
    for i in 0..4 {
        a.push(vec![]);
        for j in 0..2 {
            a[i].push(CircuitField::<C>::from((i * 2 + j + 1) as u32));
        }
    }
    let a = ctx.copy_to_device(&a);
    let mut b: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_2, 32, a, mut b).unwrap(); // ideally, broadcast from (4,2) to (32,2)
    let _result: CircuitField<C> = ctx.copy_to_host(b);
    // assert_eq!(result, CircuitField::<C>::from(32 * 33 / 2 as u32));

    let computation_graph = ctx.compile_computation_graph().unwrap();
    ctx.solve_witness().unwrap();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(
        &prover_setup,
        &computation_graph,
        ctx.export_device_memories(),
    );
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
    P::post_process();
}

#[test]
fn zkcuda_broadcast() {
    // DO NOT USE DUMMY PROVING SYSTEM IN PRODUCTION!!!
    zkcuda_test::<
        M31Config,
        expander_compiler::zkcuda::proving_system::DummyProvingSystem<M31Config>,
    >();
}
