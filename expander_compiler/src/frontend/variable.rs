use std::ops::{Add, Mul, Neg, Sub};

pub trait Variable:
    Copy + Clone + Neg<Output = Self> + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self>
{
}
