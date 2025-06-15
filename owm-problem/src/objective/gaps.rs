use std::num::NonZeroUsize;

use crate::{rect::covered_area, Rect, Size};

pub struct MinimizeGaps {
    area: NonZeroUsize,
    worst_case: f64,
}

impl MinimizeGaps {
    pub fn new(container: Size) -> Self {
        Self {
            area: container.area(),
            worst_case: (container.area().get() - 1) as f64,
        }
    }

    pub fn evaluate(&self, rects: &[Rect]) -> f64 {
        if rects.is_empty() {
            1.0
        } else {
            // This assumes rectangles do not exceed container bounds.
            // Worst case can theoretically be zero,
            // if `container.area()` is `1`,
            // but this is unrealistic in practice.
            (self.area.get() - covered_area(rects)) as f64 / self.worst_case
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::{once, repeat};

    use itertools::Itertools;
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
}
