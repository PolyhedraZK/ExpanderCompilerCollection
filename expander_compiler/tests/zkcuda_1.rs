use expander_compiler::frontend::*;
use expander_compiler::zkcuda::proving_system::DummyProvingSystem;
use expander_compiler::zkcuda::{context::*, kernel::*};

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

#[test]
fn zkcuda_1() {
    let kernel_add_2: Kernel<M31Config> = compile_with_spec(
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
    let kernel_add_16: Kernel<M31Config> = compile_with_spec(
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

    let mut ctx: Context<M31Config, DummyProvingSystem<M31Config>> = Context::default();
    let mut a = vec![];
    for i in 0..32 {
        a.push(M31::from(i + 1 as u32));
    }
    let a = ctx.copy_to_device(&a);
    let mut io = vec![Some(a), None];
    ctx.call_kernel(&kernel_add_2, &mut io, 16, &vec![false, false]);
    let b = io[1].unwrap();
    let mut io = vec![Some(b), None];
    ctx.call_kernel(&kernel_add_16, &mut io, 1, &vec![false, false]);
    let c = io[1].unwrap();
    let result = ctx.copy_to_host(c);
    assert_eq!(result, vec![M31::from(32 * 33 / 2)]);
    let proof = ctx.to_proof();
    assert!(proof.verify());
}

#[kernel]
fn macro_kernel<C: Config>(
    api: &mut API<C>,
    a: &[[InputVariable; 4]; 2],
    b: &mut [[OutputVariable; 1]; 4],
    c: &mut [[[InputOutputVariable; 2]; 1]; 4],
) {
    for i in 0..4 {
        b[i][0] = api.add(a[0][i], a[1][i]);
        c[i][0][0] = api.add(c[i][0][0], c[i][0][1]);
    }
}

#[kernel]
fn add_2_macro<C: Config>(api: &mut API<C>, a: &[InputVariable; 2], b: &mut OutputVariable) {
    *b = api.add(a[0], a[1]);
}

#[kernel]
fn add_16_macro<C: Config>(api: &mut API<C>, a: &[InputVariable; 16], b: &mut [OutputVariable; 1]) {
    let mut sum = api.constant(0);
    for i in 0..16 {
        sum = api.add(sum, a[i]);
    }
    b[0] = sum;
}

#[test]
fn zkcuda_2() {
    let kernel_add_2: Kernel<M31Config> = compile_add_2_macro().unwrap();
    let kernel_add_16: Kernel<M31Config> = compile_add_16_macro().unwrap();

    let mut ctx: Context<M31Config, DummyProvingSystem<M31Config>> = Context::default();
    let mut a = vec![];
    for i in 0..32 {
        a.push(M31::from(i + 1 as u32));
    }
    let a = ctx.copy_to_device(&a);
    let mut io = vec![Some(a), None];
    ctx.call_kernel(&kernel_add_2, &mut io, 16, &vec![false, false]);
    let b = io[1].unwrap();
    let mut io = vec![Some(b), None];
    ctx.call_kernel(&kernel_add_16, &mut io, 1, &vec![false, false]);
    let c = io[1].unwrap();
    let result = ctx.copy_to_host(c);
    assert_eq!(result, vec![M31::from(32 * 33 / 2)]);
    let proof = ctx.to_proof();
    assert!(proof.verify());
}
