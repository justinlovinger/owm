use crate::{rect::obscured_area, Rect, Size};

pub struct MinimizeOverlap {
    worst_case: f64,
}

impl MinimizeOverlap {
    pub fn new(container: Size, count: usize) -> Self {
        Self {
            worst_case: (count.saturating_sub(1) * container.area().get()) as f64,
        }
    }

    pub fn evaluate(&self, rects: &[Rect]) -> f64 {
        if rects.len() < 2 {
            0.0
        } else {
            obscured_area(rects) as f64 / self.worst_case
        }
    }
}
#[cfg(test)]
mod tests {
    use std::iter::repeat;

    use itertools::Itertools;
    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::testing::ContainedRects;

    use super::*;

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
}
