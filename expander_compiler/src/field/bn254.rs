use std::{
    fmt,
    io::{Error as IoError, ErrorKind as IoErrorKind, Read, Write},
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use halo2curves::{bn256::Fr, ff::Field as Halo2Field};

use crate::utils::serde::Serde;

use super::{Field, U256};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BN254 {
    v: Fr,
}

const MODULUS: U256 = U256([
    0x43e1f593f0000001,
    0x2833e84879b97091,
    0xb85045b68181585d,
    0x30644e72e131a029,
]);

impl Field for BN254 {
    fn zero() -> Self {
        BN254 { v: Fr::zero() }
    }

    fn is_zero(&self) -> bool {
        self.v == Fr::zero()
    }

    fn one() -> Self {
        BN254 { v: Fr::one() }
    }

    fn random_unsafe() -> Self {
        let mut rng = rand::thread_rng();
        BN254 {
            v: Fr::random(&mut rng),
        }
    }

    fn modulus() -> U256 {
        MODULUS
    }

    fn inv(&self) -> Option<Self> {
        self.v.invert().map(|v| BN254 { v }).into()
    }
}

// ====================================
// Arithmetics for BN254
// ====================================

impl Mul<&BN254> for BN254 {
    type Output = BN254;
    #[inline(always)]
    fn mul(self, rhs: &BN254) -> Self::Output {
        BN254 {
            v: self.v.mul(&rhs.v),
        }
    }
}

impl Mul for BN254 {
    type Output = BN254;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn mul(self, rhs: BN254) -> Self::Output {
        self * &rhs
    }
}

impl MulAssign<&BN254> for BN254 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &BN254) {
        *self = *self * rhs;
    }
}

impl MulAssign for BN254 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self *= &rhs;
    }
}

impl<T: ::core::borrow::Borrow<BN254>> Product<T> for BN254 {
    fn product<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, item| acc * item.borrow())
    }
}

impl Add<&BN254> for BN254 {
    type Output = BN254;
    #[inline(always)]
    fn add(self, rhs: &BN254) -> Self::Output {
        BN254 {
            v: self.v.add(&rhs.v),
        }
    }
}

impl Add for BN254 {
    type Output = BN254;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn add(self, rhs: BN254) -> Self::Output {
        self + &rhs
    }
}

impl AddAssign<&BN254> for BN254 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &BN254) {
        *self = *self + rhs;
    }
}

impl AddAssign for BN254 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl<T: ::core::borrow::Borrow<BN254>> Sum<T> for BN254 {
    fn sum<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item.borrow())
    }
}

impl Neg for BN254 {
    type Output = BN254;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        BN254 { v: self.v.neg() }
    }
}

impl Sub<&BN254> for BN254 {
    type Output = BN254;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: &BN254) -> Self::Output {
        self + &(-*rhs)
    }
}

impl Sub for BN254 {
    type Output = BN254;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: BN254) -> Self::Output {
        self - &rhs
    }
}

impl SubAssign<&BN254> for BN254 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &BN254) {
        *self = *self - rhs;
    }
}

impl SubAssign for BN254 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs;
    }
}

impl From<u32> for BN254 {
    #[inline(always)]
    fn from(x: u32) -> Self {
        BN254 { v: Fr::from(x) }
    }
}

impl From<U256> for BN254 {
    #[inline(always)]
    fn from(x: U256) -> Self {
        let mut b = [0u8; 32];
        (x % BN254::modulus()).to_little_endian(&mut b);
        BN254 {
            v: Fr::from_bytes(&b).unwrap(),
        }
    }
}

impl Into<U256> for BN254 {
    #[inline(always)]
    fn into(self) -> U256 {
        let b = self.v.to_bytes();
        U256::from_little_endian(&b)
    }
}

impl fmt::Display for BN254 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let x: U256 = (*self).into();
        write!(f, "{}", x.to_string())
    }
}

impl Serde for BN254 {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        writer.write_all(self.v.to_bytes().as_ref())
    }

    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let mut buffer = [0u8; 32];
        reader.read_exact(&mut buffer)?;
        let v = Fr::from_bytes(&buffer);
        if v.is_none().into() {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                "invalid bytes for BN254",
            ));
        }
        Ok(BN254 { v: v.unwrap() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bn254_serde() {
        let x = BN254::random_unsafe();
        let mut buffer = Vec::new();
        x.serialize_into(&mut buffer).unwrap();
        let y = BN254::deserialize_from(&buffer[..]).unwrap();
        assert_eq!(x, y);
    }

    #[test]
    fn test_bn254_u256() {
        assert_eq!(
            BN254::modulus().to_string(),
            "21888242871839275222246405745257275088548364400416034343698204186575808495617"
        );
        let x = BN254::random_unsafe();
        let y: U256 = x.into();
        let z: BN254 = y.into();
        assert_eq!(x, z);
        let x = BN254::from(123u32);
        let y: U256 = x.into();
        let z = U256::from(123u32);
        assert_eq!(y, z);
    }
}
