#![allow(clippy::ptr_arg)]
#![allow(clippy::needless_range_loop)]

use expander_compiler::frontend::{
    BasicAPI, CircuitField, Config, Error, SIMDField, Variable, API,
};
use expander_compiler::zkcuda::context::ComputationGraphDefine;
use expander_compiler::zkcuda::shape::Reshape;
use expander_compiler::zkcuda::{
    context::{call_kernel, ComputationGraph, Context, DeviceMemoryHandle},
    kernel::{compile_with_spec_and_shapes, kernel, IOVecSpec, KernelPrimitive},
};

const N_DATA_COPY: usize = 1024 * 32;
const N_KERNEL_REPEAT: usize = 16;

#[kernel]
fn add_2_macro<C: Config>(api: &mut API<C>, a: &[InputVariable; 2 * N_KERNEL_REPEAT], b: &mut OutputVariable) {
    let mut sum = api.constant(0);
    for i in 0..2 * N_KERNEL_REPEAT {
        sum = api.add(sum, a[i]);
    }
    *b = sum;
}

#[kernel]
fn add_16_macro<C: Config>(api: &mut API<C>, a: &[InputVariable; 16], b: &mut OutputVariable) {
    let mut sum = api.constant(0);
    for i in 0..16 {
        sum = api.add(sum, a[i]);
    }
    *b = sum;
}

#[allow(clippy::type_complexity)]
pub fn gen_computation_graph_and_witness<C: Config>(
    input: Option<Vec<Vec<CircuitField<C>>>>,
) -> (ComputationGraph<C>, Option<Vec<Vec<SIMDField<C>>>>) {
    let kernel_add_2: KernelPrimitive<C> = compile_add_2_macro().unwrap();
    let kernel_add_16: KernelPrimitive<C> = compile_add_16_macro().unwrap();

    let mut ctx: Context<C> = Context::default();
    let a = if let Some(input) = input.as_ref() {
        assert_eq!(input.len(), 16 * N_DATA_COPY);
        assert!(input.iter().all(|v| v.len() == 2 * N_KERNEL_REPEAT));
        input.clone()
    } else {
        let mut tmp = vec![vec![]; 16 * N_DATA_COPY];
        for i in 0..16 * N_DATA_COPY {
            for j in 0..2 * N_KERNEL_REPEAT {
                tmp[i].push(CircuitField::<C>::from((i * 2 * N_KERNEL_REPEAT + j + 1) as u32));
            }
        }
        tmp
    };

    // let expected_result = a.iter().flatten().sum::<CircuitField<C>>();

    let a = ctx.copy_to_device(&a);
    let mut b: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_2, 16 * N_DATA_COPY, a, mut b).unwrap();
    let b = b.reshape(&[N_DATA_COPY, 16]);
    let mut c: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_16, N_DATA_COPY, b, mut c).unwrap();
    let c = c.reshape(&[N_DATA_COPY]);
    let _result: Vec<CircuitField<C>> = ctx.copy_to_host(c);
    // assert_eq!(result, expected_result);

    let computation_graph = ctx.compile_computation_graph().unwrap();

    let extended_witness = if input.is_some() {
        ctx.solve_witness().unwrap();
        Some(ctx.export_device_memories())
    } else {
        None
    };

    (computation_graph, extended_witness)
}

pub struct MyCGDef;

impl<C: Config> ComputationGraphDefine<C> for MyCGDef {
    type InputType = Vec<Vec<CircuitField<C>>>;

    // In practice, we may want to read this from a file
    fn get_input() -> Self::InputType {
        let mut input = vec![vec![]; 16 * N_DATA_COPY];
        for i in 0..16 * N_DATA_COPY {
            for j in 0..2 * N_KERNEL_REPEAT {
                input[i].push(CircuitField::<C>::from((i * 2 * N_KERNEL_REPEAT + j + 1) as u32));
            }
        }
        input
    }

    fn gen_computation_graph_and_witness(
        input: Option<Self::InputType>,
    ) -> (ComputationGraph<C>, Option<Vec<Vec<SIMDField<C>>>>) {
        gen_computation_graph_and_witness(input)
    }
}
