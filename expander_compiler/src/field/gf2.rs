use rand::RngCore;
use std::{
    fmt,
    io::{Error as IoError, Read, Write},
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::utils::serde::Serde;

use super::{Field, U256};

pub const MOD: u32 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GF2 {
    pub v: bool,
}

impl Field for GF2 {
    #[inline(always)]
    fn zero() -> Self {
        GF2 { v: false }
    }

    #[inline(always)]
    fn one() -> Self {
        GF2 { v: true }
    }

    fn modulus() -> U256 {
        U256::from(MOD)
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        !self.v
    }

    fn random_unsafe() -> Self {
        let rng = &mut rand::thread_rng();
        GF2 {
            v: (rng.next_u32() & 1) == 1,
        }
    }

    fn inv(&self) -> Option<Self> {
        match self.v {
            true => Some(Self::one()),
            false => None,
        }
    }
}

// ====================================
// Arithmetics for GF2
// ====================================

impl Mul<&GF2> for GF2 {
    type Output = GF2;
    #[inline(always)]
    fn mul(self, rhs: &GF2) -> Self::Output {
        GF2 { v: self.v & rhs.v }
    }
}

impl Mul for GF2 {
    type Output = GF2;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn mul(self, rhs: GF2) -> Self::Output {
        self * &rhs
    }
}

impl MulAssign<&GF2> for GF2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &GF2) {
        *self = *self * rhs;
    }
}

impl MulAssign for GF2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self *= &rhs;
    }
}

impl<T: ::core::borrow::Borrow<GF2>> Product<T> for GF2 {
    fn product<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, item| acc * item.borrow())
    }
}

impl Add<&GF2> for GF2 {
    type Output = GF2;
    #[inline(always)]
    fn add(self, rhs: &GF2) -> Self::Output {
        GF2 { v: self.v ^ rhs.v }
    }
}

impl Add for GF2 {
    type Output = GF2;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn add(self, rhs: GF2) -> Self::Output {
        self + &rhs
    }
}

impl AddAssign<&GF2> for GF2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &GF2) {
        *self = *self + rhs;
    }
}

impl AddAssign for GF2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl<T: ::core::borrow::Borrow<GF2>> Sum<T> for GF2 {
    fn sum<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item.borrow())
    }
}

impl Neg for GF2 {
    type Output = GF2;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        self
    }
}

impl Sub<&GF2> for GF2 {
    type Output = GF2;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: &GF2) -> Self::Output {
        self + &(-*rhs)
    }
}

impl Sub for GF2 {
    type Output = GF2;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: GF2) -> Self::Output {
        self - &rhs
    }
}

impl SubAssign<&GF2> for GF2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &GF2) {
        *self = *self - rhs;
    }
}

impl SubAssign for GF2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs;
    }
}

impl From<u32> for GF2 {
    #[inline(always)]
    fn from(x: u32) -> Self {
        GF2 { v: (x & 1) == 1 }
    }
}

impl From<U256> for GF2 {
    #[inline(always)]
    fn from(x: U256) -> Self {
        GF2 {
            v: (x.as_u32() & 1) == 1,
        }
    }
}

impl Into<U256> for GF2 {
    #[inline(always)]
    fn into(self) -> U256 {
        U256::from(self.v as u32)
    }
}

impl fmt::Display for GF2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.v as u32)
    }
}

// Serde
impl Serde for GF2 {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        let mut buf = [0u8; 32];
        buf[0] = self.v as u8;
        writer.write_all(&buf)
    }

    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        for x in buf.iter().skip(1) {
            if *x != 0 {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "extra bytes in GF2",
                ));
            }
        }
        Ok(GF2 {
            v: (buf[0] & 1) == 1,
        })
    }
}
