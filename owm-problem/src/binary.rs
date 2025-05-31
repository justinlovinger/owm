#![allow(dead_code)]

use num_traits::{pow, One, Zero};
use std::ops::{Add, Div, Mul, RangeInclusive, Sub};

/// Reduce innermost axis
/// to numbers within range.
/// Leftmost is least significant.
///
/// # Examples
///
/// ```ignore
/// // It returns lower bound for empty arrays:
/// assert_eq!(ToFracLe::new(1.0..=2.0, 0)::decode(vec![]), 1.);
///
/// // It returns lower bound when all bits are false:
/// assert_eq!(ToFracLe::new(0.0..=1.0, 1)::decode(vec![false]), 0.);
/// assert_eq!(ToFracLe::new(1.0..=2.0, 2)::decode(vec![false, false]), 1.);
///
/// // It returns upper bound when all bits are true:
/// assert_eq!(ToFracLe::new(0.0..=1.0, 1)::decode(vec![true]), 1.);
/// assert_eq!(ToFracLe::new(1.0..=2.0, 2)::decode(vec![true, true]), 2.);
///
/// // It returns a number between lower and upper bound when some bits are true:
/// assert_eq!(ToFracLe::new(1.0..=4.0, 2)::decode(vec![true, false]), 2.);
/// assert_eq!(ToFracLe::new(1.0..=4.0, 2)::decode(vec![false, true]), 3.);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToFracLE<T> {
    to_int: ToIntLE<T>,
    start: T,
    a: Option<T>,
}

impl<T> ToFracLE<T> {
    pub fn new(range: RangeInclusive<T>, bits_len: usize) -> Self
    where
        T: Copy + One + Add<Output = T> + Sub<Output = T> + Div<Output = T>,
    {
        let to_int = ToIntLE::new();
        let (start, end) = range.into_inner();
        Self {
            a: if bits_len > 0 {
                Some((end - start) / (pow(to_int.two, bits_len) - T::one()))
            } else {
                None
            },
            start,
            to_int,
        }
    }

    pub fn decode(&self, bits: impl IntoIterator<Item = bool>) -> T
    where
        T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>,
    {
        match self.a {
            Some(a) => a * self.to_int.decode(bits) + self.start,
            None => self.start,
        }
    }
}

/// Reduce to base 10 integer representations of bits.
/// Leftmost is least significant.
///
/// # Examples
///
/// ```ignore
/// // It returns 0 when empty:
/// assert_eq!(ToIntLe::new().decode(vec![]), 0_u8);
///
/// // It returns the base 10 integer represented by binary bits:
/// assert_eq!(ToIntLe::new().decode(vec![false]), 0_u8);
/// assert_eq!(ToIntLe::new().decode(vec![false, false]), 0_u8);
/// assert_eq!(ToIntLe::new().decode(vec![false, false, false]), 0_u8);
/// assert_eq!(ToIntLe::new().decode(vec![true]), 1_u8);
/// assert_eq!(ToIntLe::new().decode(vec![true, true]), 3_u8);
/// assert_eq!(ToIntLe::new().decode(vec![true, true, true]), 7_u8);
///
/// // It treats leftmost as least significant:
/// assert_eq!(ToIntLe::new().decode(vec![false, true]), 2_u8);
/// assert_eq!(ToIntLe::new().decode(vec![false, false, true]), 4_u8);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToIntLE<T> {
    two: T,
}

impl<T> ToIntLE<T> {
    pub fn new() -> Self
    where
        T: One + Add<Output = T>,
    {
        Self {
            two: T::one() + T::one(),
        }
    }

    pub fn decode(&self, bits: impl IntoIterator<Item = bool>) -> T
    where
        T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>,
    {
        bits.into_iter()
            .fold((T::zero(), T::one()), |(acc, a), b| {
                (if b { acc + a } else { acc }, self.two * a)
            })
            .0
    }
}
