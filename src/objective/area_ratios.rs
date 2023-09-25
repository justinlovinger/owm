use std::{
    iter::{once, repeat},
    num::NonZeroUsize,
    ops::Mul,
};

use derive_more::Display;
use itertools::Itertools;
use num_traits::bounds::LowerBounded;

use crate::{
    derive::{
        derive_from_str_from_try_into, derive_new_from_lower_bounded_float,
        derive_try_from_from_new,
    },
    Rect, Size,
};

pub struct MaintainAreaRatios {
    ratios: Vec<AreaRatio>,
    worst_case: f64,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, PartialOrd)]
pub struct AreaRatio(f64);

impl LowerBounded for AreaRatio {
    fn min_value() -> Self {
        Self(1.0)
    }
}

derive_new_from_lower_bounded_float!(AreaRatio(f64));
derive_try_from_from_new!(AreaRatio(f64));
derive_from_str_from_try_into!(AreaRatio(f64));

impl Mul<f64> for AreaRatio {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        self.0 * rhs
    }
}

impl MaintainAreaRatios {
    pub fn new(ratios: Vec<AreaRatio>, max_size: Size, count: usize) -> Self {
        let worst_case = if !ratios.is_empty() && count > 1 {
            Self::_evaluate(
                ratios
                    .iter()
                    .sorted_unstable_by(|x, y| y.partial_cmp(x).unwrap())
                    .chain(repeat(ratios.last().unwrap()))
                    .copied(),
                once(unsafe { NonZeroUsize::new_unchecked(1) })
                    .chain(repeat(max_size.area()))
                    .take(count),
            )
        } else {
            0.0
        };
        Self { ratios, worst_case }
    }

    pub fn evaluate(&self, rects: &[Rect]) -> f64 {
        if self.worst_case == 0.0 {
            0.0
        } else {
            Self::_evaluate(
                self.ratios
                    .iter()
                    .chain(repeat(self.ratios.last().unwrap()))
                    .copied(),
                rects.iter().map(|x| x.area()),
            ) / self.worst_case
        }
    }

    fn _evaluate(
        ratios: impl Iterator<Item = AreaRatio>,
        areas: impl Iterator<Item = NonZeroUsize>,
    ) -> f64 {
        areas
            .map(|x| x.get() as f64)
            .tuple_windows()
            .zip(ratios)
            // Use `.abs()` instead of `.max(0.0)`
            // to encourage later to grow
            // when possible.
            // Otherwise,
            // the last rectangle can always be small
            // with no penalty.
            .map(|((x, y), ratio)| (ratio * y - x).abs())
            .sum::<f64>()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{prop::collection::vec, *};
    use test_strategy::proptest;

    use crate::testing::{ContainedRects, NumRectsRange};

    use super::*;

    #[proptest]
    fn maintain_area_ratios_returns_values_in_range_0_1(
        #[strategy(vec(1.0..=100.0, 0..=16))] ratios: Vec<f64>,
        #[strategy(ContainedRects::arbitrary_with(NumRectsRange(0, 16)))] x: ContainedRects,
    ) {
        prop_assert!((0.0..=1.0).contains(
            &MaintainAreaRatios::new(
                ratios
                    .into_iter()
                    .map(|x| AreaRatio::new(x).unwrap())
                    .collect(),
                x.container,
                x.rects.len()
            )
            .evaluate(&x.rects)
        ))
    }

    #[test]
    fn maintain_area_ratios_returns_1_for_worst_case() {
        // Note,
        // what exactly counts as the worst case
        // is uncertain.
        // We could define the worst case
        // as the reverse of the best case.
        // However,
        // then the middle rectangle has a good area
        // for its position.
        let max_size = Size::new_checked(10, 10);
        let rects = [
            Rect::new_checked(0, 0, 1, 1),
            Rect::new_checked(0, 0, 10, 10),
            Rect::new_checked(0, 0, 10, 10),
        ];
        assert_eq!(
            MaintainAreaRatios::new(vec![AreaRatio(2.0)], max_size, rects.len()).evaluate(&rects),
            1.0
        )
    }

    #[test]
    fn maintain_area_ratios_returns_0_for_best_case() {
        let max_size = Size::new_checked(10, 10);
        let rects = [
            Rect::new_checked(0, 0, 10, 10),
            Rect::new_checked(0, 0, 10, 5),
            Rect::new_checked(0, 0, 5, 5),
        ];
        assert_eq!(
            MaintainAreaRatios::new(vec![AreaRatio(2.0)], max_size, rects.len()).evaluate(&rects),
            0.0
        )
    }
}
