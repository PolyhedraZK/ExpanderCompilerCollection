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

    println!("\n===== 第一次执行：创建并保存图（BN254） =====");
    let mut ctx1: Context<C> = Context::default();

    // 第一组输入数据（BN254 field 元素）
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
    println!("第一次结果: {:?}", result1);
    assert_eq!(result1, CircuitField::<C>::from(32 * 33 / 2 as u32));

    let computation_graph = ctx1.compile_computation_graph().unwrap();
    ctx1.solve_witness().unwrap();
    println!("开始 setup（可能需要一些时间）...");
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    println!("开始 prove...");
    let proof1 = P::prove(
        &prover_setup,
        &computation_graph,
        ctx1.export_device_memories(),
    );
    println!("开始 verify...");
    assert!(P::verify(&verifier_setup, &computation_graph, &proof1));
    println!("第一次验证通过！");

    println!("\n===== 第二次执行：先 call_kernel（新的 BN254 数据），再 load_graph =====");
    let mut ctx2: Context<C> = Context::default();

    // 第二组输入数据（不同的 BN254 field 元素）
    let mut a2: Vec<Vec<CircuitField<C>>> = vec![];
    for i in 0..16 {
        a2.push(vec![]);
        for j in 0..2 {
            // 使用不同的值：从 1000 开始
            a2[i].push(CircuitField::<C>::from((i * 2 + j + 1000) as u32));
        }
    }
    let a2 = ctx2.copy_to_device(&a2);

    // 先调用 kernels（和第一次一样的顺序）
    let mut b2: DeviceMemoryHandle = None;
    println!("调用第一个 kernel（使用新数据）...");
    call_kernel!(ctx2, kernel_add_2, 16, a2, mut b2).unwrap();

    let b2 = b2.reshape(&[1, 16]);
    let mut c2: DeviceMemoryHandle = None;
    println!("调用第二个 kernel...");
    call_kernel!(ctx2, kernel_add_16, 1, b2, mut c2).unwrap();

    let c2 = c2.reshape(&[]);
    let result2: CircuitField<C> = ctx2.copy_to_host(c2);
    println!("第二次计算结果: {:?}", result2);

    // 验证结果确实不同
    assert_ne!(result1, result2, "两次结果应该不同");

    // 第二次的预期结果：
    // 输入: [1000,1001], [1002,1003], ..., [1030,1031] (共32个数)
    // add_2: 2001, 2005, 2009, ..., 2061 (16个数)
    // add_16: sum(2001, 2005, ..., 2061) = 16 * (2001 + 2061) / 2 = 32496
    let expected2 = CircuitField::<C>::from(32496u32);
    assert_eq!(result2, expected2, "第二次结果应该是 32496");

    // 现在加载图（复用编译好的 kernels）
    println!("加载 computation_graph...");
    ctx2.load_computation_graph(computation_graph.clone()).unwrap();
    println!("图加载成功！");

    // solve_witness（会使用新数据重新计算）
    println!("solve_witness（重新计算 witness）...");
    ctx2.solve_witness().unwrap();
    println!("solve_witness 成功！");

    // prove（使用新数据）
    println!("prove（使用新数据生成证明）...");
    let proof2 = P::prove(
        &prover_setup,
        &computation_graph,
        ctx2.export_device_memories(),
    );
    println!("prove 成功！");

    // verify
    println!("verify（验证新数据的证明）...");
    assert!(P::verify(&verifier_setup, &computation_graph, &proof2));
    println!("✓ 第二次验证通过！");
    println!("✓ 成功使用新的 BN254 数据生成并验证了不同的证明");
    println!("  - 第一次结果: {:?}", result1);
    println!("  - 第二次结果: {:?}", result2);

    P::post_process();
}

#[test]
fn test_bn254_load_graph_with_new_data() {
    test_bn254_load_graph_with_new_data_impl::<_, ExpanderNoOverSubscribe<ZKCudaBN254KZGBatchPCS>>();
}
