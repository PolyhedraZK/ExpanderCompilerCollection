use expander_compiler::frontend::*;
use expander_compiler::{
    declare_circuit,
    frontend::{BN254Config, BasicAPI, Define, Variable, API},
};

use crate::constants::BN_TWO_TO_120;

#[inline]
// a + b + carry_in = result + carry_out * 2^120
pub(crate) fn assert_add_120_with_carry(
    x: &Variable,
    y: &Variable,
    carry_in: &Variable,
    result: &Variable,
    carry_out: &Variable,
    builder: &mut API<BN254Config>,
) {
    // todo: missing constraints
    // - x, y, result are 120 bits integers
    let two_to_120 = builder.constant(BN_TWO_TO_120);
    let left = builder.add(x, y);
    let left = builder.add(left, carry_in);
    let mut right = builder.mul(carry_out, two_to_120);
    right = builder.add(right, result);

    builder.assert_is_equal(left, right);
}

#[inline]
pub(crate) fn assert_mul_120_with_carry(
    x: &Variable,
    y: &Variable,
    r: &Variable,
    carry: &Variable,
    result: &Variable,
    builder: &mut API<BN254Config>,
) {
    let left = builder.mul(x, y);
    let mut right = builder.mul(carry, r);
    right = builder.add(right, result);

    builder.assert_is_equal(left, right);
}
