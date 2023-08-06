use itertools::Itertools;
use ndarray::Array;

use crate::types::{Size, Window};

pub fn evaluate(container: Size, windows: &[Window]) -> f64 {
    10.0 * minimize_overlapping(container, windows)
        + higher_windows_should_have_larger_area(container, windows)
        + 2.0
            * windows_should_have_minimum_size(
                Size {
                    width: 800,
                    height: 600,
                },
                windows,
            )
}

fn minimize_overlapping(container: Size, windows: &[Window]) -> f64 {
    if windows.len() < 2 {
        0.0
    } else {
        let max_overlap = container.area() as f64;
        windows
            .iter()
            .tuple_combinations()
            .map(|(window, other)| window.overlap(other) as f64 / max_overlap)
            .sum::<f64>()
            / windows.iter().tuple_combinations::<(_, _)>().count() as f64
    }
}

fn higher_windows_should_have_larger_area(container: Size, windows: &[Window]) -> f64 {
    if windows.len() < 2 {
        0.0
    } else {
        let max_area = container.area() as f64;
        let ideals = Array::linspace(1.0, 0.0, windows.len());
        windows
            .iter()
            .zip(ideals.iter())
            .map(|(window, ideal)| ((window.area() as f64 / max_area) - ideal).abs())
            .sum::<f64>()
            / ideals.into_iter().map(|x| x.max(1.0 - x)).sum::<f64>()
    }
}

fn windows_should_have_minimum_size(size: Size, windows: &[Window]) -> f64 {
    if windows.is_empty() {
        0.0
    } else {
        let width = size.width as f64;
        let height = size.height as f64;
        windows
            .iter()
            .map(|window| {
                (if size.width == 0 {
                    0.0
                } else {
                    size.width.saturating_sub(window.size.width) as f64 / width
                } + if size.height == 0 {
                    0.0
                } else {
                    size.height.saturating_sub(window.size.height) as f64 / height
                }) / 2.0
            })
            .sum::<f64>()
            / windows.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use std::iter::repeat;

    use ndarray::prelude::*;
    use proptest::{
        prelude::{prop::collection::vec, *},
        test_runner::FileFailurePersistence,
    };
    use test_strategy::proptest;

    use crate::{encoding::Decoder, types::Pos};

    use super::*;

    #[proptest(failure_persistence = Some(Box::new(FileFailurePersistence::Off)))]
    fn minimize_overlapping_returns_values_in_range_0_1(x: ProblemInstance) {
        prop_assert!((0.0..=1.0).contains(&minimize_overlapping(x.container, &x.windows)))
    }

    #[proptest(failure_persistence = Some(Box::new(FileFailurePersistence::Off)))]
    fn minimize_overlapping_returns_1_for_worst_case(
        container: Size,
        #[strategy((2_usize..=16))] count: usize,
    ) {
        prop_assert_eq!(
            minimize_overlapping(
                container,
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

    #[proptest(failure_persistence = Some(Box::new(FileFailurePersistence::Off)))]
    fn minimize_overlapping_returns_0_for_less_than_2_windows(
        container: Size,
        #[strategy((0_usize..=1))] count: usize,
    ) {
        prop_assert_eq!(
            minimize_overlapping(
                container,
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
        assert_eq!(
            minimize_overlapping(
                Size {
                    width: 10,
                    height: 10,
                },
                &[
                    Window {
                        pos: Pos { x: 0, y: 0 },
                        size: Size {
                            width: 10,
                            height: 5,
                        }
                    },
                    Window {
                        pos: Pos { x: 0, y: 5 },
                        size: Size {
                            width: 5,
                            height: 5,
                        }
                    },
                    Window {
                        pos: Pos { x: 5, y: 5 },
                        size: Size {
                            width: 5,
                            height: 5,
                        }
                    },
                ]
            ),
            0.0
        )
    }

    #[proptest(failure_persistence = Some(Box::new(FileFailurePersistence::Off)))]
    fn higher_windows_should_have_larger_area_returns_values_in_range_0_1(x: ProblemInstance) {
        prop_assert!(
            (0.0..=1.0).contains(&higher_windows_should_have_larger_area(
                x.container,
                &x.windows
            ))
        )
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
        assert_eq!(
            higher_windows_should_have_larger_area(
                Size {
                    width: 10,
                    height: 10,
                },
                &[
                    Window {
                        pos: Pos { x: 0, y: 0 },
                        size: Size {
                            width: 0,
                            height: 0,
                        }
                    },
                    Window {
                        pos: Pos { x: 0, y: 0 },
                        size: Size {
                            width: 10,
                            height: 10,
                        }
                    },
                    Window {
                        pos: Pos { x: 0, y: 0 },
                        size: Size {
                            width: 10,
                            height: 10,
                        }
                    },
                ]
            ),
            1.0
        )
    }

    #[test]
    fn higher_windows_should_have_larger_area_returns_0_for_best_case() {
        assert_eq!(
            higher_windows_should_have_larger_area(
                Size {
                    width: 10,
                    height: 10,
                },
                &[
                    Window {
                        pos: Pos { x: 0, y: 0 },
                        size: Size {
                            width: 10,
                            height: 10,
                        }
                    },
                    Window {
                        pos: Pos { x: 0, y: 0 },
                        size: Size {
                            width: 10,
                            height: 5,
                        }
                    },
                    Window {
                        pos: Pos { x: 0, y: 0 },
                        size: Size {
                            width: 0,
                            height: 0,
                        }
                    },
                ]
            ),
            0.0
        )
    }

    #[proptest(failure_persistence = Some(Box::new(FileFailurePersistence::Off)))]
    fn windows_should_have_minimum_size_returns_values_in_range_0_1(
        #[strategy(
            ProblemInstance::arbitrary()
                .prop_flat_map(|x| {
                    (
                        (0..=x.container.width, 0..=x.container.height)
                            .prop_map(|(width, height)| Size { width, height }),
                        Just(x)
                    )
                })
        )]
        x: (Size, ProblemInstance),
    ) {
        prop_assert!((0.0..=1.0).contains(&windows_should_have_minimum_size(x.0, &x.1.windows)))
    }

    #[proptest(failure_persistence = Some(Box::new(FileFailurePersistence::Off)))]
    fn windows_should_have_minimum_size_returns_1_for_worst_case(
        size: Size,
        #[strategy((1_usize..=16))] count: usize,
    ) {
        assert_eq!(
            windows_should_have_minimum_size(
                size,
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
        assert_eq!(
            windows_should_have_minimum_size(
                Size {
                    width: 5,
                    height: 5,
                },
                &[
                    Window {
                        pos: Pos { x: 0, y: 0 },
                        size: Size {
                            width: 10,
                            height: 10,
                        }
                    },
                    Window {
                        pos: Pos { x: 0, y: 0 },
                        size: Size {
                            width: 5,
                            height: 5,
                        }
                    },
                    Window {
                        pos: Pos { x: 0, y: 0 },
                        size: Size {
                            width: 10,
                            height: 5,
                        }
                    },
                ]
            ),
            0.0
        )
    }

    #[derive(Debug, Clone)]
    struct ProblemInstance {
        container: Size,
        windows: Vec<Window>,
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

    impl Arbitrary for ProblemInstance {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (Size::arbitrary(), 0_usize..=16)
                .prop_map(|(size, count)| Decoder::new(16, size, count))
                .prop_flat_map(|decoder| {
                    vec(bool::arbitrary(), decoder.bits()).prop_map(move |bits| ProblemInstance {
                        windows: decoder.decode1(Array::from_vec(bits).view()).into_raw_vec(),
                        container: decoder.container(),
                    })
                })
                .boxed()
        }
    }
}
