use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::expander::config::ZKCudaBN254KZGBatchPCS;
use expander_compiler::zkcuda::proving_system::{ExpanderNoOverSubscribe, ProvingSystem};
use expander_compiler::zkcuda::shape::Reshape;
use expander_compiler::zkcuda::{context::*, kernel::*};

#[kernel]
fn add_2_macro<C: Config>(api: &mut API<C>, a: &[InputVariable; 2], b: &mut OutputVariable) {
    *b = api.add(a[0], a[1]);
}

#[kernel]
fn add_16_macro<C: Config>(api: &mut API<C>, a: &[InputVariable; 16], b: &mut OutputVariable) {
    let mut sum = api.constant(0);
    for i in 0..16 {
        sum = api.add(sum, a[i]);
    }
    *b = sum;
}

fn test_bn254_load_graph_with_new_data_impl<C: Config, P: ProvingSystem<C>>() {
    let kernel_add_2: KernelPrimitive<C> = compile_add_2_macro().unwrap();
    let kernel_add_16: KernelPrimitive<C> = compile_add_16_macro().unwrap();

    println!("\n===== First execution: create and save graph (BN254) =====");
    let mut ctx1: Context<C> = Context::default();

    // First set of input data (BN254 field elements)
    let mut a1: Vec<Vec<CircuitField<C>>> = vec![];
    for i in 0..16 {
        a1.push(vec![]);
        for j in 0..2 {
            a1[i].push(CircuitField::<C>::from((i * 2 + j + 1) as u32));
        }
    }
    let a1 = ctx1.copy_to_device(&a1);
    let mut b1: DeviceMemoryHandle = None;
    call_kernel!(ctx1, kernel_add_2, 16, a1, mut b1).unwrap();
    let b1 = b1.reshape(&[1, 16]);
    let mut c1: DeviceMemoryHandle = None;
    call_kernel!(ctx1, kernel_add_16, 1, b1, mut c1).unwrap();
    let c1 = c1.reshape(&[]);
    let result1: CircuitField<C> = ctx1.copy_to_host(c1);
    println!("First result: {:?}", result1);
    assert_eq!(result1, CircuitField::<C>::from(32 * 33 / 2 as u32));

    let computation_graph = ctx1.compile_computation_graph().unwrap();
    ctx1.solve_witness().unwrap();
    println!("Starting setup (may take some time)...");
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    println!("Starting prove...");
    let proof1 = P::prove(
        &prover_setup,
        &computation_graph,
        ctx1.export_device_memories(),
    );
    println!("Starting verify...");
    assert!(P::verify(&verifier_setup, &computation_graph, &proof1));
    println!("First verification passed!");

    println!("\n===== Second execution: call_kernel first (new BN254 data), then load_graph =====");
    let mut ctx2: Context<C> = Context::default();

    // Second set of input data (different BN254 field elements)
    let mut a2: Vec<Vec<CircuitField<C>>> = vec![];
    for i in 0..16 {
        a2.push(vec![]);
        for j in 0..2 {
            // Use different values: starting from 1000
            a2[i].push(CircuitField::<C>::from((i * 2 + j + 1000) as u32));
        }
    }
    let a2 = ctx2.copy_to_device(&a2);

    // Call kernels first (same order as the first time)
    let mut b2: DeviceMemoryHandle = None;
    println!("Calling first kernel (using new data)...");
    call_kernel!(ctx2, kernel_add_2, 16, a2, mut b2).unwrap();

    let b2 = b2.reshape(&[1, 16]);
    let mut c2: DeviceMemoryHandle = None;
    println!("Calling second kernel...");
    call_kernel!(ctx2, kernel_add_16, 1, b2, mut c2).unwrap();

    let c2 = c2.reshape(&[]);
    let result2: CircuitField<C> = ctx2.copy_to_host(c2);
    println!("Second computation result: {:?}", result2);

    // Verify results are indeed different
    assert_ne!(result1, result2, "The two results should be different");

    // Expected result for the second run:
    // Input: [1000,1001], [1002,1003], ..., [1030,1031] (32 numbers total)
    // add_2: 2001, 2005, 2009, ..., 2061 (16 numbers)
    // add_16: sum(2001, 2005, ..., 2061) = 16 * (2001 + 2061) / 2 = 32496
    let expected2 = CircuitField::<C>::from(32496u32);
    assert_eq!(result2, expected2, "Second result should be 32496");

    // Now load the graph (reuse compiled kernels)
    println!("Loading computation_graph...");
    ctx2.load_computation_graph(computation_graph.clone())
        .unwrap();
    println!("Graph loaded successfully!");

    // solve_witness (will recalculate using new data)
    println!("solve_witness (recalculating witness)...");
    ctx2.solve_witness().unwrap();
    println!("solve_witness succeeded!");

    // prove (using new data)
    println!("prove (generating proof with new data)...");
    let proof2 = P::prove(
        &prover_setup,
        &computation_graph,
        ctx2.export_device_memories(),
    );
    println!("prove succeeded!");

    // verify
    println!("verify (verifying proof with new data)...");
    assert!(P::verify(&verifier_setup, &computation_graph, &proof2));
    println!("✓ Second verification passed!");
    println!("✓ Successfully generated and verified different proofs using new BN254 data");
    println!("  - First result: {:?}", result1);
    println!("  - Second result: {:?}", result2);

    P::post_process();
}

#[test]
fn test_bn254_load_graph_with_new_data() {
    test_bn254_load_graph_with_new_data_impl::<_, ExpanderNoOverSubscribe<ZKCudaBN254KZGBatchPCS>>(
    );
}
