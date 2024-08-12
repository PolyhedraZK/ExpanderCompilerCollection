use std::{
    fmt::{self, Debug},
    hash::Hash,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use uint::construct_uint;

use crate::utils::serde::Serde;

pub mod bn254;
pub mod gf2;
pub mod m31;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

/// Field definitions.
pub trait Field:
    Copy
    + Clone
    + Debug
    + Hash
    + Default
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + From<u32>
    + From<U256>
    + Into<U256>
    + Neg<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Sum
    + Product
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> Mul<&'a Self, Output = Self>
    + for<'a> Sum<&'a Self>
    + for<'a> Product<&'a Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + for<'a> MulAssign<&'a Self>
    + fmt::Display
    + Serde
{
    // ====================================
    // constants
    // ====================================
    /// Zero element
    fn zero() -> Self;

    /// Is zero
    fn is_zero(&self) -> bool;

    /// Identity element
    fn one() -> Self;

    /// Modulus
    fn modulus() -> U256;

    // ====================================
    // generators
    // ====================================
    /// create a random element from rng.
    /// test only -- the output may not be uniformly random.
    fn random_unsafe() -> Self;

    // ====================================
    // arithmetics
    // ====================================
    /// Squaring
    #[inline(always)]
    fn square(&self) -> Self {
        *self * *self
    }

    /// find the inverse of the element; return None if not exist
    fn inv(&self) -> Option<Self>;
}
