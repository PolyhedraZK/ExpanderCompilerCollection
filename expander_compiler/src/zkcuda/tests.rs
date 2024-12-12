use super::{context::*, kernel::*, proving_system::*};
use crate::frontend::*;

fn add_2<C: Config>(api: &mut API<C>, inputs: &mut Vec<Vec<Variable>>) {
    let a = inputs[0][0];
    let b = inputs[0][1];
    let sum = api.add(a, b);
    inputs[1][0] = sum;
}

fn div_2x8<C: Config>(api: &mut API<C>, inputs: &mut Vec<Vec<Variable>>) {
    let mut s = vec![];
    for i in 0..8 {
        s.push(api.div(inputs[0][i], inputs[1][i], true));
    }
    for i in 0..8 {
        inputs[2][i] = s[i];
    }
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
    let kernel_div_2x8: Kernel<M31Config> = compile_with_spec(
        div_2x8,
        &[
            IOVecSpec {
                len: 8,
                is_input: true,
                is_output: false,
            },
            IOVecSpec {
                len: 8,
                is_input: true,
                is_output: false,
            },
            IOVecSpec {
                len: 8,
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

    let mut ctx: Context<M31Config, DummyProvingSystem<M31Config>> = Context::default();
    let mut a = vec![];
    let mut b = vec![];
    for i in 0..16 {
        a.push(M31::from((i + 1) * (i % 8 + 1) as u32));
    }
    for i in 0..8 {
        b.push(M31::from(i + 1 as u32));
    }
    let a = ctx.copy_to_device(&a);
    let b = ctx.copy_to_device(&b);
    let mut io = vec![Some(a), Some(b), None];
    ctx.call_kernel(&kernel_div_2x8, &mut io, 2, &vec![false, true, false]);
    let c = io[2].unwrap();
    let mut io = vec![Some(c), None];
    ctx.call_kernel(&kernel_add_16, &mut io, 1, &vec![false, false]);
    let c = io[1].unwrap();
    let result = ctx.copy_to_host(c);
    assert_eq!(result, vec![M31::from(16 * 17 / 2)]);
    let proof = ctx.to_proof();
    assert!(proof.verify());
}

fn div_2x5<C: Config>(api: &mut API<C>, inputs: &mut Vec<Vec<Variable>>) {
    let mut s = vec![];
    for i in 0..5 {
        s.push(api.div(inputs[0][i], inputs[1][i], true));
    }
    for i in 0..5 {
        inputs[2][i] = s[i];
    }
}

fn add_5<C: Config>(api: &mut API<C>, inputs: &mut Vec<Vec<Variable>>) {
    let mut sum = api.constant(0);
    for i in 0..5 {
        sum = api.add(sum, inputs[0][i]);
    }
    inputs[1][0] = sum;
}

#[test]
fn zkcuda_2() {
    let kernel_add_5: Kernel<M31Config> = compile_with_spec(
        add_5,
        &[
            IOVecSpec {
                len: 5,
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
    let kernel_div_2x5: Kernel<M31Config> = compile_with_spec(
        div_2x5,
        &[
            IOVecSpec {
                len: 5,
                is_input: true,
                is_output: false,
            },
            IOVecSpec {
                len: 5,
                is_input: true,
                is_output: false,
            },
            IOVecSpec {
                len: 5,
                is_input: false,
                is_output: true,
            },
        ],
    )
    .unwrap();

    let mut ctx: Context<M31Config, DummyProvingSystem<M31Config>> = Context::default();
    let mut a = vec![];
    let mut b = vec![];
    for j in 0..5 {
        for i in j * 5..j * 5 + 5 {
            a.push(M31::from((i + 1) * (i % 5 + 1) as u32));
        }
        for _ in 0..3 {
            a.push(M31::from(0));
        }
    }
    for _ in 0..24 {
        a.push(M31::from(0));
    }
    for i in 0..5 {
        b.push(M31::from(i + 1 as u32));
    }
    for _ in 0..3 {
        b.push(M31::from(0));
    }
    let a = ctx.copy_to_device(&a);
    let b = ctx.copy_to_device(&b);
    let mut io = vec![Some(a), Some(b), None];
    ctx.call_kernel(&kernel_div_2x5, &mut io, 5, &vec![false, true, false]);
    let c = io[2].unwrap();
    let mut io = vec![Some(c), None];
    ctx.call_kernel(&kernel_add_5, &mut io, 5, &vec![false, false]);
    let c = io[1].unwrap();
    let mut io = vec![Some(c), None];
    ctx.call_kernel(&kernel_add_5, &mut io, 1, &vec![false, false]);
    let c = io[1].unwrap();
    let result = ctx.copy_to_host(c);
    assert_eq!(result, vec![M31::from(25 * 26 / 2)]);
    let proof = ctx.to_proof();
    assert!(proof.verify());
}