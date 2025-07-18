use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::expander::config::{
    ZKCudaBN254Hyrax, ZKCudaBN254HyraxBatchPCS, ZKCudaBN254KZG, ZKCudaBN254KZGBatchPCS,
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

#[kernel]
fn add_16_macro<C: Config>(api: &mut API<C>, a: &[InputVariable; 16], b: &mut OutputVariable) {
    let mut sum = api.constant(0);
    for i in 0..16 {
        sum = api.add(sum, a[i]);
    }
    *b = sum;
}

pub fn gen_computation_graph_and_witness<C: Config>(
    input: Option<Vec<Vec<CircuitField<C>>>>,
) -> (ComputationGraph<C>, Option<Vec<Vec<SIMDField<C>>>>) {
    let kernel_add_2: KernelPrimitive<C> = compile_add_2_macro().unwrap();
    let kernel_add_16: KernelPrimitive<C> = compile_add_16_macro().unwrap();

    let mut ctx: Context<C> = Context::default();
    let a = if let Some(input) = input.as_ref() {
        assert_eq!(input.len(), 16);
        assert!(input.iter().all(|v| v.len() == 2));
        input.clone()
    } else {
        let mut tmp = vec![vec![]; 16];
        for i in 0..16 {
            for j in 0..2 {
                tmp[i].push(CircuitField::<C>::from((i * 2 + j + 1) as u32));
            }
        }
        tmp
    };

    let expected_result = a.iter().flat_map(|v| v).sum::<CircuitField<C>>();

    let a = ctx.copy_to_device(&a);
    let mut b: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_2, 16, a, mut b).unwrap();
    let b = b.reshape(&[1, 16]);
    let mut c: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_16, 1, b, mut c).unwrap();
    let c = c.reshape(&[]);
    let result: CircuitField<C> = ctx.copy_to_host(c);
    assert_eq!(result, expected_result);

    let computation_graph = ctx.compile_computation_graph().unwrap();

    let extended_witness = if let Some(_) = input {
        ctx.solve_witness().unwrap();
        Some(ctx.export_device_memories())
    } else {
        None
    };

    (computation_graph, extended_witness)
}
