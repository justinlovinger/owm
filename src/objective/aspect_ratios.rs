use std::iter::repeat;

use derive_more::Display;
use num_traits::bounds::LowerBounded;

use crate::{
    derive::{
        derive_from_str_from_try_into, derive_new_from_lower_bounded_float,
        derive_try_from_from_new,
    },
    Rect, Size,
};

pub struct MaintainAspectRatios {
    ratios: Vec<AspectRatio>,
    worst_case: f64,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, PartialOrd)]
pub struct AspectRatio(f64);

impl LowerBounded for AspectRatio {
    fn min_value() -> Self {
        Self(f64::EPSILON)
    }
}

derive_new_from_lower_bounded_float!(AspectRatio(f64));
derive_try_from_from_new!(AspectRatio(f64));
derive_from_str_from_try_into!(AspectRatio(f64));

impl MaintainAspectRatios {
    pub fn new(ratios: Vec<AspectRatio>, max_size: Size, count: usize) -> Self {
        // This assumes rectangles cannot have 0 width or height.
        let worst_case = if count > 0 && !ratios.is_empty() {
            ratios
                .iter()
                .chain(repeat(ratios.last().unwrap()))
                .map(|ratio| {
                    (abs_ratio(max_size.width.get() as f64 / ratio.0))
                        .max(abs_ratio((1.0 / max_size.height.get() as f64) / ratio.0))
                        - 1.0
                })
                .take(count)
                .sum()
        } else {
            0.0
        };
        Self { ratios, worst_case }
    }

    pub fn evaluate(&self, rects: &[Rect]) -> f64 {
        if self.worst_case == 0.0 {
            0.0
        } else {
            rects
                .iter()
                .zip(
                    self.ratios
                        .iter()
                        .chain(repeat(self.ratios.last().unwrap()))
                        .copied(),
                )
                .map(|(x, ratio)| {
                    abs_ratio((x.size.width.get() as f64 / x.size.height.get() as f64) / ratio.0)
                        - 1.0
                })
                .sum::<f64>()
                / self.worst_case
        }
    }
}

fn abs_ratio(x: f64) -> f64 {
    if x < 1.0 {
        1.0 / x
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{prop::collection::vec, *};
    use test_strategy::proptest;

    use crate::testing::{ContainedRects, NumRectsRange};

    use super::*;

    #[proptest]
    fn maintain_aspect_ratios_returns_values_in_range_0_1(
        #[strategy(vec(f64::EPSILON..=100.0, 0..=16))] ratios: Vec<f64>,
        #[strategy(ContainedRects::arbitrary_with(NumRectsRange(0, 16)))] x: ContainedRects,
    ) {
        prop_assert!((0.0..=1.0).contains(
            &MaintainAspectRatios::new(
                ratios
                    .into_iter()
                    .map(|x| AspectRatio::new(x).unwrap())
                    .collect(),
                x.container,
                x.rects.len()
            )
            .evaluate(&x.rects)
        ))
    }

    #[test]
    fn maintain_aspect_ratios_returns_1_for_worst_case() {
        let max_size = Size::new_checked(10, 10);
        let rects = [
            Rect::new_checked(0, 0, 1, 10),
            Rect::new_checked(0, 0, 10, 1),
        ];
        assert_eq!(
            MaintainAspectRatios::new(
                vec![AspectRatio(2.0), AspectRatio(0.5)],
                max_size,
                rects.len()
            )
            .evaluate(&rects),
            1.0
        )
    }

    #[test]
    fn maintain_aspect_ratios_returns_0_for_best_case() {
        let max_size = Size::new_checked(10, 10);
        let rects = [
            Rect::new_checked(0, 0, 10, 5),
            Rect::new_checked(0, 0, 5, 10),
        ];
        assert_eq!(
            MaintainAspectRatios::new(
                vec![AspectRatio(2.0), AspectRatio(0.5)],
                max_size,
                rects.len()
            )
            .evaluate(&rects),
            0.0
        )
    }
}
