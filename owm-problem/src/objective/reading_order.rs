use itertools::Itertools;

use crate::Rect;

pub struct PlaceInReadingOrder {
    worst_case: f64,
}

impl PlaceInReadingOrder {
    pub fn new(count: usize) -> Self {
        Self {
            worst_case: count.saturating_sub(1) as f64,
        }
    }

    pub fn evaluate(&self, rects: &[Rect]) -> f64 {
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

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::testing::ContainedRects;

    use super::*;

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
}
