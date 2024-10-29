use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::{DummyCommitment, DummyProvingSystem};
use expander_compiler::zkcuda::{context::*, kernel::*};

fn kernel_func<C: Config>(api: &mut API<C>, inputs: &mut Vec<Vec<Variable>>) {
    let a = inputs[0][0];
    let b = inputs[0][1];
    let sum = api.add(a, b);
    inputs[1][0] = sum;
}

#[test]
fn zkcuda_1() {
    let kernel: Kernel<M31Config> = compile_with_spec(
        kernel_func,
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

    let mut ctx: Context<M31Config, DummyProvingSystem<M31Config>> = Context::new();
    let a = ctx.copy_to_device(&[M31::from(1u32), M31::from(2u32)]);
    let mut io = vec![Some(a), None];
    let result = ctx.call_kernel(&kernel, &mut io);
    let b = io[1].unwrap();
    let result = ctx.copy_to_host(b);
    assert_eq!(result, vec![M31::from(3u32)]);
    let proof = ctx.get_proof();
}
