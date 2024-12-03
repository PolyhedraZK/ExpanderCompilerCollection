use expander_compiler::{
    circuit::ir::{self, dest::Instruction},
    frontend::{
        extra::UnconstrainedAPI, BN254Config, BasicAPI, Config, ToVariableOrValue, Variable, API,
    },
    hints::BuiltinHintIds,
};

#[inline]
// TODO:
// Assert the variable x is 120 bits, via LogUp
pub(crate) fn range_proof_u120(x: &Variable, builder: &mut API<BN254Config>) {}

// Accumulate up to 2^120 variables
pub(crate) fn accumulate_u120(
    x: &[Variable],
    two_to_120: &Variable,
    builder: &mut API<BN254Config>,
) -> (Variable, Variable) {
    assert!(x.len() > 1, "length is {}", x.len());

    // left = x0 + x1 + x2 + ... + x(n-1)
    let mut sum_left = x[0];
    for xi in x.iter().skip(1) {
        sum_left = builder.add(sum_left, xi);
    }

    // right = result + carry_out * 2^120
    let result = builder.unconstrained_mod(sum_left, two_to_120);
    let carry_out = builder.unconstrained_int_div(sum_left, two_to_120);
    let carry_times_two_to_120 = builder.mul(carry_out, two_to_120);
    let sum_right = builder.add(result, carry_times_two_to_120);
    builder.assert_is_equal(sum_left, sum_right);

    range_proof_u120(&carry_out, builder);
    range_proof_u120(&result, builder);

    (result, carry_out)
}

#[inline]
// Add two variables x and y, with a carry_in,
// and return the result and carry_out
// Ensures:
// - result is 120 bits
// - carry_out is 1 bit
// - x + y + carry_in = result + carry_out * 2^120
// Does not ensure:
// - x, y are 120 bits
// - carry_in is 1 bit
pub(crate) fn add_u120(
    x: &Variable,
    y: &Variable,
    carry_in: &Variable,
    two_to_120: &Variable,
    builder: &mut API<BN254Config>,
) -> (Variable, Variable) {
    let x_plus_y = builder.add(x, y);
    let sum = builder.add(x_plus_y, carry_in);

    let carry_out = builder.unconstrained_greater_eq(sum, two_to_120);
    let carry_times_two_to_120 = builder.mul(carry_out, two_to_120);
    let result = builder.sub(sum, carry_times_two_to_120);

    // carry_out is 1 bit
    builder.assert_is_bool(carry_out);
    range_proof_u120(&result, builder);

    (result, carry_out)
}

#[inline]
// Mul two variables x and y, with a carry_in,
// and return the result and carry_out
// Ensures:
// - result is 120 bits
// - carry_out is 120 bit
// - x * y + carry_in = result + carry_out * 2^120
// Does not ensure:
// - x, y are 120 bits
// - carry_in is 120 bit
pub(crate) fn mul_u120(
    x: &Variable,
    y: &Variable,
    carry_in: &Variable,
    two_to_120: &Variable,
    builder: &mut API<BN254Config>,
) -> (Variable, Variable) {
    let x_mul_y = builder.mul(x, y);
    let left = builder.add(x_mul_y, carry_in);

    let carry_out = builder.unconstrained_int_div(left, two_to_120);
    let right = builder.mul(carry_out, two_to_120);

    let result = builder.sub(left, right);

    range_proof_u120(&result, builder);
    range_proof_u120(&carry_out, builder);

    (result, carry_out)
}

// check if x < y
// assumption: x, y are 120 bits
pub(crate) fn is_less_than_u120(
    x: &Variable,
    y: &Variable,
    builder: &mut API<BN254Config>,
) -> Variable {
    let diff = builder.sub(x, y);
    let byte_decomp = crate::util::unconstrained_byte_decomposition(&diff, builder);
    let res = builder.unconstrained_lesser(x, y);

    // if res = 1: x < y, then diff will underflow so byte_decomp[31] will be non-zero
    // if res = 0: x >= y, then diff will not underflow so byte_decomp[31] will be zero
    let zero = builder.constant(0);
    let one = builder.constant(1);
    let one_minus_res = builder.sub(one, res);
    let t1 = builder.mul(one_minus_res, byte_decomp[31]);
    let t2 = builder.mul(res, zero);
    let t3 = builder.add(t1, t2);
    builder.assert_is_zero(t3);

    res
}
