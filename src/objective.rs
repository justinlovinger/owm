use itertools::Itertools;

use crate::types::{covered_area, obscured_area, Pos, Size, Window};

pub struct Problem {
    gaps: MinimizeGaps,
    overlapping: MinimizeOverlapping,
    higher_larger_area: GiveHigherInStackLargerArea,
    near_in_stack_close: PlaceNearInStackClose,
    reading_order: PlaceInReadingOrder,
    center_main: CenterMain,
}

impl Problem {
    pub fn new(container: Size, window_count: usize) -> Self {
        Self {
            gaps: MinimizeGaps::new(container),
            overlapping: MinimizeOverlapping::new(container, window_count),
            higher_larger_area: GiveHigherInStackLargerArea::new(2.0, container, window_count),
            near_in_stack_close: PlaceNearInStackClose::new(container, window_count),
            reading_order: PlaceInReadingOrder::new(window_count),
            center_main: CenterMain::new(container),
        }
    }

    pub fn evaluate(&self, windows: &[Window]) -> f64 {
        3.0 * self.gaps.evaluate(windows)
            + 2.0 * self.overlapping.evaluate(windows)
            + 1.5 * self.higher_larger_area.evaluate(windows)
            + 0.5 * self.near_in_stack_close.evaluate(windows)
            + 0.5 * self.reading_order.evaluate(windows)
            + 3.0 * self.center_main.evaluate(windows)
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

    fn evaluate(&self, windows: &[Window]) -> f64 {
        if windows.is_empty() {
            1.0
        } else {
            // This assumes windows do not exceed container bounds.
            (self.area - covered_area(windows)) as f64 / self.worst_case
        }
    }
}

struct MinimizeOverlapping {
    worst_case: f64,
}

impl MinimizeOverlapping {
    fn new(container: Size, window_count: usize) -> Self {
        Self {
            worst_case: (window_count.saturating_sub(1) * container.area()) as f64,
        }
    }

    fn evaluate(&self, windows: &[Window]) -> f64 {
        if windows.len() < 2 {
            0.0
        } else {
            obscured_area(windows) as f64 / self.worst_case
        }
    }
}

struct GiveHigherInStackLargerArea {
    ratio: f64,
    worst_case: f64,
}

impl GiveHigherInStackLargerArea {
    fn new(ratio: f64, container: Size, window_count: usize) -> Self {
        debug_assert!(ratio >= 1.0);
        Self {
            ratio,
            // The first pair of windows can be `container.area()` apart in area,
            // but then remaining pairs can only be equal at worst.
            worst_case: ratio * container.area() as f64
                + (ratio - 1.0) * (container.area() * window_count.saturating_sub(2)) as f64,
        }
    }

    fn evaluate(&self, windows: &[Window]) -> f64 {
        if windows.len() < 2 {
            0.0
        } else {
            windows
                .iter()
                .map(|x| x.area() as f64)
                .tuple_windows()
                // Use `.abs()` instead of `.max(0.0)`
                // to encourage lower windows to grow
                // when possible.
                // Otherwise,
                // the lowest window can always be small
                // with no penalty.
                .map(|(x, y)| (self.ratio * y - x).abs())
                .sum::<f64>()
                / self.worst_case
        }
    }
}

struct PlaceNearInStackClose {
    worst_case: f64,
}

impl PlaceNearInStackClose {
    fn new(container: Size, window_count: usize) -> Self {
        Self {
            worst_case: (window_count.saturating_sub(1) * (Pos::new(0, 0)).dist(container.into()))
                as f64,
        }
    }

    fn evaluate(&self, windows: &[Window]) -> f64 {
        if windows.len() < 2 {
            0.0
        } else {
            windows
                .iter()
                .tuple_windows()
                .map(|(window, other)| {
                    [
                        window.top_left().dist(other.top_right()),
                        window.top_left().dist(other.bottom_left()),
                        window.top_right().dist(other.top_left()),
                        window.top_right().dist(other.bottom_right()),
                        window.bottom_left().dist(other.top_left()),
                        window.bottom_left().dist(other.bottom_right()),
                        window.bottom_right().dist(other.top_right()),
                        window.bottom_right().dist(other.bottom_left()),
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
    fn new(window_count: usize) -> Self {
        Self {
            worst_case: window_count.saturating_sub(1) as f64,
        }
    }

    fn evaluate(&self, windows: &[Window]) -> f64 {
        if windows.len() < 2 {
            0.0
        } else {
            windows
                .iter()
                .tuple_windows()
                .filter(|(window, other)| {
                    other.top() < window.top() || other.left() < window.left()
                })
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

    fn evaluate(&self, windows: &[Window]) -> f64 {
        match windows.get(0) {
            Some(window) => window.center().dist(self.center) as f64 / self.worst_case,
            None => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::{once, repeat};

    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::testing::ContainedWindows;

    use super::*;

    #[proptest]
    fn minimize_gaps_returns_values_in_range_0_1(x: ContainedWindows) {
        prop_assert!((0.0..=1.0).contains(&MinimizeGaps::new(x.container).evaluate(&x.windows)))
    }

    #[proptest]
    fn minimize_gaps_returns_1_for_worst_case(
        container: Size,
        #[strategy((0_usize..=16))] count: usize,
    ) {
        prop_assert_eq!(
            MinimizeGaps::new(container)
                .evaluate(&repeat(Window::new(0, 0, 0, 0)).take(count).collect_vec()),
            1.0
        )
    }

    #[test]
    fn minimize_gaps_returns_0_for_best_case_without_overlap() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let windows = [
            Window::new(0, 0, 10, 5),
            Window::new(0, 5, 5, 5),
            Window::new(5, 5, 5, 5),
        ];
        assert_eq!(MinimizeGaps::new(container).evaluate(&windows), 0.0)
    }

    #[proptest]
    fn minimize_gaps_returns_0_for_best_case_with_overlap(x: ContainedWindows) {
        prop_assert_eq!(
            MinimizeGaps::new(x.container).evaluate(
                &once(Window::new(0, 0, x.container.width, x.container.height))
                    .chain(x.windows)
                    .collect_vec()
            ),
            0.0
        )
    }

    #[proptest]
    fn minimize_overlapping_returns_values_in_range_0_1(x: ContainedWindows) {
        prop_assert!((0.0..=1.0)
            .contains(&MinimizeOverlapping::new(x.container, x.windows.len()).evaluate(&x.windows)))
    }

    #[proptest]
    fn minimize_overlapping_returns_1_for_worst_case(
        container: Size,
        #[strategy((2_usize..=16))] count: usize,
    ) {
        prop_assert_eq!(
            MinimizeOverlapping::new(container, count).evaluate(
                &repeat(Window::new(0, 0, container.width, container.height))
                    .take(count)
                    .collect_vec()
            ),
            1.0
        )
    }

    #[proptest]
    fn minimize_overlapping_returns_0_for_less_than_2_windows(
        container: Size,
        #[strategy((0_usize..=1))] count: usize,
    ) {
        prop_assert_eq!(
            MinimizeOverlapping::new(container, count).evaluate(
                &repeat(Window::new(0, 0, container.width, container.height))
                    .take(count)
                    .collect_vec()
            ),
            0.0
        )
    }

    #[test]
    fn minimize_overlapping_returns_0_for_best_case() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let windows = [
            Window::new(0, 0, 10, 5),
            Window::new(0, 5, 5, 5),
            Window::new(5, 5, 5, 5),
        ];
        assert_eq!(
            MinimizeOverlapping::new(container, windows.len()).evaluate(&windows),
            0.0
        )
    }

    #[proptest]
    fn give_higher_in_stack_larger_area_returns_values_in_range_0_1(
        #[strategy((1.0..=100.0))] ratio: f64,
        x: ContainedWindows,
    ) {
        prop_assert!((0.0..=1.0).contains(
            &GiveHigherInStackLargerArea::new(ratio, x.container, x.windows.len())
                .evaluate(&x.windows)
        ))
    }

    #[test]
    fn give_higher_in_stack_larger_area_returns_1_for_worst_case() {
        // Note,
        // what exactly counts as the worst case
        // is uncertain.
        // We could define the worst case
        // as the reverse of the best case.
        // However,
        // then the middle window has a good area
        // for its position.
        let container = Size {
            width: 10,
            height: 10,
        };
        let windows = [
            Window::new(0, 0, 0, 0),
            Window::new(0, 0, 10, 10),
            Window::new(0, 0, 10, 10),
        ];
        assert_eq!(
            GiveHigherInStackLargerArea::new(2.0, container, windows.len()).evaluate(&windows),
            1.0
        )
    }

    #[test]
    fn give_higher_in_stack_larger_area_returns_0_for_best_case() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let windows = [
            Window::new(0, 0, 10, 10),
            Window::new(0, 0, 10, 5),
            Window::new(0, 0, 5, 5),
        ];
        assert_eq!(
            GiveHigherInStackLargerArea::new(2.0, container, windows.len()).evaluate(&windows),
            0.0
        )
    }

    #[proptest]
    fn place_near_in_stack_close_returns_values_in_range_0_1(x: ContainedWindows) {
        prop_assert!((0.0..=1.0).contains(
            &PlaceNearInStackClose::new(x.container, x.windows.len()).evaluate(&x.windows)
        ))
    }

    #[test]
    fn place_near_in_stack_close_returns_1_for_worst_case() {
        // Worst case is windows with zero size alternating opposite corners.
        let container = Size {
            width: 10,
            height: 10,
        };
        let windows = [
            Window::new(0, 0, 0, 0),
            Window::new(10, 10, 0, 0),
            Window::new(0, 0, 0, 0),
        ];
        assert_eq!(
            PlaceNearInStackClose::new(container, windows.len()).evaluate(&windows),
            1.0
        )
    }

    #[test]
    fn place_near_in_stack_close_returns_0_for_best_case() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let windows = [
            Window::new(0, 0, 5, 5),
            Window::new(0, 5, 5, 5),
            Window::new(5, 5, 5, 5),
        ];
        assert_eq!(
            PlaceNearInStackClose::new(container, windows.len()).evaluate(&windows),
            0.0
        )
    }

    #[proptest]
    fn place_in_reading_order_returns_values_in_range_0_1(x: ContainedWindows) {
        prop_assert!(
            (0.0..=1.0).contains(&PlaceInReadingOrder::new(x.windows.len()).evaluate(&x.windows))
        )
    }

    #[test]
    fn place_in_reading_order_returns_1_for_worst_case() {
        let windows = [
            Window::new(2, 0, 0, 0),
            Window::new(1, 0, 0, 0),
            Window::new(0, 0, 0, 0),
        ];
        assert_eq!(
            PlaceInReadingOrder::new(windows.len()).evaluate(&windows),
            1.0
        );
        let windows = [
            Window::new(0, 2, 0, 0),
            Window::new(0, 1, 0, 0),
            Window::new(0, 0, 0, 0),
        ];
        assert_eq!(
            PlaceInReadingOrder::new(windows.len()).evaluate(&windows),
            1.0
        );
    }

    #[test]
    fn place_in_reading_order_returns_0_for_best_case() {
        let windows = [
            Window::new(0, 0, 0, 0),
            Window::new(1, 0, 0, 0),
            Window::new(2, 0, 0, 0),
        ];
        assert_eq!(
            PlaceInReadingOrder::new(windows.len()).evaluate(&windows),
            0.0
        );
        let windows = [
            Window::new(0, 0, 0, 0),
            Window::new(0, 1, 0, 0),
            Window::new(0, 2, 0, 0),
        ];
        assert_eq!(
            PlaceInReadingOrder::new(windows.len()).evaluate(&windows),
            0.0
        );
    }

    #[proptest]
    fn center_main_returns_values_in_range_0_1(x: ContainedWindows) {
        prop_assert!((0.0..=1.0).contains(&CenterMain::new(x.container).evaluate(&x.windows)))
    }

    #[test]
    fn center_main_returns_1_for_worst_case() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let windows = [
            Window::new(0, 0, 0, 0),
            Window::new(0, 5, 5, 5),
            Window::new(0, 0, 10, 10),
        ];
        assert_eq!(CenterMain::new(container).evaluate(&windows), 1.0)
    }

    #[test]
    fn center_main_returns_0_for_centered_main() {
        let container = Size {
            width: 12,
            height: 12,
        };
        let windows = [
            Window::new(3, 3, 6, 6),
            Window::new(0, 0, 12, 12),
            Window::new(0, 5, 5, 5),
        ];
        assert_eq!(CenterMain::new(container).evaluate(&windows), 0.0)
    }

    #[proptest]
    fn center_main_returns_0_for_full_main(x: ContainedWindows) {
        assert_eq!(
            CenterMain::new(x.container).evaluate(
                &once(Window::new(0, 0, x.container.width, x.container.height))
                    .chain(x.windows)
                    .collect_vec()
            ),
            0.0
        )
    }
}
