use num_bigint::BigInt;

#[derive(Default,Clone, Copy)]
pub struct bls12381_fp {}
impl bls12381_fp {
    pub fn nb_limbs() -> u32 {
        48
    }
    pub fn bits_per_limb() -> u32 {
        8
    }
    pub fn is_prime() -> bool {
        true
    }
    pub fn modulus() -> BigInt {
        let hex_str = "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab";
        BigInt::parse_bytes(hex_str.as_bytes(), 16).unwrap()
    }
}
#[derive(Default,Clone)]
pub struct bls12381_fr {}
impl bls12381_fr {
    pub fn nb_limbs() -> u32 {
        32
    }
    pub fn bits_per_limb() -> u32 {
        8
    }
    pub fn is_prime() -> bool {
        true
    }
    pub fn modulus() -> BigInt {
        let hex_str = "73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001";
        BigInt::parse_bytes(hex_str.as_bytes(), 16).unwrap()
    }
}
pub trait FieldParams {
    fn nb_limbs() -> u32;
    fn bits_per_limb() -> u32;
    fn is_prime() -> bool;
    fn modulus() -> BigInt;
}

impl FieldParams for bls12381_fr {
    fn nb_limbs() -> u32 {
        bls12381_fr::nb_limbs()
    }
    fn bits_per_limb() -> u32 {
        bls12381_fr::bits_per_limb()
    }
    fn is_prime() -> bool {
        bls12381_fr::is_prime()
    }
    fn modulus() -> BigInt {
        bls12381_fr::modulus()
    }
}

impl FieldParams for bls12381_fp {
    fn nb_limbs() -> u32 {
        bls12381_fp::nb_limbs()
    }
    fn bits_per_limb() -> u32 {
        bls12381_fp::bits_per_limb()
    }
    fn is_prime() -> bool {
        bls12381_fp::is_prime()
    }
    fn modulus() -> BigInt {
        bls12381_fp::modulus()
    }
}


