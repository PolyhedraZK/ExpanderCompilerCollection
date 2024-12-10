use ark_std::test_rng;

use crate::{
    constants::{MASK120, MASK8, N_LIMBS},
    native::{self, RSAFieldElement},
};

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
        let result = RSAFieldElement::from_string(
            "000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001\
             000000000000000000000000000001",
        );

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

        let carry = RSAFieldElement::from_string(
            "000000000000000000000000000000\
             020000000000000000000000000000\
             040000000000000000000000000000\
             060000000000000000000000000000\
             080000000000000000000000000000\
             0a0000000000000000000000000000\
             0c0000000000000000000000000000\
             0e0000000000000000000000000000\
             100000000000000000000000000000\
             120000000000000000000000000000\
             140000000000000000000000000000\
             160000000000000000000000000000\
             180000000000000000000000000000\
             1a0000000000000000000000000000\
             1c0000000000000000000000000000\
             1e0000000000000000000000000000\
             200000000000000000000000000000\
             220000000000000000000000000000",
        );
        let result = RSAFieldElement::from_string(
            "000000000000000000000000000024\
             020000000000000000000000000022\
             040000000000000000000000000020\
             06000000000000000000000000001e\
             08000000000000000000000000001c\
             0a000000000000000000000000001a\
             0c0000000000000000000000000018\
             0e0000000000000000000000000016\
             100000000000000000000000000014\
             120000000000000000000000000012\
             140000000000000000000000000010\
             16000000000000000000000000000e\
             18000000000000000000000000000c\
             1a000000000000000000000000000a\
             1c0000000000000000000000000008\
             1e0000000000000000000000000006\
             200000000000000000000000000004\
             220000000000000000000000000002",
        );

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

        let carry = RSAFieldElement::from_string(
            "000000000000000000000000000001\
             fe0000000000000000000000000001\
             fc0000000000000000000000000001\
             fa0000000000000000000000000001\
             f80000000000000000000000000001\
             f60000000000000000000000000001\
             f40000000000000000000000000001\
             f20000000000000000000000000001\
             f00000000000000000000000000001\
             ee0000000000000000000000000001\
             ec0000000000000000000000000001\
             ea0000000000000000000000000001\
             e80000000000000000000000000001\
             e60000000000000000000000000001\
             e40000000000000000000000000001\
             e20000000000000000000000000001\
             e00000000000000000000000000001\
             de0000000000000000000000000001",
        );
        let result = RSAFieldElement::from_string(
            "0000000000000000000000000000db\
             fdffffffffffffffffffffffffffdd\
             fbffffffffffffffffffffffffffdf\
             f9ffffffffffffffffffffffffffe1\
             f7ffffffffffffffffffffffffffe3\
             f5ffffffffffffffffffffffffffe5\
             f3ffffffffffffffffffffffffffe7\
             f1ffffffffffffffffffffffffffe9\
             efffffffffffffffffffffffffffeb\
             edffffffffffffffffffffffffffed\
             ebffffffffffffffffffffffffffef\
             e9fffffffffffffffffffffffffff1\
             e7fffffffffffffffffffffffffff3\
             e5fffffffffffffffffffffffffff5\
             e3fffffffffffffffffffffffffff7\
             e1fffffffffffffffffffffffffff9\
             dffffffffffffffffffffffffffffb\
             ddfffffffffffffffffffffffffffd",
        );

        println!("a: {:?}", a.to_string());
        println!("b: {:?}", b.to_string());
        println!("r: {:?}", r.to_string());
        println!("carry: {:?}", result.to_string());
        println!("result: {:?}", carry.to_string());
        RSAFieldElement::assert_multiplication(&a, &b, &r, &carry, &result);
        println!("case 1 passed");
    }
}
