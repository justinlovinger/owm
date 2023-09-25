use crate::{Pos, Rect, Size};

pub struct CenterMain {
    center: Pos,
    worst_case: f64,
}

impl CenterMain {
    pub fn new(container: Size) -> Self {
        let center = Pos::new(container.width.get() / 2, container.height.get() / 2);
        Self {
            center,
            worst_case: center
                .dist(Pos::new(0, 0))
                .max(center.dist(container.into())) as f64,
        }
    }

    pub fn evaluate(&self, rects: &[Rect]) -> f64 {
        match rects.get(0) {
            Some(rect) => rect.center().dist(self.center) as f64 / self.worst_case,
            None => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::once;

    use itertools::Itertools;
    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::testing::ContainedRects;

    use super::*;

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
