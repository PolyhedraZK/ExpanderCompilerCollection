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
pub(crate) fn add_u120<C: Config>(
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
pub(crate) fn mul_u120<C: Config>(
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

// #[inline]
// // a + b + carry_in = result + carry_out * 2^120
// //
// // caller will need to ensure
// // - x, y, result are all 120 bits
// // - carry_in, carry_out are 1 bit
// //
// // note: 2^120 is also passed in as it may be reused by other functions
// pub(crate) fn assert_add_120_with_carry(
//     x: &Variable,
//     y: &Variable,
//     carry_in: &Variable,
//     result: &Variable,
//     carry_out: &Variable,
//     two_to_120: &Variable,
//     builder: &mut API<BN254Config>,
// ) {
//     let left = builder.add(x, y);
//     let left = builder.add(left, carry_in);

//     let right = builder.mul(carry_out, two_to_120);
//     let right = builder.add(right, result);

//     builder.assert_is_equal(left, right);
// }

// #[inline]
// // a x b + carry_in = result + carry_out * 2^120
// //
// // caller will need to ensure
// // - x, y, carry_in, result, carry_out are all 120 bits
// //
// // note: 2^120 is also passed in as it may be reused by other functions
// pub(crate) fn assert_mul_120_with_carry(
//     x: &Variable,
//     y: &Variable,
//     carry_in: &Variable,
//     result: &Variable,
//     carry_out: &Variable,
//     two_to_120: &Variable,
//     builder: &mut API<BN254Config>,
// ) {
//     let left = builder.mul(x, y);
//     let left = builder.add(left, carry_in);

//     let right = builder.mul(carry_out, two_to_120);
//     let right = builder.add(right, result);

//     builder.assert_is_equal(left, right);
// }
