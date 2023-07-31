#![allow(dead_code)]

use ndarray::{prelude::*, Data, RawData, RemoveAxis};
use num_traits::{pow, One, Zero};
use std::ops::{Add, Div, Mul, RangeInclusive, Sub};

/// Reduce innermost axis
/// to numbers within corresponding ranges.
/// Leftmost is least significant.
///
/// # Examples
///
/// ```ignore
/// use ndarray::{arr3, prelude::*};
/// use optimal_binary::reversed_bits_to_fracs;
///
/// // It uses the corresponding range for each second to innermost index:
/// assert_eq!(
///     reversed_bits_to_fracs(&[0.0..=3.0, 1.0..=4.0, 2.0..=5.0], arr2(&[[],
///                                                                       [],
///                                                                       []])),
///     arr1(&[0., 1., 2.])
/// );
/// assert_eq!(
///     reversed_bits_to_fracs(&[0.0..=3.0, 1.0..=4.0, 2.0..=5.0], arr2(&[[false, false],
///                                                                       [true,  false],
///                                                                       [false,  true]])),
///     arr1(&[0., 2., 4.])
/// );
/// assert_eq!(
///     reversed_bits_to_fracs(&[0.0..=3.0, 1.0..=4.0, 2.0..=5.0], arr3(&[[[false, false],
///                                                                        [true,  false],
///                                                                        [false,  true]],
///                                                                       [[true,   true],
///                                                                        [false,  true],
///                                                                        [true,  false]]])),
///     arr2(&[[0., 2., 4.], [3., 3., 3.]])
/// );
/// ```
pub fn reversed_bits_to_fracs<'a, I, S, D, T>(
    ranges: I,
    bss: ArrayBase<S, D>,
) -> Array<T, D::Smaller>
where
    D::Smaller: RemoveAxis,
    I: IntoIterator<Item = RangeInclusive<T>>,
    S: RawData<Elem = bool> + Data,
    D: Dimension + RemoveAxis,
    T: 'a
        + Copy
        + Zero
        + One
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>,
{
    let ranges = ranges.into_iter().map(|x| x.into_inner());
    // Without `ndarray` `zip_axis`,
    // we require a lot of redundant code.
    // Ideally,
    // we would `zip_axis` with `ranges`
    // and `reversed_bits_to_frac` each slice.
    let outermost_axis = Axis(bss.ndim() - 1);
    let num_bits = bss.len_of(outermost_axis);
    if num_bits == 0 {
        // This requires an unnecessary additional allocation,
        // but as of 2022-10-31,
        // `ndarray` cannot `zip_axis`
        // like it can `map_axis`.
        let mut xss = bss.map_axis(outermost_axis, |_| T::zero()).reversed_axes();
        xss.outer_iter_mut()
            .zip(ranges)
            .for_each(|(mut xs, (start, _))| xs.map_inplace(|x| *x = start));
        xss.reversed_axes()
    } else {
        let two = T::one() + T::one();
        let max_value = pow(two, num_bits) - T::one();
        let mut xss = reversed_bits_to_int(bss).reversed_axes();
        xss.outer_iter_mut()
            .zip(ranges)
            .for_each(|(mut xs, (start, end))| {
                let a = (end - start) / max_value;
                xs.map_inplace(|x| *x = a * *x + start)
            });
        xss.reversed_axes()
    }
}

/// Reduce innermost axis
/// to numbers within range.
/// Leftmost is least significant.
///
/// # Examples
///
/// ```ignore
/// use ndarray::{arr3, prelude::*};
/// use optimal_binary::reversed_bits_to_frac;
///
/// // It returns lower bound for empty arrays:
/// assert_eq!(reversed_bits_to_frac(1.0..=2.0, arr1(&[])), arr0(1.));
/// assert_eq!(
///     reversed_bits_to_frac(2.0..=3.0, arr2(&[[], [], []])),
///     arr1(&[2., 2., 2.])
/// );
///
/// // It returns lower bound when all bits are false:
/// assert_eq!(reversed_bits_to_frac(0.0..=1.0, arr1(&[false])), arr0(0.));
/// assert_eq!(reversed_bits_to_frac(1.0..=2.0, arr1(&[false, false])), arr0(1.));
///
/// // It returns upper bound when all bits are true:
/// assert_eq!(reversed_bits_to_frac(0.0..=1.0, arr1(&[true])), arr0(1.));
/// assert_eq!(reversed_bits_to_frac(1.0..=2.0, arr1(&[true, true])), arr0(2.));
///
/// // It returns a number between lower and upper bound when some bits are true:
/// assert_eq!(reversed_bits_to_frac(1.0..=4.0, arr1(&[true, false])), arr0(2.));
/// assert_eq!(reversed_bits_to_frac(1.0..=4.0, arr1(&[false, true])), arr0(3.));
///
/// // It converts each row of a matrix to a number:
/// assert_eq!(
///     reversed_bits_to_frac(0.0..=3.0, arr2(&[[false, false],
///                                             [true,  false],
///                                             [false,  true]])),
///     arr1(&[0., 1., 2.])
/// );
///
/// // It converts the innermost axis of a tensor to numbers:
/// assert_eq!(
///     reversed_bits_to_frac(0.0..=3.0, arr3(&[[[false, false],
///                                              [true,  false],
///                                              [false,  true]],
///                                             [[true,   true],
///                                              [false,  true],
///                                              [true,  false]]])),
///     arr2(&[[0., 1., 2.], [3., 2., 1.]])
/// );
/// ```
pub fn reversed_bits_to_frac<T, S, D>(
    range: RangeInclusive<T>,
    bss: ArrayBase<S, D>,
) -> Array<T, D::Smaller>
where
    T: Copy + Zero + One + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
    S: RawData<Elem = bool> + Data,
    D: Dimension + RemoveAxis,
{
    let outermost_axis = Axis(bss.ndim() - 1);
    let num_bits = bss.len_of(outermost_axis);
    if num_bits == 0 {
        bss.map_axis(outermost_axis, |_| *range.start())
    } else {
        let two = T::one() + T::one();
        let a = (*range.end() - *range.start()) / (pow(two, num_bits) - T::one());
        reversed_bits_to_int(bss).map(|x| a * *x + *range.start())
    }
}

/// Reduce innermost dimension
/// to base 10 integer representations of bits.
/// Leftmost is least significant.
///
/// # Examples
///
/// ```ignore
/// use ndarray::prelude::*;
/// use optimal_binary::reversed_bits_to_int;
///
/// // It returns 0 when empty:
/// assert_eq!(reversed_bits_to_int::<_, _, u8>(arr1(&[])), arr0(0));
///
/// // It returns the base 10 integer represented by binary bits:
/// assert_eq!(reversed_bits_to_int::<_, _, u8>(arr1(&[false])), arr0(0));
/// assert_eq!(reversed_bits_to_int::<_, _, u8>(arr1(&[false, false])), arr0(0));
/// assert_eq!(reversed_bits_to_int::<_, _, u8>(arr1(&[false, false, false])), arr0(0));
/// assert_eq!(reversed_bits_to_int::<_, _, u8>(arr1(&[true])), arr0(1));
/// assert_eq!(reversed_bits_to_int::<_, _, u8>(arr1(&[true, true])), arr0(3));
/// assert_eq!(reversed_bits_to_int::<_, _, u8>(arr1(&[true, true, true])), arr0(7));
///
/// // It treats leftmost as least significant:
/// assert_eq!(reversed_bits_to_int::<_, _, u8>(arr1(&[false, true])), arr0(2));
/// assert_eq!(reversed_bits_to_int::<_, _, u8>(arr1(&[false, false, true])), arr0(4));
///
/// // It converts each row of a matrix to a number:
/// assert_eq!(
///     reversed_bits_to_int::<_, _, u8>(arr2(&[[false, false],
///                                             [true, false],
///                                             [false, true]])),
///     arr1(&[0, 1, 2])
/// );
/// ```
pub fn reversed_bits_to_int<S, D, T>(bss: ArrayBase<S, D>) -> Array<T, D::Smaller>
where
    S: RawData<Elem = bool> + Data,
    D: Dimension + RemoveAxis,
    T: Copy + Zero + One + Add<Output = T> + Mul<Output = T>,
{
    let two = T::one() + T::one();
    bss.map_axis(Axis(bss.ndim() - 1), |bs| {
        bs.fold((T::zero(), T::one()), |(acc, a), b| {
            (if *b { acc + a } else { acc }, two * a)
        })
        .0
    })
}
