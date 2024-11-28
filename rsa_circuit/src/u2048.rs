use expander_compiler::frontend::{BN254Config, BasicAPI, Variable, API};

use crate::{constants::N_LIMBS, };

#[derive(Debug, Clone, Copy)]
pub struct U2048Variable {
    pub limbs: [Variable; N_LIMBS],
}

impl U2048Variable {
    #[inline]
    pub fn from_raw(limbs: [Variable; N_LIMBS]) -> Self {
        Self { limbs }
    }

    // #[inline]
    // // add two U2048 variables without mod reductions
    // // the result may be a 2049 bits number. it can still be hosted by N_LIMBS limbs
    // fn assert_add_without_mod(
    //     x: &U2048Variable,
    //     y: &U2048Variable,
    //     result: &U2048Variable,
    //     carries: &[Variable; N_LIMBS + 1],
    //     two_to_120: &Variable,
    //     builder: &mut API<BN254Config>,
    // ) {
    //     // TODO assert the inputs are 120 bits
    //     // ...

    //     // Assert the carries are binary
    //     for i in 0..N_LIMBS + 1 {
    //         let square = builder.mul(carries[i], carries[i]);
    //         builder.assert_is_equal(square, carries[i]);
    //     }

    //     // // Assert the addition is correct
    //     // for i in 0..N_LIMBS {
    //     //     assert_add_120_with_carry(
    //     //         &x.limbs[i],
    //     //         &y.limbs[i],
    //     //         &carries[i],
    //     //         &result.limbs[i],
    //     //         &carries[i + 1],
    //     //         &two_to_120,
    //     //         builder,
    //     //     );
    //     // }
    // }

    #[inline]
    // add two U2048 variables with mod reductions
    // this is done by asserting
    // a + b + carries[0] = result + carries[N_LIMB] * modulus
    //
    // - carries: used by addition a + b
    // - helper_carries: used by addition result + carries[N_LIMB] * modulus
    fn assert_add(
        x: &U2048Variable,
        y: &U2048Variable,
        result: &U2048Variable,
        modulus: &U2048Variable,
        carries: &[Variable; N_LIMBS + 1],
        helper_carries: &[Variable; N_LIMBS + 1],
        two_to_120: &Variable,
        builder: &mut API<BN254Config>,
    ) {
    }
}

// /// assert
// /// a + b + carry_in = result + carry_out * modulus
// fn add_without_mod(
//     x: &U2048Variable,
//     y: &U2048Variable,
//     result: &U2048Variable,
//     modulus: &U2048Variable,
//     carry_in: &Variable,
//     carry_out: &Variable,
//     // temp carries that are used for u120 additions
//     tmp_carries: &[Variable; N_LIMBS - 1],
//     two_to_120: &Variable,
//     builder: &mut API<BN254Config>,
// ) {
//     let mut carry = carry_in.clone();
//     for i in 0..N_LIMBS {
//         let left = builder.add(&x.limbs[i], &y.limbs[i]);
//         let left = builder.add(&left, &carry);

//         let right = builder.mul(carry_out, two_to_120);
//         let right = builder.add(&right, &result.limbs[i]);

//         builder.assert_is_equal(left, right);

//         if i < N_LIMBS - 1 {
//             carry = tmp_carries[i].clone();
//         }
//     }
// }
