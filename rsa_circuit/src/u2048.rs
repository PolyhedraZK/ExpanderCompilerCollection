use expander_compiler::frontend::{extra::UnconstrainedAPI, BN254Config, BasicAPI, Variable, API};

use crate::{constants::N_LIMBS, u120};

#[derive(Debug, Clone, Copy)]
pub struct U2048Variable {
    pub limbs: [Variable; N_LIMBS],
}

impl U2048Variable {
    #[inline]
    pub fn from_raw(limbs: [Variable; N_LIMBS]) -> Self {
        Self { limbs }
    }

    #[inline]
    // generate a bool variable for the comparison of two U2048 variables
    pub fn unconstrained_greater_eq(
        &self,
        other: &Self,
        builder: &mut API<BN254Config>,
    ) -> Variable {
        // Start from most significant limb (N_LIMBS-1) and work down
        let mut gt_flags = Vec::with_capacity(N_LIMBS);
        let mut eq_flags = Vec::with_capacity(N_LIMBS);

        // Compute comparison flags for all limbs
        for i in (0..N_LIMBS).rev() {
            gt_flags.push(builder.unconstrained_greater(self.limbs[i], other.limbs[i]));
            eq_flags.push(builder.unconstrained_eq(self.limbs[i], other.limbs[i]));
        }

        // Start with the most significant limb comparison
        let mut result = gt_flags[0]; // corresponds to limb N_LIMBS-1
        let mut all_eq_so_far = eq_flags[0];

        // Process remaining limbs from most to least significant
        for i in 1..N_LIMBS {
            // If all previous limbs were equal and current limb is greater
            let curr_greater = builder.unconstrained_bit_and(all_eq_so_far, gt_flags[i]);
            result = builder.unconstrained_bit_or(result, curr_greater);

            // Update equality chain
            all_eq_so_far = builder.unconstrained_bit_and(all_eq_so_far, eq_flags[i]);
        }

        // Result is true if we found a greater limb or if all limbs were equal
        builder.unconstrained_bit_or(result, all_eq_so_far)
    }

    #[inline]
    // add two U2048 variables with mod reductions
    // a + b = result + carry * modulus
    pub fn assert_add(
        x: &U2048Variable,
        y: &U2048Variable,
        result: &U2048Variable,
        carry: &Variable,
        modulus: &U2048Variable,
        two_to_120: &Variable,
        builder: &mut API<BN254Config>,
    ) {
        // First compute raw sum x + y with carries between limbs
        let mut sum = vec![];
        let mut temp_carry = builder.constant(0);
        for i in 0..N_LIMBS {
            let (r, c) = u120::add_u120(&x.limbs[i], &y.limbs[i], &temp_carry, two_to_120, builder);
            temp_carry = c;
            sum.push(r);
        }
        let sum = U2048Variable::from_raw(sum.try_into().unwrap());

        // Verify carry is boolean
        builder.assert_is_bool(*carry);

        // Now verify: sum = result + carry * modulus

        // First compute carry * modulus
        let mut carry_times_modulus = vec![];
        for i in 0..N_LIMBS {
            carry_times_modulus.push(builder.mul(*carry, modulus.limbs[i]));
        }
        let carry_times_modulus = U2048Variable::from_raw(carry_times_modulus.try_into().unwrap());

        // For each limb, verify: sum[i] = result[i] + (carry * modulus)[i]
        let mut temp_carry = builder.constant(0);
        for i in 0..N_LIMBS {
            let (expected, c) = u120::add_u120(
                &result.limbs[i],
                &carry_times_modulus.limbs[i],
                &temp_carry,
                two_to_120,
                builder,
            );
            temp_carry = c;

            // Assert equality for this limb
            builder.assert_is_equal(sum.limbs[i], expected);
        }

        // Final carry should be 0 since all numbers are within range
        builder.assert_is_zero(temp_carry);
    }
}
