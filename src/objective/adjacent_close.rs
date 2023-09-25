use itertools::Itertools;

use crate::{Pos, Rect, Size};

pub struct PlaceAdjacentClose {
    worst_case: f64,
}

impl PlaceAdjacentClose {
    pub fn new(container: Size, count: usize) -> Self {
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

    pub fn evaluate(&self, rects: &[Rect]) -> f64 {
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

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::testing::ContainedRects;

    use super::*;

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
}
