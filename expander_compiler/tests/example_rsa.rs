use ark_std::test_rng;
use halo2curves::bn256::Fr;

const N_LIMBS: usize = 18;
const MASK120: u128 = (1 << 120) - 1;
const MASK60: u128 = (1 << 60) - 1;
const MASK8: u128 = (1 << 8) - 1;
const HEX_PER_LIMB: usize = 30;
const BN_TWO_TO_120: Fr = Fr::from_raw([0, 1 << 56, 0, 0]);

mod native {
    use super::*;
    use rand::Rng;

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
                    let (sum, new_carry) =
                        add_u120_with_carry(&right_result[i], &modulus.data[i], &c);
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
                        let (prod, prod_carry) =
                            mul_u120_with_carry(&a.data[i], &b.data[j], &carry);

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
        pub fn assert_multiplication(
            a: &Self,
            b: &Self,
            modulus: &Self,
            carry: &Self,
            result: &Self,
        ) {
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

    #[test]
    fn test_rsa_field_serial() {
        let mut rng = test_rng();
        let a = RSAFieldElement::random(&mut rng);
        let a_str = a.to_string();
        println!("{:?}", a_str);

        let a2 = RSAFieldElement::from_string(&a_str);
        assert_eq!(a, a2);

        for _ in 0..100 {
            let a = RSAFieldElement::random(&mut rng);
            let a_str = a.to_string();
            let a2 = RSAFieldElement::from_string(&a_str);
            assert_eq!(a, a2);
        }
    }

    #[test]
    fn test_u120_add() {
        let a = MASK120;
        let b = 1;
        let carry = 0;
        let (sum, carry) = native::add_u120_with_carry(&a, &b, &carry);

        assert_eq!(sum, 0);
        assert_eq!(carry, 1);
    }

    #[test]
    fn test_u120_mul() {
        let a = MASK120;
        let b = 8;
        let carry = 0;
        let (sum, carry) = native::mul_u120_with_carry(&a, &b, &carry);

        assert_eq!(sum, 0xfffffffffffffffffffffffffffff8);
        assert_eq!(carry, 7);

        let a = MASK120;
        let b = MASK120 - 1;
        let carry = a;
        let (sum, carry) = native::mul_u120_with_carry(&a, &b, &carry);

        assert_eq!(sum, 1);
        assert_eq!(carry, 0xfffffffffffffffffffffffffffffe);
    }

    #[test]
    fn test_assert_rsa_addition() {
        let mut r = RSAFieldElement::new([MASK120; N_LIMBS]);
        r.data[N_LIMBS - 1] = MASK8;

        {
            let a = RSAFieldElement::new([1u128; N_LIMBS]);
            let b = RSAFieldElement::new([2u128; N_LIMBS]);
            let result = RSAFieldElement::new([3u128; N_LIMBS]);
            RSAFieldElement::assert_addition(&a, &b, &r, &false, &result);
            println!("case 1 passed");
        }

        {
            let mut a = RSAFieldElement::new([MASK120 - 1; N_LIMBS]);
            a.data[N_LIMBS - 1] = MASK8 - 1;
            let b = RSAFieldElement::new([1u128; N_LIMBS]);
            let result = RSAFieldElement::new([0u128; N_LIMBS]);
            println!("a: {:?}", a.to_string());
            println!("b: {:?}", b.to_string());
            println!("r: {:?}", r.to_string());
            println!("result: {:?}", result.to_string());
            RSAFieldElement::assert_addition(&a, &b, &r, &true, &result);
            println!("case 2 passed");
        }

        {
            let mut a = RSAFieldElement::new([MASK120 - 1; N_LIMBS]);
            a.data[N_LIMBS - 1] = MASK8 - 1;
            let b = RSAFieldElement::new([2u128; N_LIMBS]);
            let result =  RSAFieldElement::from_string("000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001000000000000000000000000000001");

            println!("a: {:?}", a.to_string());
            println!("b: {:?}", b.to_string());
            println!("r: {:?}", r.to_string());
            println!("result: {:?}", result.to_string());
            RSAFieldElement::assert_addition(&a, &b, &r, &true, &result);
            println!("case 3 passed");
        }
    }
    #[test]
    fn test_assert_rsa_multiplication() {
        let mut r = RSAFieldElement::new([MASK120; N_LIMBS]);
        r.data[N_LIMBS - 1] = MASK8;

        {
            let a = RSAFieldElement::new([1u128; N_LIMBS]);
            let b = RSAFieldElement::new([2u128; N_LIMBS]);

            let carry =    RSAFieldElement::from_string("0000000000000000000000000000000200000000000000000000000000000400000000000000000000000000000600000000000000000000000000000800000000000000000000000000000a00000000000000000000000000000c00000000000000000000000000000e00000000000000000000000000001000000000000000000000000000001200000000000000000000000000001400000000000000000000000000001600000000000000000000000000001800000000000000000000000000001a00000000000000000000000000001c00000000000000000000000000001e0000000000000000000000000000200000000000000000000000000000220000000000000000000000000000");
            let result =  RSAFieldElement::from_string("00000000000000000000000000002402000000000000000000000000002204000000000000000000000000002006000000000000000000000000001e08000000000000000000000000001c0a000000000000000000000000001a0c00000000000000000000000000180e000000000000000000000000001610000000000000000000000000001412000000000000000000000000001214000000000000000000000000001016000000000000000000000000000e18000000000000000000000000000c1a000000000000000000000000000a1c00000000000000000000000000081e0000000000000000000000000006200000000000000000000000000004220000000000000000000000000002");

            println!("a: {:?}", a.to_string());
            println!("b: {:?}", b.to_string());
            println!("r: {:?}", r.to_string());
            println!("carry: {:?}", result.to_string());
            println!("result: {:?}", carry.to_string());
            RSAFieldElement::assert_multiplication(&a, &b, &r, &carry, &result);
            println!("case 1 passed");
        }

        {
            let mut a = RSAFieldElement::new([MASK120 - 1; N_LIMBS]);
            a.data[N_LIMBS - 1] = MASK8 - 1;
            let b = RSAFieldElement::new([2u128; N_LIMBS]);

            let carry =    RSAFieldElement::from_string("000000000000000000000000000001fe0000000000000000000000000001fc0000000000000000000000000001fa0000000000000000000000000001f80000000000000000000000000001f60000000000000000000000000001f40000000000000000000000000001f20000000000000000000000000001f00000000000000000000000000001ee0000000000000000000000000001ec0000000000000000000000000001ea0000000000000000000000000001e80000000000000000000000000001e60000000000000000000000000001e40000000000000000000000000001e20000000000000000000000000001e00000000000000000000000000001de0000000000000000000000000001");
            let result =   RSAFieldElement::from_string("0000000000000000000000000000dbfdffffffffffffffffffffffffffddfbffffffffffffffffffffffffffdff9ffffffffffffffffffffffffffe1f7ffffffffffffffffffffffffffe3f5ffffffffffffffffffffffffffe5f3ffffffffffffffffffffffffffe7f1ffffffffffffffffffffffffffe9efffffffffffffffffffffffffffebedffffffffffffffffffffffffffedebffffffffffffffffffffffffffefe9fffffffffffffffffffffffffff1e7fffffffffffffffffffffffffff3e5fffffffffffffffffffffffffff5e3fffffffffffffffffffffffffff7e1fffffffffffffffffffffffffff9dffffffffffffffffffffffffffffbddfffffffffffffffffffffffffffd");

            println!("a: {:?}", a.to_string());
            println!("b: {:?}", b.to_string());
            println!("r: {:?}", r.to_string());
            println!("carry: {:?}", result.to_string());
            println!("result: {:?}", carry.to_string());
            RSAFieldElement::assert_multiplication(&a, &b, &r, &carry, &result);
            println!("case 1 passed");
        }
    }
}

mod add_circuit {
    use expander_compiler::frontend::*;
    use expander_compiler::{
        declare_circuit,
        frontend::{BN254Config, BasicAPI, Define, Variable, API},
    };
    use halo2curves::{bn256::Bn256, bn256::Fr, pairing::Engine};

    use super::*;

    declare_circuit!(AddCircuit {
        x: [Variable; N_LIMBS],
        y: [Variable; N_LIMBS],
        r: [Variable; N_LIMBS],
        carry_ins: [Variable; N_LIMBS],
        carry_outs: [Variable; N_LIMBS],
        result: [Variable; N_LIMBS],
    });

    #[inline]
    // a + b + carry_in = result + carry_out * 2^120
    fn assert_add_120_with_carry(
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
    fn assert_mul_120_with_carry(
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

    impl Define<BN254Config> for AddCircuit<Variable> {
        fn define(&self, builder: &mut API<BN254Config>) {
            for i in 0..N_LIMBS {
                assert_add_120_with_carry(
                    &self.x[i],
                    &self.y[i],
                    &self.carry_ins[i],
                    &self.result[i],
                    &self.carry_outs[i],
                    builder,
                );
            }
        }
    }

    // Helper function to create circuit instance with given inputs
    fn create_circuit(
        x: [u64; N_LIMBS],
        y: [u64; N_LIMBS],
        r: [u64; N_LIMBS],
        carry_ins: [u64; N_LIMBS],
        carry_outs: [u64; N_LIMBS],
        result: [u64; N_LIMBS],
    ) -> AddCircuit<Fr> {
        AddCircuit {
            x: x.map(|v| Fr::from(v)),
            y: y.map(|v| Fr::from(v)),
            r: r.map(|v| Fr::from(v)),
            carry_ins: carry_ins.map(|v| Fr::from(v)),
            carry_outs: carry_outs.map(|v| Fr::from(v)),
            result: result.map(|v| Fr::from(v)),
        }
    }

    #[test]
    fn test_rsa_circuit_120_addition() {
        let compile_result = compile(&AddCircuit::default()).unwrap();

        {
            // Test case: Simple addition without carries
            let mut x = [0u64; N_LIMBS];
            x[0] = 50;

            let mut y = [0u64; N_LIMBS];
            y[0] = 30;

            let mut result = [0u64; N_LIMBS];
            result[0] = 80;

            let carry_ins = [0u64; N_LIMBS];
            let carry_outs = [0u64; N_LIMBS];

            let r = [0u64; N_LIMBS];

            let assignment = create_circuit(x, y, r, carry_ins, carry_outs, result);
            let witness = compile_result
                .witness_solver
                .solve_witness(&assignment)
                .unwrap();

            let output = compile_result.layered_circuit.run(&witness);
            assert_eq!(output, vec![true]);
        }
        {
            // Test case: negative case
            let mut x = [0u64; N_LIMBS];
            x[0] = 50;

            let mut y = [0u64; N_LIMBS];
            y[0] = 40;

            let mut result = [0u64; N_LIMBS];
            result[0] = 80;

            let carry_ins = [0u64; N_LIMBS];
            let carry_outs = [0u64; N_LIMBS];

            let r = [0u64; N_LIMBS];

            let assignment = create_circuit(x, y, r, carry_ins, carry_outs, result);
            let witness = compile_result
                .witness_solver
                .solve_witness(&assignment)
                .unwrap();

            let output = compile_result.layered_circuit.run(&witness);
            assert_eq!(output, vec![false]);
        }
    }
}
