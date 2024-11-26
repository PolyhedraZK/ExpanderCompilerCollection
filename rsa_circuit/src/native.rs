use rand::Rng;

use crate::constants::{HEX_PER_LIMB, MASK120, MASK60, MASK8, N_LIMBS};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RSAFieldElement {
    // an RSA field element is a 2048 bits integer
    // it is represented as an array of 18 u120 elements, stored each in a u128
    pub data: [u128; N_LIMBS],
}

#[inline]
// a + b + carry_in = sum + carry_out * 2^120
pub fn add_u120_with_carry(a: &u128, b: &u128, carry: &u128) -> (u128, u128) {
    // a, b, carry are all 120 bits integers, so we can simply add them
    let mut sum = *a + *b + *carry;

    let carry = sum >> 120;
    sum = sum & MASK120;

    (sum, carry)
}

#[inline]
pub fn mul_u120_with_carry(a: &u128, b: &u128, carry: &u128) -> (u128, u128) {
    let a_lo = a & MASK60;
    let a_hi = a >> 60;
    let b_lo = b & MASK60;
    let b_hi = b >> 60;
    let c_lo = *carry & MASK60;
    let c_hi = *carry >> 60;

    let tmp_0 = &a_lo * &b_lo + &c_lo;
    let tmp_1 = &a_lo * &b_hi + &a_hi * &b_lo + c_hi;
    let tmp_2 = &a_hi * &b_hi;

    let tmp_1_lo = tmp_1 & MASK60;
    let tmp_1_hi = tmp_1 >> 60;

    let (res, mut c) = add_u120_with_carry(&tmp_0, &(tmp_1_lo << 60), &0u128);
    c += tmp_1_hi + tmp_2;

    (res, c)
}

impl RSAFieldElement {
    pub fn new(data: [u128; N_LIMBS]) -> Self {
        Self { data }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        let mut data = [0; N_LIMBS];
        rng.fill(&mut data);
        data.iter_mut()
            .take(N_LIMBS - 1)
            .for_each(|x| *x &= MASK120);
        data[N_LIMBS - 1] &= MASK8;
        Self { data }
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for i in 0..N_LIMBS {
            s = (&format!("{:030x}", self.data[i])).to_string() + &s;
        }
        s
    }

    pub fn from_string(s: &str) -> Self {
        let mut data = [0; N_LIMBS];
        for i in 0..N_LIMBS {
            data[N_LIMBS - 1 - i] =
                u128::from_str_radix(&s[i * HEX_PER_LIMB..(i + 1) * HEX_PER_LIMB], 16).unwrap();
        }
        Self { data }
    }

    // assert a + b = result + r * carry
    // a, b, result, modulus are all RSAFieldElement
    pub fn assert_addition(a: &Self, b: &Self, modulus: &Self, carry: &bool, result: &Self) {
        let mut left_result = [0u128; N_LIMBS]; // for a + b
        let mut right_result = result.data.clone(); // for result + r * carry

        // First compute a + b
        let mut c = 0u128;
        for i in 0..N_LIMBS {
            let (sum, new_carry) = add_u120_with_carry(&a.data[i], &b.data[i], &c);
            left_result[i] = sum;
            c = new_carry;
        }

        // If carry is true, add modulus to right_result
        if *carry {
            let mut c = 0u128;
            for i in 0..N_LIMBS {
                let (sum, new_carry) = add_u120_with_carry(&right_result[i], &modulus.data[i], &c);
                right_result[i] = sum;
                c = new_carry;
            }
        }

        // Assert equality
        assert!(
            left_result == right_result,
            "Addition assertion failed\n{:?}\n{:?}",
            left_result,
            right_result
        );
    }

    #[inline]
    // compute a*b without reduction, add the result to res
    fn mul_without_reduction(a: &Self, b: &Self, res: &mut [u128; 2 * N_LIMBS]) {
        for i in 0..N_LIMBS {
            let mut carry = 0u128;
            for j in 0..N_LIMBS {
                if i + j < 2 * N_LIMBS {
                    let (prod, prod_carry) = mul_u120_with_carry(&a.data[i], &b.data[j], &carry);

                    // Add to accumulator at position i+j
                    let mut acc_carry = 0u128;
                    let (sum, new_carry) = add_u120_with_carry(&res[i + j], &prod, &acc_carry);
                    res[i + j] = sum;

                    // Propagate carries
                    carry = prod_carry;
                    acc_carry = new_carry;
                    if acc_carry > 0 {
                        let mut k = 1;
                        while acc_carry > 0 && (i + j + k) < 2 * N_LIMBS {
                            let (new_val, new_carry) =
                                add_u120_with_carry(&res[i + j + k], &acc_carry, &0u128);
                            res[i + j + k] = new_val;
                            acc_carry = new_carry;
                            k += 1;
                        }
                    }
                }
            }
            // Handle final multiplication carry
            if carry > 0 && i + N_LIMBS < 2 * N_LIMBS {
                let mut k = 0;
                while carry > 0 && (i + N_LIMBS + k) < 2 * N_LIMBS {
                    let (new_val, new_carry) =
                        add_u120_with_carry(&res[i + N_LIMBS + k], &carry, &0u128);
                    res[i + N_LIMBS + k] = new_val;
                    carry = new_carry;
                    k += 1;
                }
            }
        }
    }

    // assert a * b = result + r * carry
    // a, b, result, modulus, carry are all RSAFieldElement
    pub fn assert_multiplication(a: &Self, b: &Self, modulus: &Self, carry: &Self, result: &Self) {
        // Two arrays to hold left and right results: a * b and result + r * carry
        let mut left_result = [0u128; 2 * N_LIMBS]; // for a * b
        let mut right_result = [0u128; 2 * N_LIMBS]; // for result + r * carry

        // First compute a * b
        Self::mul_without_reduction(a, b, &mut left_result);
        println!("left_result: {:0x?}", left_result);

        // Now compute result + r * carry
        // First copy result
        for i in 0..N_LIMBS {
            right_result[i] = result.data[i];
        }
        Self::mul_without_reduction(modulus, carry, &mut right_result);
        println!("right_result: {:0x?}", right_result);

        // Assert equality
        assert!(
            left_result == right_result,
            "Multiplication assertion failed"
        );
    }
}
