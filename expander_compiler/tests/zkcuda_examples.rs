use std::panic;

use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::{
    ExpanderGKRProvingSystem, ParallelizedExpanderGKRProvingSystem, ProvingSystem,
};
use expander_compiler::zkcuda::shape::Reshape;
use expander_compiler::zkcuda::{context::*, kernel::*};
use gkr::{BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use serdes::ExpSerde;

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

fn zkcuda_test<C: Config, P: ProvingSystem<C>>() {
    let kernel_add_2: KernelPrimitive<C> = compile_add_2_macro().unwrap();
    let kernel_add_16: KernelPrimitive<C> = compile_add_16_macro().unwrap();
    println!("{:?}", kernel_add_2.io_shapes());
    println!("{:?}", kernel_add_16.io_shapes());

    let mut ctx: Context<C> = Context::default();
    let mut a: Vec<Vec<CircuitField<C>>> = vec![];
    for i in 0..16 {
        a.push(vec![]);
        for j in 0..2 {
            a[i].push(CircuitField::<C>::from((i * 2 + j + 1) as u32));
        }
    }
    let a = ctx.copy_to_device(&a);
    let mut b: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_2, 16, a, mut b).unwrap();
    let b = b.reshape(&[1, 16]);
    let mut c: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_16, 1, b, mut c).unwrap();
    let c = c.reshape(&[]);
    let result: CircuitField<C> = ctx.copy_to_host(c);
    assert_eq!(result, CircuitField::<C>::from(32 * 33 / 2));

    let computation_graph = ctx.compile_computation_graph().unwrap();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(
        &prover_setup,
        &computation_graph,
        &ctx.export_device_memories(),
    );
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
    P::post_process();
}

#[test]
fn zkcuda_test_single_core() {
    zkcuda_test::<GF2Config, ExpanderGKRProvingSystem<GF2Config>>();
    zkcuda_test::<M31Config, ExpanderGKRProvingSystem<M31Config>>();
    zkcuda_test::<GoldilocksConfig, ExpanderGKRProvingSystem<GoldilocksConfig>>();
    zkcuda_test::<BabyBearConfig, ExpanderGKRProvingSystem<BabyBearConfig>>();
    zkcuda_test::<BN254Config, ExpanderGKRProvingSystem<BN254Config>>();
    zkcuda_test::<BN254Config, ExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>>();
    zkcuda_test::<BN254Config, ExpanderGKRProvingSystem<BN254ConfigSha2KZG>>();
}

#[test]
fn zkcuda_test_multi_core() {
    zkcuda_test::<M31Config, ParallelizedExpanderGKRProvingSystem<M31Config>>();
    zkcuda_test::<GF2Config, ParallelizedExpanderGKRProvingSystem<GF2Config>>();
    zkcuda_test::<GoldilocksConfig, ParallelizedExpanderGKRProvingSystem<GoldilocksConfig>>();
    zkcuda_test::<BabyBearConfig, ParallelizedExpanderGKRProvingSystem<BabyBearConfig>>();
    zkcuda_test::<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254Config>>();
    zkcuda_test::<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2Hyrax>>();

    // The setup phase is now called in an incorrect way for KZG. This does not affect efficiency
    let result = panic::catch_unwind(|| {
        zkcuda_test::<BN254Config, ParallelizedExpanderGKRProvingSystem<BN254ConfigSha2KZG>>()
    });
    assert!(result.is_err());
    <ParallelizedExpanderGKRProvingSystem::<BN254ConfigSha2KZG> as ProvingSystem<BN254Config>>::post_process();
}

#[test]
fn zkcuda_test_simd() {
    use arith::SimdField;
    type P = ExpanderGKRProvingSystem<M31Config>;

    let kernel_add_2_tmp: KernelPrimitive<M31Config> = compile_add_2_macro().unwrap();
    let kernel_add_16: KernelPrimitive<M31Config> = compile_add_16_macro().unwrap();

    let mut buf: Vec<u8> = Vec::new();
    kernel_add_2_tmp.serialize_into(&mut buf).unwrap();
    let kernel_add_2: KernelPrimitive<M31Config> =
        KernelPrimitive::deserialize_from(&mut buf.as_slice()).unwrap();

    let mut ctx: Context<M31Config> = Context::default();
    let mut a: Vec<Vec<mersenne31::M31x16>> = vec![];
    for i in 0..16 {
        a.push(vec![]);
        for j in 0..2 {
            let mut tmp = Vec::new();
            for k in 0..16 {
                tmp.push(M31::from((i * 2 + j + 1 + k) as u32));
            }
            a[i].push(mersenne31::M31x16::pack(&tmp));
        }
    }
    let a = ctx.copy_simd_to_device(&a);
    let mut b = None;
    call_kernel!(ctx, kernel_add_2, 16, a, mut b).unwrap();
    let b = b.reshape(&[1, 16]);
    let mut c = None;
    call_kernel!(ctx, kernel_add_16, 1, b, mut c).unwrap();
    let c = c.reshape(&[]);
    let result: mersenne31::M31x16 = ctx.copy_simd_to_host(c);
    let result = result.unpack();
    for k in 0..16 {
        assert_eq!(result[k], M31::from((32 * 33 / 2 + 32 * k) as u32));
    }

    let computation_graph = ctx.compile_computation_graph().unwrap();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(
        &prover_setup,
        &computation_graph,
        &ctx.export_device_memories(),
    );
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));

    // test serde
    let mut buf_cg: Vec<u8> = Vec::new();
    computation_graph.serialize_into(&mut buf_cg).unwrap();
    let mut buf_proof: Vec<u8> = Vec::new();
    proof.serialize_into(&mut buf_proof).unwrap();

    let computation_graph2 =
        ComputationGraph::<M31Config>::deserialize_from(&mut buf_cg.as_slice()).unwrap();
    let proof2 =
        <P as ProvingSystem<M31Config>>::Proof::deserialize_from(&mut buf_proof.as_slice())
            .unwrap();
    let (_prover_setup2, verifier_setup2) = P::setup(&computation_graph2);
    assert!(P::verify(&verifier_setup2, &computation_graph2, &proof2));
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
    assert!(P::verify(&verifier_setup2, &computation_graph2, &proof));
}

#[test]
fn zkcuda_test_simd_autopack() {
    let kernel_add_2_tmp: KernelPrimitive<M31Config> = compile_add_2_macro().unwrap();
    let kernel_add_16: KernelPrimitive<M31Config> = compile_add_16_macro().unwrap();

    let mut buf: Vec<u8> = Vec::new();
    kernel_add_2_tmp.serialize_into(&mut buf).unwrap();
    let kernel_add_2: KernelPrimitive<M31Config> =
        KernelPrimitive::deserialize_from(&mut buf.as_slice()).unwrap();

    let mut ctx: Context<M31Config> = Context::default();
    let mut a: Vec<Vec<Vec<M31>>> = vec![];
    for k in 0..16 {
        a.push(vec![]);
        for i in 0..16 {
            a[k].push(vec![]);
            for j in 0..2 {
                a[k][i].push(M31::from((i * 2 + j + 1 + k) as u32));
            }
        }
    }
    let a = ctx.copy_to_device_and_pack_simd(&a);
    let mut b = None;
    call_kernel!(ctx, kernel_add_2, 16, a, mut b).unwrap();
    let b = b.reshape(&[1, 16]);
    let mut c = None;
    call_kernel!(ctx, kernel_add_16, 1, b, mut c).unwrap();
    let c = c.reshape(&[]);
    let result: Vec<M31> = ctx.copy_to_host_and_unpack_simd(c);
    for k in 0..16 {
        assert_eq!(result[k], M31::from((32 * 33 / 2 + 32 * k) as u32));
    }

    type P = ExpanderGKRProvingSystem<M31Config>;
    let computation_graph = ctx.compile_computation_graph().unwrap();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(
        &prover_setup,
        &computation_graph,
        &ctx.export_device_memories(),
    );
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
}

fn to_binary<C: Config>(api: &mut API<C>, x: Variable, n_bits: usize) -> Vec<Variable> {
    api.new_hint("myhint.tobinary", &[x], n_bits)
}

fn to_binary_hint(x: &[M31], y: &mut [M31]) -> Result<(), Error> {
    let t = x[0].to_u256();
    for (i, k) in y.iter_mut().enumerate() {
        *k = M31::from_u256(t >> i as u32 & 1);
    }
    Ok(())
}

#[kernel]
fn convert_to_binary<C: Config>(api: &mut API<C>, x: &InputVariable, y: &mut [OutputVariable; 8]) {
    let bits = to_binary(api, *x, 8);
    for i in 0..8 {
        y[i] = bits[i];
    }
}

#[test]
fn zkcuda_to_binary() {
    let mut hint_registry = HintRegistry::<M31>::new();
    hint_registry.register("myhint.tobinary", to_binary_hint);

    let kernel: KernelPrimitive<M31Config> = compile_convert_to_binary().unwrap();
    let mut ctx: Context<M31Config, _> = Context::new(hint_registry);

    let a = M31::from(0x55);
    let a = ctx.copy_to_device(&a);
    let a = a.reshape(&[1]);
    let mut b: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel, 1, a, mut b).unwrap();
    let b = b.reshape(&[8]);
    let result: Vec<M31> = ctx.copy_to_host(b);
    assert_eq!(
        result,
        vec![
            M31::from(1),
            M31::from(0),
            M31::from(1),
            M31::from(0),
            M31::from(1),
            M31::from(0),
            M31::from(1),
            M31::from(0)
        ]
    );

    type P = ExpanderGKRProvingSystem<M31Config>;
    let computation_graph = ctx.compile_computation_graph().unwrap();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(
        &prover_setup,
        &computation_graph,
        &ctx.export_device_memories(),
    );
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
}

#[kernel]
fn assertion<C: Config>(api: &mut API<C>, a: &InputVariable, b: &InputVariable) {
    api.assert_is_equal(*a, *b);
}

#[test]
fn zkcuda_assertion() {
    let kernel_tmp: KernelPrimitive<M31Config> = compile_assertion().unwrap();

    let mut ctx: Context<M31Config> = Context::default();
    let a = ctx.copy_to_device(&M31::from(10u32)).reshape(&[1]);
    let b = ctx.copy_to_device(&M31::from(10u32)).reshape(&[1]);
    call_kernel!(ctx, kernel_tmp, 1, a, b).unwrap();

    type P = ExpanderGKRProvingSystem<M31Config>;
    let computation_graph = ctx.compile_computation_graph().unwrap();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(
        &prover_setup,
        &computation_graph,
        &ctx.export_device_memories(),
    );
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
}

#[test]
#[should_panic]
fn zkcuda_assertion_fail() {
    let kernel_tmp: KernelPrimitive<M31Config> = compile_assertion().unwrap();

    let mut ctx: Context<M31Config> = Context::default();
    let a = ctx.copy_to_device(&M31::from(10u32)).reshape(&[1]);
    let b = ctx.copy_to_device(&M31::from(9u32)).reshape(&[1]);
    call_kernel!(ctx, kernel_tmp, 1, a, b).unwrap();

    type P = ExpanderGKRProvingSystem<M31Config>;
    let computation_graph = ctx.compile_computation_graph().unwrap();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(
        &prover_setup,
        &computation_graph,
        &ctx.export_device_memories(),
    );
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
}
