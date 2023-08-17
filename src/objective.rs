use std::ops::Mul;

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
    area_ratio: MaintainAreaRatio,
    adjacent_close: PlaceAdjacentClose,
    reading_order: PlaceInReadingOrder,
    center_main: CenterMain,
}

#[derive(Clone, Copy, Debug)]
pub struct Weights {
    pub gaps_weight: Weight,
    pub overlap_weight: Weight,
    pub area_ratio_weight: Weight,
    pub adjacent_close_weight: Weight,
    pub reading_order_weight: Weight,
    pub center_main_weight: Weight,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            gaps_weight: Weight(3.0),
            overlap_weight: Weight(2.0),
            area_ratio_weight: Weight(1.5),
            adjacent_close_weight: Weight(0.5),
            reading_order_weight: Weight(0.5),
            center_main_weight: Weight(3.0),
        }
    }
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
    pub fn new(weights: Weights, area_ratio: Ratio, container: Size, count: usize) -> Self {
        Self {
            weights,
            gaps: MinimizeGaps::new(container),
            overlap: MinimizeOverlap::new(container, count),
            area_ratio: MaintainAreaRatio::new(area_ratio, container, count),
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
        }) + (if self.weights.area_ratio_weight > Weight(0.0) {
            self.weights.area_ratio_weight * self.area_ratio.evaluate(rects)
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
    area: usize,
    worst_case: f64,
}

impl MinimizeGaps {
    fn new(container: Size) -> Self {
        Self {
            area: container.area(),
            worst_case: container.area() as f64,
        }
    }

    fn evaluate(&self, rects: &[Rect]) -> f64 {
        if rects.is_empty() {
            1.0
        } else {
            // This assumes rectangles do not exceed container bounds.
            (self.area - covered_area(rects)) as f64 / self.worst_case
        }
    }
}

struct MinimizeOverlap {
    worst_case: f64,
}

impl MinimizeOverlap {
    fn new(container: Size, count: usize) -> Self {
        Self {
            worst_case: (count.saturating_sub(1) * container.area()) as f64,
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

struct MaintainAreaRatio {
    ratio: Ratio,
    worst_case: f64,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, PartialOrd)]
pub struct Ratio(f64);

impl LowerBounded for Ratio {
    fn min_value() -> Self {
        Self(1.0)
    }
}

derive_new_from_lower_bounded_float!(Ratio(f64));
derive_try_from_from_new!(Ratio(f64));
derive_from_str_from_try_into!(Ratio(f64));

impl Mul<f64> for Ratio {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        self.0 * rhs
    }
}

impl MaintainAreaRatio {
    fn new(ratio: Ratio, container: Size, count: usize) -> Self {
        Self {
            ratio,
            // The first pair can be `container.area()` apart in area,
            // but then remaining pairs can only be equal at worst.
            worst_case: ratio * container.area() as f64
                + (ratio.0 - 1.0) * (container.area() * count.saturating_sub(2)) as f64,
        }
    }

    fn evaluate(&self, rects: &[Rect]) -> f64 {
        if rects.len() < 2 {
            0.0
        } else {
            rects
                .iter()
                .map(|x| x.area() as f64)
                .tuple_windows()
                // Use `.abs()` instead of `.max(0.0)`
                // to encourage later to grow
                // when possible.
                // Otherwise,
                // the last rectangle can always be small
                // with no penalty.
                .map(|(x, y)| (self.ratio * y - x).abs())
                .sum::<f64>()
                / self.worst_case
        }
    }
}

struct PlaceAdjacentClose {
    worst_case: f64,
}

impl PlaceAdjacentClose {
    fn new(container: Size, count: usize) -> Self {
        Self {
            worst_case: (count.saturating_sub(1) * (Pos::new(0, 0)).dist(container.into())) as f64,
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
        let center = Pos::new(container.width / 2, container.height / 2);
        Self {
            center,
            worst_case: center
                .dist(Pos::new(0, 0))
                .max(center.dist(Pos::new(container.width, container.height)))
                as f64,
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

    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::testing::ContainedRects;

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
        prop_assert_eq!(
            MinimizeGaps::new(container)
                .evaluate(&repeat(Rect::new(0, 0, 0, 0)).take(count).collect_vec()),
            1.0
        )
    }

    #[test]
    fn minimize_gaps_returns_0_for_best_case_without_overlap() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let rects = [
            Rect::new(0, 0, 10, 5),
            Rect::new(0, 5, 5, 5),
            Rect::new(5, 5, 5, 5),
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
        let container = Size {
            width: 10,
            height: 10,
        };
        let rects = [
            Rect::new(0, 0, 10, 5),
            Rect::new(0, 5, 5, 5),
            Rect::new(5, 5, 5, 5),
        ];
        assert_eq!(
            MinimizeOverlap::new(container, rects.len()).evaluate(&rects),
            0.0
        )
    }

    #[proptest]
    fn maintain_area_ratio_returns_values_in_range_0_1(
        #[strategy((1.0..=100.0))] ratio: f64,
        x: ContainedRects,
    ) {
        prop_assert!((0.0..=1.0).contains(
            &MaintainAreaRatio::new(Ratio::new(ratio).unwrap(), x.container, x.rects.len())
                .evaluate(&x.rects)
        ))
    }

    #[test]
    fn maintain_area_ratio_returns_1_for_worst_case() {
        // Note,
        // what exactly counts as the worst case
        // is uncertain.
        // We could define the worst case
        // as the reverse of the best case.
        // However,
        // then the middle rectangle has a good area
        // for its position.
        let container = Size {
            width: 10,
            height: 10,
        };
        let rects = [
            Rect::new(0, 0, 0, 0),
            Rect::new(0, 0, 10, 10),
            Rect::new(0, 0, 10, 10),
        ];
        assert_eq!(
            MaintainAreaRatio::new(Ratio(2.0), container, rects.len()).evaluate(&rects),
            1.0
        )
    }

    #[test]
    fn maintain_area_ratio_returns_0_for_best_case() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let rects = [
            Rect::new(0, 0, 10, 10),
            Rect::new(0, 0, 10, 5),
            Rect::new(0, 0, 5, 5),
        ];
        assert_eq!(
            MaintainAreaRatio::new(Ratio(2.0), container, rects.len()).evaluate(&rects),
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
        // Worst case is rectangles with zero size alternating opposite corners.
        let container = Size {
            width: 10,
            height: 10,
        };
        let rects = [
            Rect::new(0, 0, 0, 0),
            Rect::new(10, 10, 0, 0),
            Rect::new(0, 0, 0, 0),
        ];
        assert_eq!(
            PlaceAdjacentClose::new(container, rects.len()).evaluate(&rects),
            1.0
        )
    }

    #[test]
    fn place_adjacent_close_returns_0_for_best_case() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let rects = [
            Rect::new(0, 0, 5, 5),
            Rect::new(0, 5, 5, 5),
            Rect::new(5, 5, 5, 5),
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
            Rect::new(2, 0, 0, 0),
            Rect::new(1, 0, 0, 0),
            Rect::new(0, 0, 0, 0),
        ];
        assert_eq!(PlaceInReadingOrder::new(rects.len()).evaluate(&rects), 1.0);
        let rects = [
            Rect::new(0, 2, 0, 0),
            Rect::new(0, 1, 0, 0),
            Rect::new(0, 0, 0, 0),
        ];
        assert_eq!(PlaceInReadingOrder::new(rects.len()).evaluate(&rects), 1.0);
    }

    #[test]
    fn place_in_reading_order_returns_0_for_best_case() {
        let rects = [
            Rect::new(0, 0, 0, 0),
            Rect::new(1, 0, 0, 0),
            Rect::new(2, 0, 0, 0),
        ];
        assert_eq!(PlaceInReadingOrder::new(rects.len()).evaluate(&rects), 0.0);
        let rects = [
            Rect::new(0, 0, 0, 0),
            Rect::new(0, 1, 0, 0),
            Rect::new(0, 2, 0, 0),
        ];
        assert_eq!(PlaceInReadingOrder::new(rects.len()).evaluate(&rects), 0.0);
    }

    #[proptest]
    fn center_main_returns_values_in_range_0_1(x: ContainedRects) {
        prop_assert!((0.0..=1.0).contains(&CenterMain::new(x.container).evaluate(&x.rects)))
    }

    #[test]
    fn center_main_returns_1_for_worst_case() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let rects = [
            Rect::new(0, 0, 0, 0),
            Rect::new(0, 5, 5, 5),
            Rect::new(0, 0, 10, 10),
        ];
        assert_eq!(CenterMain::new(container).evaluate(&rects), 1.0)
    }

    #[test]
    fn center_main_returns_0_for_centered_main() {
        let container = Size {
            width: 12,
            height: 12,
        };
        let rects = [
            Rect::new(3, 3, 6, 6),
            Rect::new(0, 0, 12, 12),
            Rect::new(0, 5, 5, 5),
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
