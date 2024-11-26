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
