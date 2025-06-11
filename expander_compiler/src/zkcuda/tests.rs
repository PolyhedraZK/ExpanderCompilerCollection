use super::kernel::*;
use crate::frontend::*;

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
fn macro_kernel_2<C: Config>(
    api: &mut API<C>,
    a: &InputVariable,
    b: &mut OutputVariable,
    c: &mut InputOutputVariable,
) {
    *b = api.add(*a, *c);
    *c = api.add(*c, *b);
}

#[kernel]
fn macro_kernel_3<C: Config>(
    api: &mut API<C>,
    a: &mut [[[InputOutputVariable; 4]; 8]; 16],
    b: &mut [[[InputOutputVariable; 16]; 8]; 4],
    c: &InputVariable,
) {
    for i in 0..16 {
        for j in 0..8 {
            for k in 0..4 {
                let x = api.add(a[i][j][k], c);
                a[i][j][k] = b[k][j][i];
                b[k][j][i] = x;
            }
        }
    }
}

#[test]
fn compile_macro_kernels() {
    let _ = compile_macro_kernel::<M31Config>();
    let _ = compile_macro_kernel_2::<M31Config>();
    let _ = compile_macro_kernel_3::<M31Config>();
}
