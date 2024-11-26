use expander_compiler::frontend::*;
use expander_compiler::{
    declare_circuit,
    frontend::{BN254Config, BasicAPI, Define, Variable, API},
};

use crate::constants::BN_TWO_TO_120;


#[inline]
// Assert the variable x is 120 bits, via LogUp
pub(crate) fn assert_u120(
    x: &Variable,
    builder: &mut API<BN254Config>
) {

}

#[inline]
// a + b + carry_in = result + carry_out * 2^120
//
// caller will need to ensure
// - x, y, result are all 120 bits
// - carry_in, carry_out are 1 bit
//
// note: 2^120 is also passed in as it may be reused by other functions
pub(crate) fn assert_add_120_with_carry(
    x: &Variable,
    y: &Variable,
    carry_in: &Variable,
    result: &Variable,
    carry_out: &Variable,
    two_to_120: &Variable,
    builder: &mut API<BN254Config>,
) {
    let left = builder.add(x, y);
    let left = builder.add(left, carry_in);

    let right = builder.mul(carry_out, two_to_120);
    let right = builder.add(right, result);

    builder.assert_is_equal(left, right);
}

#[inline]
// a x b + carry_in = result + carry_out * 2^120
//
// caller will need to ensure
// - x, y, carry_in, result, carry_out are all 120 bits
//
// note: 2^120 is also passed in as it may be reused by other functions
pub(crate) fn assert_mul_120_with_carry(
    x: &Variable,
    y: &Variable,
    carry_in: &Variable,
    result: &Variable,
    carry_out: &Variable,
    two_to_120: &Variable,
    builder: &mut API<BN254Config>,
) {
    let left = builder.mul(x, y);
    let left = builder.add(left, carry_in);

    let right = builder.mul(carry_out, two_to_120);
    let right = builder.add(right, result);

    builder.assert_is_equal(left, right);
}
