use expander_compiler::frontend::*;
use expander_compiler::{
    declare_circuit,
    frontend::{BN254Config, Define, Variable, API},
};
use extra::UnconstrainedAPI;
use halo2curves::bn256::Fr;

pub(crate) fn unconstrained_byte_decomposition(
    x: &Variable,
    builder: &mut API<BN254Config>,
) -> Vec<Variable> {
    let mut res = vec![];
    let mut x = x.clone();
    let u8modulus = builder.constant(1<<8);
    for _ in 0..256 / 8 {
        let byte = builder.unconstrained_mod(x, u8modulus);
        x = builder.unconstrained_int_div(x, u8modulus);

        res.push(byte);
    }
    res
}

// assert bit decomposition
// the constant_scalars are 2^8, 2^16, ... 2^248
pub(crate) fn assert_byte_decomposition(
    x: &Variable,
    constant_scalars: &[Variable],
    builder: &mut API<BN254Config>,
) -> Vec<Variable> {
    let bytes = unconstrained_byte_decomposition(x, builder);

    let inner_product = bytes.iter().zip(constant_scalars.iter()).fold(
        builder.constant(Fr::zero()),
        |acc, (byte, scalar)| {
            let product = builder.mul(byte, scalar);
            builder.add(acc, product)
        },
    );

    builder.assert_is_equal(x, inner_product);
    bytes
}
