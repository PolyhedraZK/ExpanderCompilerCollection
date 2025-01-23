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
    let a = ctx.copy_raw_to_device(&a);
    let mut io = vec![a, None];
    ctx.call_kernel_raw(&kernel_add_2, &mut io, 16, &vec![false, false]);
    let b = io[1].clone();
    let mut io = vec![b, None];
    ctx.call_kernel_raw(&kernel_add_16, &mut io, 1, &vec![false, false]);
    let c = io[1].clone();
    let result = ctx.copy_raw_to_host(c);
    assert_eq!(result, vec![M31::from(32 * 33 / 2)]);
    let proof = ctx.to_proof();
    assert!(proof.verify());
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

/*

Design:

let kernel shape be (a,b,c), parallel count=n, simd size=m

acceptable shapes are:
(a,b,c) - broadcast, non simd
(a,b,c,m) - broadcast, simd
(n,a,b,c) - non broadcast, non simd
(n,a,b,c,m) - non broadcast, simd

in case of possible confusion, throw error

---

but shapes has another property
the actual shape of the committed input is (n,pad2(a*b*c))

---

so the final solution:

simd arrays are explicitly defined

a reshape is acceptable if:
let sequence of a*b*c,b*c,c be a1,a2,a3,... and other one be b1,b2,b3,...
there must exist n such that a1=b1, a2=b2, a3=b3, ..., an=bn
and for i>n, ai=2^k and bi=2^k for some k

---

only one type vec_with_shape, it's always simd (or DeviceMemoryHandleShaped)
it has two bool fields: broadcast, simd

and it's underlying data can be determined by the shape and broadcast

if it's perfect 2^n (or, perfect 2^n but the first one), broadcast doesn't matter

*/

#[test]
fn zkcuda_2() {
    let kernel_add_2: Kernel<M31Config> = compile_add_2_macro().unwrap();
    let kernel_add_16: Kernel<M31Config> = compile_add_16_macro().unwrap();
    println!("{:?}", kernel_add_2.io_shapes);
    println!("{:?}", kernel_add_16.io_shapes);

    let mut ctx: Context<M31Config, DummyProvingSystem<M31Config>> = Context::default();
    let mut a = vec![];
    for i in 0..32 {
        a.push(M31::from(i + 1 as u32));
    }
    let a = ctx.copy_raw_to_device(&a);
    let mut io = vec![a, None];
    ctx.call_kernel_raw(&kernel_add_2, &mut io, 16, &vec![false, false]);
    let b = io[1].clone();
    let mut io = vec![b, None];
    ctx.call_kernel_raw(&kernel_add_16, &mut io, 1, &vec![false, false]);
    let c = io[1].clone();
    let result = ctx.copy_raw_to_host(c);
    assert_eq!(result, vec![M31::from(32 * 33 / 2)]);
    let proof = ctx.to_proof();
    assert!(proof.verify());
}
