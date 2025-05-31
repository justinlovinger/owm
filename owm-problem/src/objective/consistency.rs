use crate::{Rect, Size};

pub struct MaximizeConsistency {
    previous_layout: Vec<Rect>,
    worst_case: f64,
}

impl MaximizeConsistency {
    pub fn new(container: Size, previous_layout: Vec<Rect>) -> Self {
        let max_pos_x = container.width.get() - 1;
        let max_pos_y = container.height.get() - 1;
        let max_width = container.width.get();
        let max_height = container.height.get();
        Self {
            // This assumes rects cannot exceed their container.
            worst_case: previous_layout
                .iter()
                .map(|rect| {
                    [
                        Rect::new_checked(0, 0, 1, 1),
                        Rect::new_checked(0, 0, max_width, 1),
                        Rect::new_checked(0, 0, 1, max_height),
                        Rect::new_checked(0, 0, max_width, max_height),
                        Rect::new_checked(max_pos_x, 0, 1, 1),
                        Rect::new_checked(max_pos_x, 0, 1, max_height),
                        Rect::new_checked(0, max_pos_y, 1, 1),
                        Rect::new_checked(0, max_pos_y, max_width, 1),
                        Rect::new_checked(max_pos_x, max_pos_y, 1, 1),
                    ]
                    .into_iter()
                    .map(|other| rect.diff(other))
                    .max()
                    .unwrap()
                })
                .sum::<usize>() as f64,
            previous_layout,
        }
    }

    pub fn evaluate(&self, rects: &[Rect]) -> f64 {
        if self.worst_case == 0.0 {
            0.0
        } else {
            rects
                .iter()
                .zip(&self.previous_layout)
                .map(|(rect, prev)| rect.diff(*prev))
                .sum::<usize>() as f64
                / self.worst_case
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::testing::{ContainedRects, ContainedRectsParams};

    use super::*;

    #[proptest]
    fn maximize_consistency_returns_values_in_range_0_1(
        #[strategy(arbitrary_maximize_consistency_args())] args: (ContainedRects, Vec<Rect>),
    ) {
        prop_assert!((0.0..=1.0)
            .contains(&MaximizeConsistency::new(args.0.container, args.0.rects).evaluate(&args.1)))
    }

    #[test]
    fn maximize_consistency_returns_1_for_worst_case() {
        let container = Size::new_checked(10, 10);
        let prev = [
            Rect::new_checked(0, 0, 10, 10),
            Rect::new_checked(9, 9, 1, 1),
            Rect::new_checked(4, 4, 1, 1),
        ];
        let rects = [
            Rect::new_checked(9, 9, 1, 1),
            Rect::new_checked(0, 0, 10, 10),
            Rect::new_checked(0, 0, 10, 10),
            Rect::new_checked(0, 0, 1, 1),
        ];
        assert_eq!(
            MaximizeConsistency::new(container, prev.to_vec()).evaluate(&rects),
            1.0
        );
    }

    #[proptest]
    fn maximize_consistency_returns_0_for_best_case(
        #[strategy(arbitrary_maximize_consistency_args())] args: (ContainedRects, Vec<Rect>),
    ) {
        assert_eq!(
            MaximizeConsistency::new(args.0.container, args.0.rects.clone()).evaluate(
                &args
                    .0
                    .rects
                    .into_iter()
                    .chain(args.1.into_iter())
                    .collect_vec()
            ),
            0.0
        );
    }

    fn arbitrary_maximize_consistency_args() -> BoxedStrategy<(ContainedRects, Vec<Rect>)> {
        ContainedRects::arbitrary()
            .prop_flat_map(|x| {
                (
                    ContainedRects::arbitrary_with(ContainedRectsParams {
                        width_range: x.container.width..=x.container.width,
                        height_range: x.container.height..=x.container.height,
                        len_range: (x.rects.len() + 1)..=(x.rects.len() + 17),
                    }),
                    Just(x),
                )
            })
            .prop_map(|(x, y)| (y, x.rects))
            .boxed()
    }
}
