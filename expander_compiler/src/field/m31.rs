use rand::RngCore;
use std::{
    fmt,
    io::{Error as IoError, Read, Write},
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::utils::serde::Serde;

use super::{Field, U256};

pub const M31_MOD: u32 = 2147483647;

#[inline]
pub fn mod_reduce_u32(x: u32) -> u32 {
    (x & M31_MOD) + (x >> 31)
}

#[inline]
fn mod_reduce_i64(x: i64) -> i64 {
    (x & M31_MOD as i64) + (x >> 31)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct M31 {
    pub v: u32,
}

impl Field for M31 {
    #[inline(always)]
    fn zero() -> Self {
        M31 { v: 0 }
    }

    #[inline(always)]
    fn one() -> Self {
        M31 { v: 1 }
    }

    fn modulus() -> U256 {
        U256::from(M31_MOD)
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.v == 0 || self.v == M31_MOD
    }

    fn random_unsafe() -> Self {
        let rng = &mut rand::thread_rng();
        rng.next_u32().into()
    }

    fn inv(&self) -> Option<Self> {
        self.try_inverse()
    }
}

// ====================================
// Arithmetics for M31
// ====================================

impl Mul<&M31> for M31 {
    type Output = M31;
    #[inline(always)]
    fn mul(self, rhs: &M31) -> Self::Output {
        let mut vv = self.v as i64 * rhs.v as i64;
        vv = mod_reduce_i64(vv);

        if vv >= M31_MOD as i64 {
            vv -= M31_MOD as i64;
        }
        M31 { v: vv as u32 }
    }
}

impl Mul for M31 {
    type Output = M31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn mul(self, rhs: M31) -> Self::Output {
        self * &rhs
    }
}

impl MulAssign<&M31> for M31 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &M31) {
        *self = *self * rhs;
    }
}

impl MulAssign for M31 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self *= &rhs;
    }
}

impl<T: ::core::borrow::Borrow<M31>> Product<T> for M31 {
    fn product<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, item| acc * item.borrow())
    }
}

impl Add<&M31> for M31 {
    type Output = M31;
    #[inline(always)]
    fn add(self, rhs: &M31) -> Self::Output {
        let mut vv = self.v + rhs.v;
        if vv >= M31_MOD {
            vv -= M31_MOD;
        }
        M31 { v: vv }
    }
}

impl Add for M31 {
    type Output = M31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn add(self, rhs: M31) -> Self::Output {
        self + &rhs
    }
}

impl AddAssign<&M31> for M31 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &M31) {
        *self = *self + rhs;
    }
}

impl AddAssign for M31 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl<T: ::core::borrow::Borrow<M31>> Sum<T> for M31 {
    fn sum<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item.borrow())
    }
}

impl Neg for M31 {
    type Output = M31;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        M31 {
            v: if self.v == 0 { 0 } else { M31_MOD - self.v },
        }
    }
}

impl Sub<&M31> for M31 {
    type Output = M31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: &M31) -> Self::Output {
        self + &(-*rhs)
    }
}

impl Sub for M31 {
    type Output = M31;
    #[inline(always)]
    #[allow(clippy::op_ref)]
    fn sub(self, rhs: M31) -> Self::Output {
        self - &rhs
    }
}

impl SubAssign<&M31> for M31 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &M31) {
        *self = *self - rhs;
    }
}

impl SubAssign for M31 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self -= &rhs;
    }
}

impl From<u32> for M31 {
    #[inline(always)]
    fn from(x: u32) -> Self {
        M31 {
            v: if x < M31_MOD { x } else { x % M31_MOD },
        }
    }
}

impl From<U256> for M31 {
    #[inline(always)]
    fn from(x: U256) -> Self {
        M31 {
            v: (x % M31_MOD).as_u32(),
        }
    }
}

impl Into<U256> for M31 {
    #[inline(always)]
    fn into(self) -> U256 {
        U256::from(mod_reduce_u32(self.v))
    }
}

impl fmt::Display for M31 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.v % M31_MOD)
    }
}

impl M31 {
    #[inline(always)]
    fn exp_power_of_2(&self, power_log: usize) -> Self {
        let mut res = *self;
        for _ in 0..power_log {
            res = res.square();
        }
        res
    }

    /// credit: https://github.com/Plonky3/Plonky3/blob/ed21a5e11cb20effadaab606598ccad4e70e1a3e/mersenne-31/src/mersenne_31.rs#L235
    #[inline(always)]
    fn try_inverse(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }

        // From Fermat's little theorem, in a prime field `F_p`, the inverse of `a` is `a^(p-2)`.
        // Here p-2 = 2147483646 = 1111111111111111111111111111101_2.
        // Uses 30 Squares + 7 Multiplications => 37 Operations total.

        let p1 = *self;
        let p101 = p1.exp_power_of_2(2) * p1;
        let p1111 = p101.square() * p101;
        let p11111111 = p1111.exp_power_of_2(4) * p1111;
        let p111111110000 = p11111111.exp_power_of_2(4);
        let p111111111111 = p111111110000 * p1111;
        let p1111111111111111 = p111111110000.exp_power_of_2(4) * p11111111;
        let p1111111111111111111111111111 = p1111111111111111.exp_power_of_2(12) * p111111111111;
        let p1111111111111111111111111111101 =
            p1111111111111111111111111111.exp_power_of_2(3) * p101;
        Some(p1111111111111111111111111111101)
    }
}

// Serde
impl Serde for M31 {
    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<(), IoError> {
        let extra_buf = [0u8; 28];
        writer.write_all(&self.v.to_le_bytes())?;
        writer.write_all(&extra_buf)
    }

    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self, IoError> {
        let mut u = [0u8; 4];
        let mut extra_buf = [0u8; 28];
        reader.read_exact(&mut u)?;
        reader.read_exact(&mut extra_buf)?;
        for x in extra_buf.iter() {
            if *x != 0 {
                return Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "extra bytes in M31",
                ));
            }
        }
        Ok(u32::from_le_bytes(u).into())
    }
}
