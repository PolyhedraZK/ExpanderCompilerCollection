use expander_compiler::frontend::*;
use expander_compiler::zkcuda::shape::Reshape;
use expander_compiler::zkcuda::{context2::*, kernel2::*};

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
fn zkcuda_example_tmp() {
    let kernel_add_2: KernelPrimitive<M31Config> = compile_add_2_macro().unwrap();
    let kernel_add_16: KernelPrimitive<M31Config> = compile_add_16_macro().unwrap();
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
    let a = ctx.copy_to_device(&a);
    let mut b: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_2, 16, a, mut b).unwrap();
    let b = b.reshape(&[1, 16]);
    let mut c: DeviceMemoryHandle = None;
    call_kernel!(ctx, kernel_add_16, 1, b, mut c).unwrap();
    let c = c.reshape(&[]);
    let result: M31 = ctx.copy_to_host(c);
    assert_eq!(result, M31::from(32 * 33 / 2));

    ctx.compile_computation_graph();
}
