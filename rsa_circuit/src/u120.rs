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
pub(crate) fn assert_u120(x: &Variable, builder: &mut API<BN254Config>) {}

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
    // todo: constrain result to be 120 bits

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

    // todo: constrain result and carry_out to be 120 bits
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
