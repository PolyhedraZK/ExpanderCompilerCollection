use ark_std::test_rng;
use num_bigint::BigUint;
use rand::Rng;

use crate::{
    constants::{MASK120, MASK8, N_LIMBS},
    native::RSAFieldElement,
};
#[test]
fn test_zero() {
    let zero = RSAFieldElement::new([0; N_LIMBS]);
    let big_zero = zero.to_big_uint();
    assert_eq!(big_zero, BigUint::from(0u32));
    assert_eq!(zero, RSAFieldElement::from_big_uint(big_zero));
}

#[test]
fn test_small_numbers() {
    let numbers = vec![1u32, 42, 100, 65535];
    for n in numbers {
        let big_n = BigUint::from(n);
        let field_n = RSAFieldElement::from_big_uint(big_n.clone());
        let recovered = field_n.to_big_uint();
        assert_eq!(recovered, big_n, "Failed for number {}", n);
    }
}

#[test]
fn test_large_numbers() {
    // Test numbers of various bit lengths
    for bits in [64, 120, 240, 500, 1000, 2000].iter() {
        let big_n: BigUint = (BigUint::from(1u32) << *bits) - BigUint::from(1u32);
        let field_n = RSAFieldElement::from_big_uint(big_n.clone());
        let recovered = field_n.to_big_uint();
        assert_eq!(recovered, big_n, "Failed for {}-bit number", bits);
    }
}

#[test]
fn test_max_value() {
    // Create maximum 2048-bit value
    let max_value: BigUint = (BigUint::from(1u32) << 2048) - BigUint::from(1u32);
    let field_max = RSAFieldElement::from_big_uint(max_value.clone());
    let recovered = field_max.to_big_uint();
    assert_eq!(recovered, max_value);
}

#[test]
fn test_limb_boundaries() {
    // Test values near 120-bit boundaries
    for i in 0..N_LIMBS {
        let bits = i * 120;
        if bits >= 2048 {
            continue;
        }

        let value = BigUint::from(1u32) << bits;
        let field_elem = RSAFieldElement::from_big_uint(value.clone());
        let recovered = field_elem.to_big_uint();
        assert_eq!(recovered, value, "Failed at limb boundary {}", i);
    }
}

#[test]
fn test_random_values() {
    let mut rng = test_rng();
    for _ in 0..10 {
        // Generate random value less than 2048 bits
        let bits = rng.gen_range(1..2048);
        let len = (bits + 7) / 8;
        let mut bytes = vec![0u8; len];
        rng.fill(&mut bytes[..]);
        // Mask the top byte to ensure we don't exceed the desired bit length
        if bits % 8 != 0 {
            bytes[len - 1] &= (1 << (bits % 8)) - 1;
        }

        let value = BigUint::from_bytes_le(&bytes);
        let field_elem = RSAFieldElement::from_big_uint(value.clone());
        let recovered = field_elem.to_big_uint();
        assert_eq!(recovered, value);
    }
}

#[test]
#[should_panic(expected = "Input exceeds 2048 bits")]
fn test_too_large() {
    let too_large = BigUint::from(1u32) << 2048;
    RSAFieldElement::from_big_uint(too_large);
}

#[test]
fn test_all_ones_in_limb() {
    // Test with all ones in various limbs
    for i in 0..N_LIMBS {
        let mut data = [0u128; N_LIMBS];
        data[i] = if i == N_LIMBS - 1 { MASK8 } else { MASK120 };
        let field_elem = RSAFieldElement::new(data);
        let big_value = field_elem.to_big_uint();
        let recovered = RSAFieldElement::from_big_uint(big_value);
        assert_eq!(field_elem, recovered, "Failed for all ones in limb {}", i);
    }
}
