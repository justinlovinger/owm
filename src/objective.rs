use itertools::Itertools;
use ndarray::Array;

use crate::types::{Pos, Size, Window};

pub struct Problem {
    minimize_gaps: MinimizeGaps,
    minimize_overlapping: MinimizeOverlapping,
    higher_windows_larger_area: HigherWindowsShouldHaveLargerArea,
    minimum_size: WindowsShouldHaveMinimumSize,
    windows_near_in_stack_close: WindowsNearInStackShouldBeClose,
}

impl Problem {
    pub fn new(container: Size, window_count: usize) -> Self {
        Self {
            minimize_gaps: MinimizeGaps::new(container),
            minimize_overlapping: MinimizeOverlapping::new(container, window_count),
            higher_windows_larger_area: HigherWindowsShouldHaveLargerArea::new(
                container,
                window_count,
            ),
            minimum_size: WindowsShouldHaveMinimumSize::new(
                Size {
                    width: 800,
                    height: 600,
                },
                window_count,
            ),
            windows_near_in_stack_close: WindowsNearInStackShouldBeClose::new(
                container,
                window_count,
            ),
        }
    }

    pub fn evaluate(&self, windows: &[Window]) -> f64 {
        20.0 * self.minimize_gaps.evaluate(windows)
            + 10.0 * self.minimize_overlapping.evaluate(windows)
            + self.higher_windows_larger_area.evaluate(windows)
            + 2.0 * self.minimum_size.evaluate(windows)
            + self.windows_near_in_stack_close.evaluate(windows)
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

// Adapted from a solution by `m-hgn` on Code Wars,
// <https://www.codewars.com/kata/reviews/6380bc55c34ac10001dde712/groups/63b6d7c8ec0d060001ce20f1>.
// This could be optimized using segment trees.
/// Return the total area of a union of rectangles.
fn covered_area(windows: &[Window]) -> usize {
    let mut xs = windows
        .iter()
        .flat_map(|window| [window.left(), window.right()])
        .collect_vec();
    xs.sort();
    xs.dedup();

    let mut windows = windows.to_vec();
    windows.sort_by_key(|window| window.top());

    xs.into_iter()
        .tuple_windows()
        .map(|(left, right)| {
            let width = right - left;
            let mut last_y2 = usize::MIN;
            windows
                .iter()
                .filter(|window| window.left() <= left && right <= window.right())
                .map(|window| {
                    let ret = width * window.bottom().saturating_sub(last_y2.max(window.top()));
                    last_y2 = window.bottom().max(last_y2);
                    ret
                })
                .sum::<usize>()
        })
        .sum()
}

/// Return the total area obscured in a set of rectangles.
/// If `n` rectangles are overlapped by an `n + 1`th rectangle,
/// the overlapped area will be counted `n` times,
/// but not `n + 1` times.
fn obscured_area(windows: &[Window]) -> usize {
    if windows.len() < 2 {
        0
    } else {
        let overlaps = windows
            .iter()
            .enumerate()
            .map(|(i, window)| {
                windows
                    .iter()
                    .enumerate()
                    .filter(|(other_i, _)| i != *other_i)
                    .filter_map(|(_, other)| window.overlap(other))
                    .collect_vec()
            })
            .collect_vec();
        overlaps.iter().map(|x| covered_area(x)).sum::<usize>()
            - covered_area(&overlaps.into_iter().flatten().collect_vec())
    }
}

struct HigherWindowsShouldHaveLargerArea {
    max_area: f64,
    ideals: Vec<f64>,
    worst_case: f64,
}

impl HigherWindowsShouldHaveLargerArea {
    fn new(container: Size, window_count: usize) -> Self {
        let ideals = Array::linspace(1.0, 0.0, window_count).into_raw_vec();
        Self {
            max_area: container.area() as f64,
            worst_case: ideals.iter().copied().map(|x: f64| x.max(1.0 - x)).sum(),
            ideals,
        }
    }

    fn evaluate(&self, windows: &[Window]) -> f64 {
        if windows.len() < 2 {
            0.0
        } else {
            windows
                .iter()
                .zip(self.ideals.iter())
                .map(|(window, ideal)| ((window.area() as f64 / self.max_area) - ideal).abs())
                .sum::<f64>()
                / self.worst_case
        }
    }
}

struct WindowsShouldHaveMinimumSize {
    size: Size,
    width: f64,
    height: f64,
    worst_case: f64,
}

impl WindowsShouldHaveMinimumSize {
    fn new(size: Size, window_count: usize) -> Self {
        Self {
            size,
            width: size.width as f64,
            height: size.height as f64,
            worst_case: window_count as f64,
        }
    }

    fn evaluate(&self, windows: &[Window]) -> f64 {
        if windows.is_empty() {
            0.0
        } else {
            windows
                .iter()
                .map(
                    |window| match (self.size.width == 0, self.size.height == 0) {
                        (true, true) => 0.0,
                        (true, false) => self.evaluate_height(window),
                        (false, true) => self.evaluate_width(window),
                        (false, false) => {
                            1.0 - ((1.0 - self.evaluate_width(window))
                                * (1.0 - self.evaluate_height(window)))
                        }
                    },
                )
                .sum::<f64>()
                / self.worst_case
        }
    }

    fn evaluate_width(&self, window: &Window) -> f64 {
        self.size.width.saturating_sub(window.size.width) as f64 / self.width
    }

    fn evaluate_height(&self, window: &Window) -> f64 {
        self.size.height.saturating_sub(window.size.height) as f64 / self.height
    }
}

struct WindowsNearInStackShouldBeClose {
    worst_case: f64,
}

impl WindowsNearInStackShouldBeClose {
    fn new(container: Size, window_count: usize) -> Self {
        Self {
            worst_case: (window_count.saturating_sub(1)
                * (Pos { x: 0, y: 0 }).dist(container.into())) as f64,
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

#[cfg(test)]
mod tests {
    use std::iter::{once, repeat};

    use ndarray::prelude::*;
    use proptest::prelude::{prop::collection::vec, *};
    use test_strategy::proptest;

    use crate::encoding::Decoder;

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
            MinimizeGaps::new(container).evaluate(
                &repeat(Window {
                    pos: Pos { x: 0, y: 0 },
                    size: Size {
                        width: 0,
                        height: 0
                    },
                })
                .take(count)
                .collect_vec()
            ),
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
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 10,
                    height: 5,
                },
            },
            Window {
                pos: Pos { x: 0, y: 5 },
                size: Size {
                    width: 5,
                    height: 5,
                },
            },
            Window {
                pos: Pos { x: 5, y: 5 },
                size: Size {
                    width: 5,
                    height: 5,
                },
            },
        ];
        assert_eq!(MinimizeGaps::new(container).evaluate(&windows), 0.0)
    }

    #[proptest]
    fn minimize_gaps_returns_0_for_best_case_with_overlap(x: ContainedWindows) {
        prop_assert_eq!(
            MinimizeGaps::new(x.container).evaluate(
                &once(Window {
                    pos: Pos { x: 0, y: 0 },
                    size: x.container
                })
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
                &repeat(Window {
                    pos: Pos { x: 0, y: 0 },
                    size: container
                })
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
                &repeat(Window {
                    pos: Pos { x: 0, y: 0 },
                    size: container
                })
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
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 10,
                    height: 5,
                },
            },
            Window {
                pos: Pos { x: 0, y: 5 },
                size: Size {
                    width: 5,
                    height: 5,
                },
            },
            Window {
                pos: Pos { x: 5, y: 5 },
                size: Size {
                    width: 5,
                    height: 5,
                },
            },
        ];
        assert_eq!(
            MinimizeOverlapping::new(container, windows.len()).evaluate(&windows),
            0.0
        )
    }

    #[proptest]
    fn higher_windows_should_have_larger_area_returns_values_in_range_0_1(x: ContainedWindows) {
        prop_assert!((0.0..=1.0).contains(
            &HigherWindowsShouldHaveLargerArea::new(x.container, x.windows.len())
                .evaluate(&x.windows)
        ))
    }

    #[test]
    fn higher_windows_should_have_larger_area_returns_1_for_worst_case() {
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
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 0,
                    height: 0,
                },
            },
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 10,
                    height: 10,
                },
            },
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 10,
                    height: 10,
                },
            },
        ];
        assert_eq!(
            HigherWindowsShouldHaveLargerArea::new(container, windows.len()).evaluate(&windows),
            1.0
        )
    }

    #[test]
    fn higher_windows_should_have_larger_area_returns_0_for_best_case() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let windows = [
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 10,
                    height: 10,
                },
            },
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 10,
                    height: 5,
                },
            },
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 0,
                    height: 0,
                },
            },
        ];
        assert_eq!(
            HigherWindowsShouldHaveLargerArea::new(container, windows.len()).evaluate(&windows),
            0.0
        )
    }

    #[proptest]
    fn windows_should_have_minimum_size_returns_values_in_range_0_1(
        #[strategy(
            ContainedWindows::arbitrary()
                .prop_flat_map(|x| {
                    (
                        (0..=x.container.width, 0..=x.container.height)
                            .prop_map(|(width, height)| Size { width, height }),
                        Just(x)
                    )
                })
        )]
        x: (Size, ContainedWindows),
    ) {
        prop_assert!((0.0..=1.0).contains(
            &WindowsShouldHaveMinimumSize::new(x.0, x.1.windows.len()).evaluate(&x.1.windows)
        ))
    }

    #[proptest]
    fn windows_should_have_minimum_size_returns_1_for_worst_case(
        size: Size,
        #[strategy((1_usize..=16))] count: usize,
    ) {
        assert_eq!(
            WindowsShouldHaveMinimumSize::new(size, count).evaluate(
                &repeat(Window {
                    pos: Pos { x: 0, y: 0 },
                    size: Size {
                        width: 0,
                        height: 0,
                    }
                })
                .take(count)
                .collect_vec()
            ),
            1.0
        )
    }

    #[test]
    fn windows_should_have_minimum_size_returns_0_for_best_case() {
        let size = Size {
            width: 5,
            height: 5,
        };
        let windows = [
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 10,
                    height: 10,
                },
            },
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 5,
                    height: 5,
                },
            },
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 10,
                    height: 5,
                },
            },
        ];
        assert_eq!(
            WindowsShouldHaveMinimumSize::new(size, windows.len()).evaluate(&windows),
            0.0
        )
    }
    #[proptest]
    fn windows_near_in_stack_should_be_close_returns_values_in_range_0_1(x: ContainedWindows) {
        prop_assert!((0.0..=1.0).contains(
            &WindowsNearInStackShouldBeClose::new(x.container, x.windows.len())
                .evaluate(&x.windows)
        ))
    }

    #[test]
    fn windows_near_in_stack_should_be_close_returns_1_for_worst_case() {
        // Worst case is windows with zero size alternating opposite corners.
        let container = Size {
            width: 10,
            height: 10,
        };
        let windows = [
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 0,
                    height: 0,
                },
            },
            Window {
                pos: Pos { x: 10, y: 10 },
                size: Size {
                    width: 0,
                    height: 0,
                },
            },
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 0,
                    height: 0,
                },
            },
        ];
        assert_eq!(
            WindowsNearInStackShouldBeClose::new(container, windows.len()).evaluate(&windows),
            1.0
        )
    }

    #[test]
    fn windows_near_in_stack_should_be_close_returns_0_for_best_case() {
        let container = Size {
            width: 10,
            height: 10,
        };
        let windows = [
            Window {
                pos: Pos { x: 0, y: 0 },
                size: Size {
                    width: 5,
                    height: 5,
                },
            },
            Window {
                pos: Pos { x: 0, y: 5 },
                size: Size {
                    width: 5,
                    height: 5,
                },
            },
            Window {
                pos: Pos { x: 5, y: 5 },
                size: Size {
                    width: 5,
                    height: 5,
                },
            },
        ];
        assert_eq!(
            WindowsNearInStackShouldBeClose::new(container, windows.len()).evaluate(&windows),
            0.0
        )
    }

    impl Arbitrary for Size {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (1_usize..=5120, 1_usize..=2160)
                .prop_map(|(width, height)| Size { width, height })
                .boxed()
        }
    }

    #[derive(Debug, Clone)]
    struct ContainedWindows {
        container: Size,
        windows: Vec<Window>,
    }

    impl Arbitrary for ContainedWindows {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (Size::arbitrary(), 0_usize..=16)
                .prop_map(|(size, count)| Decoder::new(16, size, count))
                .prop_flat_map(|decoder| {
                    vec(bool::arbitrary(), decoder.bits()).prop_map(move |bits| ContainedWindows {
                        windows: decoder.decode1(Array::from_vec(bits).view()).into_raw_vec(),
                        container: decoder.container(),
                    })
                })
                .boxed()
        }
    }
}
