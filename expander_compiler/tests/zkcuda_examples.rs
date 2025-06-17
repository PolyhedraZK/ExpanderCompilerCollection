use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proof::ComputationGraph;
use expander_compiler::zkcuda::proving_system::expander_pcs_defered::BN254ConfigSha2UniKZG;
use expander_compiler::zkcuda::proving_system::{
    Expander, ExpanderPCSDefered, ParallelizedExpander, ProvingSystem,
};
use expander_compiler::zkcuda::{context::*, kernel::*};

use gkr::{BN254ConfigSha2Hyrax, BN254ConfigSha2KZG};
use gkr_engine::FieldEngine;
use serdes::ExpSerde;

fn add_2<C: Config>(api: &mut API<C>, inputs: &mut Vec<Vec<Variable>>) {
    let a = inputs[0][0];
    let b = inputs[0][1];
    let sum = api.add(a, b);
    inputs[1][0] = sum;
}

fn add_16<C: Config>(api: &mut API<C>, inputs: &mut Vec<Vec<Variable>>) {
    let mut sum = api.constant(0);
    for i in 0..16 {
        sum = api.add(sum, inputs[0][i]);
    }
    inputs[1][0] = sum;
}

fn zkcuda_1_expander<C: Config, P: ProvingSystem<C>>() {
    let kernel_add_2: Kernel<C> = compile_with_spec(
        add_2,
        &[
            IOVecSpec {
                len: 2,
                is_input: true,
                is_output: false,
            },
            IOVecSpec {
                len: 1,
                is_input: false,
                is_output: true,
            },
        ],
    )
    .unwrap();
    let kernel_add_16: Kernel<C> = compile_with_spec(
        add_16,
        &[
            IOVecSpec {
                len: 16,
                is_input: true,
                is_output: false,
            },
            IOVecSpec {
                len: 1,
                is_input: false,
                is_output: true,
            },
        ],
    )
    .unwrap();

    let mut ctx = Context::<C>::default();
    let mut a = vec![];
    for i in 0..32 {
        a.push(<C::FieldConfig as FieldEngine>::CircuitField::from(
            i + 1_u32,
        ));
    }
    let a = ctx.copy_raw_to_device(&a);
    let mut io = vec![a, None];
    ctx.call_kernel_raw(&kernel_add_2, &mut io, 16, &[false, false]);
    let b = io[1].clone();
    let mut io = vec![b, None];
    ctx.call_kernel_raw(&kernel_add_16, &mut io, 1, &[false, false]);
    let c = io[1].clone();
    let result = ctx.copy_raw_to_host(c);
    assert_eq!(
        result,
        vec![<C::FieldConfig as FieldEngine>::CircuitField::from(
            32 * 33 / 2
        )]
    );

    let computation_graph = ctx.to_computation_graph();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(&prover_setup, &computation_graph, &ctx.device_memories);
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
    P::post_process();
}

#[test]
fn zkcuda_1_single_core() {
    zkcuda_1_expander::<GF2Config, Expander<GF2Config>>();
    zkcuda_1_expander::<M31Config, Expander<M31Config>>();
    zkcuda_1_expander::<GoldilocksConfig, Expander<GoldilocksConfig>>();
    zkcuda_1_expander::<BabyBearConfig, Expander<BabyBearConfig>>();
    zkcuda_1_expander::<BN254Config, Expander<BN254Config>>();
    zkcuda_1_expander::<BN254Config, Expander<BN254ConfigSha2Hyrax>>();
    zkcuda_1_expander::<BN254Config, Expander<BN254ConfigSha2KZG>>();
}

#[test]
fn zkcuda_1_multi_core() {
    zkcuda_1_expander::<M31Config, ParallelizedExpander<M31Config>>();
    zkcuda_1_expander::<GF2Config, ParallelizedExpander<GF2Config>>();
    zkcuda_1_expander::<GoldilocksConfig, ParallelizedExpander<GoldilocksConfig>>();
    zkcuda_1_expander::<BabyBearConfig, ParallelizedExpander<BabyBearConfig>>();
    zkcuda_1_expander::<BN254Config, ParallelizedExpander<BN254Config>>();
    zkcuda_1_expander::<BN254Config, ParallelizedExpander<BN254ConfigSha2Hyrax>>();
    zkcuda_1_expander::<BN254Config, ParallelizedExpander<BN254ConfigSha2KZG>>();

    zkcuda_1_expander::<BN254Config, ExpanderPCSDefered<BN254ConfigSha2Hyrax>>();
    zkcuda_1_expander::<BN254Config, ExpanderPCSDefered<BN254ConfigSha2UniKZG>>();
}

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

#[test]
fn zkcuda_2() {
    let kernel_add_2: Kernel<M31Config> = compile_add_2_macro().unwrap();
    let kernel_add_16: Kernel<M31Config> = compile_add_16_macro().unwrap();
    println!("{:?}", kernel_add_2.io_shapes);
    println!("{:?}", kernel_add_16.io_shapes);

    let mut ctx: Context<M31Config> = Context::default();
    let mut a: Vec<Vec<M31>> = vec![];
    for i in 0..16 {
        a.push(vec![]);
        for j in 0..2 {
            a[i].push(M31::from((i * 2 + j + 1) as u32));
        }
    }
    let a = ctx.copy_to_device(&a, false);
    let mut b: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_2, a, mut b);
    let b = b.reshape(&[1, 16]);
    let mut c: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_16, b, mut c);
    let c = c.reshape(&[]);
    let result: M31 = ctx.copy_to_host(c);
    assert_eq!(result, M31::from(32 * 33 / 2));

    let computation_graph = ctx.to_computation_graph();
    let (prover_setup, verifier_setup) = Expander::<M31Config>::setup(&computation_graph);
    let proof =
        Expander::<M31Config>::prove(&prover_setup, &computation_graph, &ctx.device_memories);
    assert!(Expander::<M31Config>::verify(
        &verifier_setup,
        &computation_graph,
        &proof
    ));
}

#[test]
fn zkcuda_2_simd() {
    use arith::SimdField;
    type P = Expander<M31Config>;

    let kernel_add_2_tmp: Kernel<M31Config> = compile_add_2_macro().unwrap();
    let kernel_add_16: Kernel<M31Config> = compile_add_16_macro().unwrap();

    let mut buf: Vec<u8> = Vec::new();
    kernel_add_2_tmp.serialize_into(&mut buf).unwrap();
    let kernel_add_2: Kernel<M31Config> = Kernel::deserialize_from(&mut buf.as_slice()).unwrap();

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
    let a = ctx.copy_simd_to_device(&a, false);
    let mut b = None;
    call_kernel!(ctx, kernel_add_2, a, mut b);
    let b = b.reshape(&[1, 16]);
    let mut c = None;
    call_kernel!(ctx, kernel_add_16, b, mut c);
    let c = c.reshape(&[]);
    let result: mersenne31::M31x16 = ctx.copy_simd_to_host(c);
    let result = result.unpack();
    for k in 0..16 {
        assert_eq!(result[k], M31::from((32 * 33 / 2 + 32 * k) as u32));
    }

    let computation_graph = ctx.to_computation_graph();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(&prover_setup, &computation_graph, &ctx.device_memories);
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
fn zkcuda_2_simd_autopack() {
    let kernel_add_2_tmp: Kernel<M31Config> = compile_add_2_macro().unwrap();
    let kernel_add_16: Kernel<M31Config> = compile_add_16_macro().unwrap();

    let mut buf: Vec<u8> = Vec::new();
    kernel_add_2_tmp.serialize_into(&mut buf).unwrap();
    let kernel_add_2: Kernel<M31Config> = Kernel::deserialize_from(&mut buf.as_slice()).unwrap();

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
    let a = ctx.copy_to_device_and_pack_simd(&a, false);
    let mut b = None;
    call_kernel!(ctx, kernel_add_2, a, mut b);
    let b = b.reshape(&[1, 16]);
    let mut c = None;
    call_kernel!(ctx, kernel_add_16, b, mut c);
    let c = c.reshape(&[]);
    let result: Vec<M31> = ctx.copy_to_host_and_unpack_simd(c);
    for k in 0..16 {
        assert_eq!(result[k], M31::from((32 * 33 / 2 + 32 * k) as u32));
    }

    type P = Expander<M31Config>;
    let computation_graph = ctx.to_computation_graph();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(&prover_setup, &computation_graph, &ctx.device_memories);
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

    let kernel: Kernel<M31Config> = compile_convert_to_binary().unwrap();
    let mut ctx: Context<M31Config, _> = Context::new(hint_registry);

    let a = M31::from(0x55);
    let a = ctx.copy_to_device(&a, false);
    let a = a.reshape(&[1]);
    let mut b: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel, a, mut b);
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

    type P = Expander<M31Config>;
    let computation_graph = ctx.to_computation_graph();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(&prover_setup, &computation_graph, &ctx.device_memories);
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
}

#[kernel]
fn assertion<C: Config>(api: &mut API<C>, a: &InputVariable, b: &InputVariable) {
    api.assert_is_equal(*a, *b);
}

#[test]
fn zkcuda_assertion() {
    let kernel_tmp: Kernel<M31Config> = compile_assertion().unwrap();

    let mut ctx: Context<M31Config> = Context::default();
    let a = ctx.copy_to_device(&M31::from(10u32), false).reshape(&[1]);
    let b = ctx.copy_to_device(&M31::from(10u32), false).reshape(&[1]);
    call_kernel!(ctx, kernel_tmp, a, b);

    type P = Expander<M31Config>;
    let computation_graph = ctx.to_computation_graph();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(&prover_setup, &computation_graph, &ctx.device_memories);
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
}

#[test]
#[should_panic]
fn zkcuda_assertion_fail() {
    let kernel_tmp: Kernel<M31Config> = compile_assertion().unwrap();

    let mut ctx: Context<M31Config> = Context::default();
    let a = ctx.copy_to_device(&M31::from(10u32), false).reshape(&[1]);
    let b = ctx.copy_to_device(&M31::from(9u32), false).reshape(&[1]);
    call_kernel!(ctx, kernel_tmp, a, b);

    type P = Expander<M31Config>;
    let computation_graph = ctx.to_computation_graph();
    let (prover_setup, verifier_setup) = P::setup(&computation_graph);
    let proof = P::prove(&prover_setup, &computation_graph, &ctx.device_memories);
    assert!(P::verify(&verifier_setup, &computation_graph, &proof));
}
