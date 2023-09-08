use std::{
    iter::{once, repeat},
    num::NonZeroUsize,
    ops::Mul,
};

use derive_more::Display;
use itertools::Itertools;
use num_traits::bounds::LowerBounded;

use crate::{
    derive::*,
    rect::{covered_area, obscured_area, Pos, Rect, Size},
};

pub struct Problem {
    weights: Weights,
    gaps: MinimizeGaps,
    overlap: MinimizeOverlap,
    area_ratios: MaintainAreaRatios,
    aspect_ratios: MaintainAspectRatios,
    adjacent_close: PlaceAdjacentClose,
    reading_order: PlaceInReadingOrder,
    center_main: CenterMain,
}

#[derive(Clone, Copy, Debug)]
pub struct Weights {
    pub gaps_weight: Weight,
    pub overlap_weight: Weight,
    pub area_ratios_weight: Weight,
    pub aspect_ratios_weight: Weight,
    pub adjacent_close_weight: Weight,
    pub reading_order_weight: Weight,
    pub center_main_weight: Weight,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, PartialOrd)]
pub struct Weight(f64);

impl LowerBounded for Weight {
    fn min_value() -> Self {
        Self(0.0)
    }
}

derive_new_from_lower_bounded_float!(Weight(f64));
derive_try_from_from_new!(Weight(f64));
derive_from_str_from_try_into!(Weight(f64));

impl Mul<f64> for Weight {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        self.0 * rhs
    }
}

impl Problem {
    pub fn new(
        weights: Weights,
        area_ratios: Vec<AreaRatio>,
        aspect_ratios: Vec<AspectRatio>,
        max_size: Size,
        container: Size,
        count: usize,
    ) -> Self {
        Self {
            weights,
            gaps: MinimizeGaps::new(container),
            overlap: MinimizeOverlap::new(container, count),
            area_ratios: MaintainAreaRatios::new(area_ratios, max_size, count),
            aspect_ratios: MaintainAspectRatios::new(aspect_ratios, max_size, count),
            adjacent_close: PlaceAdjacentClose::new(container, count),
            reading_order: PlaceInReadingOrder::new(count),
            center_main: CenterMain::new(container),
        }
    }

    pub fn evaluate(&self, rects: &[Rect]) -> f64 {
        (if self.weights.gaps_weight > Weight(0.0) {
            self.weights.gaps_weight * self.gaps.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.overlap_weight > Weight(0.0) {
            self.weights.overlap_weight * self.overlap.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.area_ratios_weight > Weight(0.0) {
            self.weights.area_ratios_weight * self.area_ratios.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.aspect_ratios_weight > Weight(0.0) {
            self.weights.aspect_ratios_weight * self.aspect_ratios.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.adjacent_close_weight > Weight(0.0) {
            self.weights.adjacent_close_weight * self.adjacent_close.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.reading_order_weight > Weight(0.0) {
            self.weights.reading_order_weight * self.reading_order.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.center_main_weight > Weight(0.0) {
            self.weights.center_main_weight * self.center_main.evaluate(rects)
        } else {
            0.0
        })
    }
}

struct MinimizeGaps {
    area: NonZeroUsize,
    worst_case: f64,
}

impl MinimizeGaps {
    fn new(container: Size) -> Self {
        Self {
            area: container.area(),
            worst_case: (container.area().get() - 1) as f64,
        }
    }

    fn evaluate(&self, rects: &[Rect]) -> f64 {
        if rects.is_empty() {
            1.0
        } else {
            // This assumes rectangles do not exceed container bounds.
            // Worst case can theoretically be zero,
            // if `container.area()` is `1`,
            // but this is unrealistic in practice.
            (self.area.get() - covered_area(rects).get()) as f64 / self.worst_case
        }
    }
}

struct MinimizeOverlap {
    worst_case: f64,
}

impl MinimizeOverlap {
    fn new(container: Size, count: usize) -> Self {
        Self {
            worst_case: (count.saturating_sub(1) * container.area().get()) as f64,
        }
    }

    fn evaluate(&self, rects: &[Rect]) -> f64 {
        if rects.len() < 2 {
            0.0
        } else {
            obscured_area(rects) as f64 / self.worst_case
        }
    }
}

struct MaintainAreaRatios {
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
    fn new(ratios: Vec<AreaRatio>, max_size: Size, count: usize) -> Self {
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

    fn evaluate(&self, rects: &[Rect]) -> f64 {
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

struct MaintainAspectRatios {
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
    fn new(ratios: Vec<AspectRatio>, max_size: Size, count: usize) -> Self {
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

    fn evaluate(&self, rects: &[Rect]) -> f64 {
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

struct PlaceAdjacentClose {
    worst_case: f64,
}

impl PlaceAdjacentClose {
    fn new(container: Size, count: usize) -> Self {
        Self {
            // This assumes rectangles cannot exceed container bounds.
            // `container.width.get()` is not `- 1`
            // because we only compare *some* corners.
            worst_case: (count.saturating_sub(1)
                * (Pos::new(1, 1))
                    .dist(Pos::new(container.width.get(), container.height.get() - 1)))
                as f64,
        }
    }

    fn evaluate(&self, rects: &[Rect]) -> f64 {
        if rects.len() < 2 {
            0.0
        } else {
            rects
                .iter()
                .tuple_windows()
                .map(|(rect, other)| {
                    [
                        rect.top_left().dist(other.top_right()),
                        rect.top_left().dist(other.bottom_left()),
                        rect.top_right().dist(other.top_left()),
                        rect.top_right().dist(other.bottom_right()),
                        rect.bottom_left().dist(other.top_left()),
                        rect.bottom_left().dist(other.bottom_right()),
                        rect.bottom_right().dist(other.top_right()),
                        rect.bottom_right().dist(other.bottom_left()),
                    ]
                    .into_iter()
                    .min()
                    .unwrap()
                })
                .sum::<usize>() as f64
                / self.worst_case
        }
    }
}

struct PlaceInReadingOrder {
    worst_case: f64,
}

impl PlaceInReadingOrder {
    fn new(count: usize) -> Self {
        Self {
            worst_case: count.saturating_sub(1) as f64,
        }
    }

    fn evaluate(&self, rects: &[Rect]) -> f64 {
        if rects.len() < 2 {
            0.0
        } else {
            rects
                .iter()
                .tuple_windows()
                .filter(|(rect, other)| other.top() < rect.top() || other.left() < rect.left())
                .count() as f64
                / self.worst_case
        }
    }
}

struct CenterMain {
    center: Pos,
    worst_case: f64,
}

impl CenterMain {
    fn new(container: Size) -> Self {
        let center = Pos::new(container.width.get() / 2, container.height.get() / 2);
        Self {
            center,
            worst_case: center
                .dist(Pos::new(0, 0))
                .max(center.dist(container.into())) as f64,
        }
    }

    fn evaluate(&self, rects: &[Rect]) -> f64 {
        match rects.get(0) {
            Some(rect) => rect.center().dist(self.center) as f64 / self.worst_case,
            None => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::{once, repeat};

    use proptest::prelude::{prop::collection::vec, *};
    use test_strategy::proptest;

    use crate::testing::{ContainedRects, NumRectsRange};

    use super::*;

    #[proptest]
    fn minimize_gaps_returns_values_in_range_0_1(x: ContainedRects) {
        prop_assert!((0.0..=1.0).contains(&MinimizeGaps::new(x.container).evaluate(&x.rects)))
    }

    #[proptest]
    fn minimize_gaps_returns_1_for_worst_case(
        container: Size,
        #[strategy((0_usize..=16))] count: usize,
    ) {
        prop_assume!(container.width.get() > 1 || container.height.get() > 1);
        prop_assert_eq!(
            MinimizeGaps::new(container).evaluate(
                &repeat(Rect::new_checked(0, 0, 1, 1))
                    .take(count)
                    .collect_vec()
            ),
            1.0
        )
    }

    #[test]
    fn minimize_gaps_returns_0_for_best_case_without_overlap() {
        let container = Size::new_checked(10, 10);
        let rects = [
            Rect::new_checked(0, 0, 10, 5),
            Rect::new_checked(0, 5, 5, 5),
            Rect::new_checked(5, 5, 5, 5),
        ];
        assert_eq!(MinimizeGaps::new(container).evaluate(&rects), 0.0)
    }

    #[proptest]
    fn minimize_gaps_returns_0_for_best_case_with_overlap(x: ContainedRects) {
        prop_assert_eq!(
            MinimizeGaps::new(x.container).evaluate(
                &once(Rect::new(0, 0, x.container.width, x.container.height))
                    .chain(x.rects)
                    .collect_vec()
            ),
            0.0
        )
    }

    #[proptest]
    fn minimize_overlap_returns_values_in_range_0_1(x: ContainedRects) {
        prop_assert!((0.0..=1.0)
            .contains(&MinimizeOverlap::new(x.container, x.rects.len()).evaluate(&x.rects)))
    }

    #[proptest]
    fn minimize_overlap_returns_1_for_worst_case(
        container: Size,
        #[strategy((2_usize..=16))] count: usize,
    ) {
        prop_assert_eq!(
            MinimizeOverlap::new(container, count).evaluate(
                &repeat(Rect::new(0, 0, container.width, container.height))
                    .take(count)
                    .collect_vec()
            ),
            1.0
        )
    }

    #[proptest]
    fn minimize_overlap_returns_0_for_less_than_2_rects(
        container: Size,
        #[strategy((0_usize..=1))] count: usize,
    ) {
        prop_assert_eq!(
            MinimizeOverlap::new(container, count).evaluate(
                &repeat(Rect::new(0, 0, container.width, container.height))
                    .take(count)
                    .collect_vec()
            ),
            0.0
        )
    }

    #[test]
    fn minimize_overlap_returns_0_for_best_case() {
        let container = Size::new_checked(10, 10);
        let rects = [
            Rect::new_checked(0, 0, 10, 5),
            Rect::new_checked(0, 5, 5, 5),
            Rect::new_checked(5, 5, 5, 5),
        ];
        assert_eq!(
            MinimizeOverlap::new(container, rects.len()).evaluate(&rects),
            0.0
        )
    }

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

    #[proptest]
    fn place_adjacent_close_returns_values_in_range_0_1(x: ContainedRects) {
        prop_assert!((0.0..=1.0)
            .contains(&PlaceAdjacentClose::new(x.container, x.rects.len()).evaluate(&x.rects)))
    }

    #[test]
    fn place_adjacent_close_returns_1_for_worst_case() {
        // Worst case is rectangles with min size alternating opposite corners.
        let container = Size::new_checked(10, 10);
        let rects = [
            Rect::new_checked(0, 0, 1, 1),
            Rect::new_checked(9, 9, 1, 1),
            Rect::new_checked(0, 0, 1, 1),
        ];
        assert_eq!(
            PlaceAdjacentClose::new(container, rects.len()).evaluate(&rects),
            1.0
        )
    }

    #[test]
    fn place_adjacent_close_returns_0_for_best_case() {
        let container = Size::new_checked(10, 10);
        let rects = [
            Rect::new_checked(0, 0, 5, 5),
            Rect::new_checked(0, 5, 5, 5),
            Rect::new_checked(5, 5, 5, 5),
        ];
        assert_eq!(
            PlaceAdjacentClose::new(container, rects.len()).evaluate(&rects),
            0.0
        )
    }

    #[proptest]
    fn place_in_reading_order_returns_values_in_range_0_1(x: ContainedRects) {
        prop_assert!(
            (0.0..=1.0).contains(&PlaceInReadingOrder::new(x.rects.len()).evaluate(&x.rects))
        )
    }

    #[test]
    fn place_in_reading_order_returns_1_for_worst_case() {
        let rects = [
            Rect::new_checked(2, 0, 1, 1),
            Rect::new_checked(1, 0, 1, 1),
            Rect::new_checked(0, 0, 1, 1),
        ];
        assert_eq!(PlaceInReadingOrder::new(rects.len()).evaluate(&rects), 1.0);
        let rects = [
            Rect::new_checked(0, 2, 1, 1),
            Rect::new_checked(0, 1, 1, 1),
            Rect::new_checked(0, 0, 1, 1),
        ];
        assert_eq!(PlaceInReadingOrder::new(rects.len()).evaluate(&rects), 1.0);
    }

    #[test]
    fn place_in_reading_order_returns_0_for_best_case() {
        let rects = [
            Rect::new_checked(0, 0, 1, 1),
            Rect::new_checked(1, 0, 1, 1),
            Rect::new_checked(2, 0, 1, 1),
        ];
        assert_eq!(PlaceInReadingOrder::new(rects.len()).evaluate(&rects), 0.0);
        let rects = [
            Rect::new_checked(0, 0, 1, 1),
            Rect::new_checked(0, 1, 1, 1),
            Rect::new_checked(0, 2, 1, 1),
        ];
        assert_eq!(PlaceInReadingOrder::new(rects.len()).evaluate(&rects), 0.0);
    }

    #[proptest]
    fn center_main_returns_values_in_range_0_1(x: ContainedRects) {
        prop_assert!((0.0..=1.0).contains(&CenterMain::new(x.container).evaluate(&x.rects)))
    }

    #[test]
    fn center_main_returns_1_for_worst_case() {
        let container = Size::new_checked(10, 10);
        let rects = [
            Rect::new_checked(0, 0, 1, 1),
            Rect::new_checked(0, 5, 5, 5),
            Rect::new_checked(0, 0, 10, 10),
        ];
        assert_eq!(CenterMain::new(container).evaluate(&rects), 1.0)
    }

    #[test]
    fn center_main_returns_0_for_centered_main() {
        let container = Size::new_checked(12, 12);
        let rects = [
            Rect::new_checked(3, 3, 6, 6),
            Rect::new_checked(0, 0, 12, 12),
            Rect::new_checked(0, 5, 5, 5),
        ];
        assert_eq!(CenterMain::new(container).evaluate(&rects), 0.0)
    }

    #[proptest]
    fn center_main_returns_0_for_full_main(x: ContainedRects) {
        assert_eq!(
            CenterMain::new(x.container).evaluate(
                &once(Rect::new(0, 0, x.container.width, x.container.height))
                    .chain(x.rects)
                    .collect_vec()
            ),
            0.0
        )
    }
}
