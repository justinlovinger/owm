use std::{borrow::Borrow, ops::RangeInclusive};

use itertools::Itertools;
use ndarray::prelude::*;

use crate::types::{Size, Window};

pub fn trim_off_screen(container: Size, mut windows: ArrayViewMut1<Window>) {
    for window in windows.iter_mut() {
        window.size.width = window.size.width.min(container.width - window.pos.x);
        window.size.height = window.size.height.min(container.height - window.pos.y);
    }
}

pub fn remove_gaps(max_size: Size, container: Size, mut windows: ArrayViewMut1<Window>) {
    debug_assert!(max_size.width <= container.width);
    debug_assert!(max_size.height <= container.height);

    let flip_flop = |dist, x: usize, y: usize| {
        let x_ = x.min(div_ceil(dist, 2));
        let y = y.min(dist - x_);
        let x = x.min(dist - y);
        (x, y)
    };

    let mut freedoms = windows
        .iter()
        .map(|window| Freedoms {
            left: window.left(),
            right: container.width.saturating_sub(window.right()),
            top: window.top(),
            bottom: container.height.saturating_sub(window.bottom()),
        })
        .collect_vec();
    loop {
        // These bounds may overestimate.
        // However,
        // for the algorithm to work
        // they only need to not underestimate,
        // as long as they are accurate
        // when freedom is zero.
        let x_rays = windows
            .iter()
            .zip(freedoms.iter())
            .map(|(window, freedoms)| {
                let y_range = window.y_range();
                let max_free = max_size.width.saturating_sub(window.size.width);
                let left = if freedoms.left == 0 {
                    window.left()
                } else {
                    windows
                        .iter()
                        .filter(|other| {
                            other.right() < window.left() && intersects(&y_range, other.y_range())
                        })
                        .map(|other| other.right())
                        .max()
                        .map_or(0, |x| x + 1)
                        .max(window.left().saturating_sub(max_free))
                };
                let right = if freedoms.right == 0 {
                    window.right()
                } else {
                    windows
                        .iter()
                        .filter(|other| {
                            window.right() < other.left() && intersects(&y_range, other.y_range())
                        })
                        .map(|other| other.left())
                        .min()
                        .map_or(container.width, |x| x - 1)
                        .min(window.right() + max_free)
                };
                left..=right
            })
            .collect_vec();
        let y_rays = windows
            .iter()
            .zip(freedoms.iter())
            .map(|(window, freedoms)| {
                let x_range = window.x_range();
                let max_free = max_size.height.saturating_sub(window.size.height);
                let top = if freedoms.top == 0 {
                    window.top()
                } else {
                    windows
                        .iter()
                        .filter(|other| {
                            other.bottom() < window.top() && intersects(&x_range, other.x_range())
                        })
                        .map(|other| other.bottom())
                        .max()
                        .map_or(0, |x| x + 1)
                        .max(window.top().saturating_sub(max_free))
                };
                let bottom = if freedoms.bottom == 0 {
                    window.bottom()
                } else {
                    windows
                        .iter()
                        .filter(|other| {
                            window.bottom() < other.top() && intersects(&x_range, other.x_range())
                        })
                        .map(|other| other.top())
                        .min()
                        .map_or(container.height, |x| x - 1)
                        .min(window.bottom() + max_free)
                };
                top..=bottom
            })
            .collect_vec();

        for (window, freedoms) in windows.iter().zip(freedoms.iter_mut()) {
            let (left, right) = flip_flop(
                max_size.width.saturating_sub(window.size.width),
                window.left(),
                container.width.saturating_sub(window.right()),
            );
            freedoms.left = left;
            freedoms.right = right;
            let (top, bottom) = flip_flop(
                max_size.height.saturating_sub(window.size.height),
                window.top(),
                container.height.saturating_sub(window.bottom()),
            );
            freedoms.top = top;
            freedoms.bottom = bottom;
        }
        for (
            ((i, window), (x_ray, y_ray)),
            ((other_i, other_window), (other_x_ray, other_y_ray)),
        ) in windows
            .iter()
            .enumerate()
            .zip(x_rays.iter().zip(y_rays.iter()))
            .tuple_combinations()
        {
            let x_range = window.x_range();
            let other_x_range = other_window.x_range();
            let y_range = window.y_range();
            let other_y_range = other_window.y_range();

            if intersects(y_ray, other_y_ray) {
                let y_intersects = intersects(&y_range, &other_y_range);
                if y_intersects {
                    if other_x_range.contains(&window.left()) {
                        freedoms.get_mut(i).unwrap().left = 0;
                    }
                    if x_range.contains(&other_window.right()) {
                        freedoms.get_mut(other_i).unwrap().right = 0;
                    }
                    if other_x_range.contains(&window.right()) {
                        freedoms.get_mut(i).unwrap().right = 0;
                    }
                    if x_range.contains(&other_window.left()) {
                        freedoms.get_mut(other_i).unwrap().left = 0;
                    }
                }
                if other_window.right() < window.left() {
                    let dist = window.left() - other_window.right() - 1;
                    if dist > 0 || y_intersects {
                        let (left, right) =
                            flip_flop(dist, freedoms[i].left, freedoms[other_i].right);
                        freedoms.get_mut(i).unwrap().left = left;
                        freedoms.get_mut(other_i).unwrap().right = right;
                    }
                }
                if window.right() < other_window.left() {
                    let dist = other_window.left() - window.right() - 1;
                    if dist > 0 || y_intersects {
                        let (right, left) =
                            flip_flop(dist, freedoms[i].right, freedoms[other_i].left);
                        freedoms.get_mut(i).unwrap().right = right;
                        freedoms.get_mut(other_i).unwrap().left = left;
                    }
                }
            }

            if intersects(x_ray, other_x_ray) {
                let x_intersects = intersects(&x_range, &other_x_range);
                if x_intersects {
                    if other_y_range.contains(&window.top()) {
                        freedoms.get_mut(i).unwrap().top = 0;
                    }
                    if y_range.contains(&other_window.bottom()) {
                        freedoms.get_mut(other_i).unwrap().bottom = 0;
                    }
                    if other_y_range.contains(&window.bottom()) {
                        freedoms.get_mut(i).unwrap().bottom = 0;
                    }
                    if y_range.contains(&other_window.top()) {
                        freedoms.get_mut(other_i).unwrap().top = 0;
                    }
                }
                if other_window.bottom() < window.top() {
                    let dist = window.top() - other_window.bottom() - 1;
                    if dist > 0 || x_intersects {
                        let (top, bottom) =
                            flip_flop(dist, freedoms[i].top, freedoms[other_i].bottom);
                        freedoms.get_mut(i).unwrap().top = top;
                        freedoms.get_mut(other_i).unwrap().bottom = bottom;
                    }
                }
                if window.bottom() < other_window.top() {
                    let dist = other_window.top() - window.bottom() - 1;
                    if dist > 0 || x_intersects {
                        let (bottom, top) =
                            flip_flop(dist, freedoms[i].bottom, freedoms[other_i].top);
                        freedoms.get_mut(i).unwrap().bottom = bottom;
                        freedoms.get_mut(other_i).unwrap().top = top;
                    }
                }
            }
        }

        let largest_safe_step = freedoms
            .iter()
            .flat_map(|freedoms| freedoms.flatten())
            .filter(|x| x > &0)
            .min();
        match largest_safe_step {
            Some(largest_safe_step) => {
                for (window, freedoms) in windows.iter_mut().zip(freedoms.iter()) {
                    window.expand_left(freedoms.left.min(largest_safe_step));
                    window.expand_right(freedoms.right.min(largest_safe_step));
                    window.expand_top(freedoms.top.min(largest_safe_step));
                    window.expand_bottom(freedoms.bottom.min(largest_safe_step));
                }
            }
            None => break,
        }
    }
}

pub fn overlap_borders(
    border_thickness: usize,
    container: Size,
    mut windows: ArrayViewMut1<Window>,
) {
    let border_thickness_ceil = div_ceil(border_thickness, 2);

    let filter_map = |i, other_i, range, other_range, left, right| {
        if i != other_i && intersects(range, other_range) && left > right {
            Some((left - right, other_i))
        } else {
            None
        }
    };

    let filter_out_of_range = |(x, i)| {
        if x <= border_thickness {
            Some(i)
        } else {
            None
        }
    };

    let borders = windows
        .iter()
        .enumerate()
        .map(|(i, window)| {
            let x_range = window.x_range();
            let y_range = window.y_range();
            Sides {
                left: {
                    windows
                        .iter()
                        .enumerate()
                        .filter_map(|(other_i, other_window)| {
                            filter_map(
                                i,
                                other_i,
                                y_range.clone(),
                                other_window.y_range(),
                                window.left(),
                                other_window.right(),
                            )
                        })
                        .min()
                        .and_then(filter_out_of_range)
                },
                right: {
                    windows
                        .iter()
                        .enumerate()
                        .filter_map(|(other_i, other_window)| {
                            filter_map(
                                i,
                                other_i,
                                y_range.clone(),
                                other_window.y_range(),
                                other_window.left(),
                                window.right(),
                            )
                        })
                        .min()
                        .and_then(filter_out_of_range)
                },
                top: {
                    windows
                        .iter()
                        .enumerate()
                        .filter_map(|(other_i, other_window)| {
                            filter_map(
                                i,
                                other_i,
                                x_range.clone(),
                                other_window.x_range(),
                                window.top(),
                                other_window.bottom(),
                            )
                        })
                        .min()
                        .and_then(filter_out_of_range)
                },
                bottom: {
                    windows
                        .iter()
                        .enumerate()
                        .filter_map(|(other_i, other_window)| {
                            filter_map(
                                i,
                                other_i,
                                x_range.clone(),
                                other_window.x_range(),
                                other_window.top(),
                                window.bottom(),
                            )
                        })
                        .min()
                        .and_then(filter_out_of_range)
                },
            }
        })
        .collect_vec();

    for ((i, window), borders) in windows.iter_mut().enumerate().zip(borders.iter()) {
        match borders.left {
            Some(other_i) => {
                if i < other_i {
                    window.expand_left(border_thickness_ceil);
                }
            }
            None => {
                window.expand_left(border_thickness.min(window.left()));
            }
        }
        match borders.right {
            Some(other_i) => {
                if i < other_i {
                    window.expand_right(border_thickness_ceil);
                }
            }
            None => {
                window.expand_right(
                    border_thickness.min(container.width.saturating_sub(window.right())),
                );
            }
        }
        match borders.top {
            Some(other_i) => {
                if i < other_i {
                    window.expand_top(border_thickness_ceil);
                }
            }
            None => {
                window.expand_top(border_thickness.min(window.top()));
            }
        }
        match borders.bottom {
            Some(other_i) => {
                if i < other_i {
                    window.expand_bottom(border_thickness_ceil);
                }
            }
            None => {
                window.expand_bottom(
                    border_thickness.min(container.height.saturating_sub(window.bottom())),
                );
            }
        }
    }

    let borders = borders
        .into_iter()
        .enumerate()
        .map(|(i, borders)| Sides {
            left: borders.left.and_then(|other_i| {
                if i > other_i {
                    Some(windows[other_i].right())
                } else {
                    None
                }
            }),
            right: borders.right.and_then(|other_i| {
                if i > other_i {
                    Some(windows[other_i].left())
                } else {
                    None
                }
            }),
            top: borders.top.and_then(|other_i| {
                if i > other_i {
                    Some(windows[other_i].bottom())
                } else {
                    None
                }
            }),
            bottom: borders.bottom.and_then(|other_i| {
                if i > other_i {
                    Some(windows[other_i].top())
                } else {
                    None
                }
            }),
        })
        .collect_vec();
    for (window, borders) in windows.iter_mut().zip(borders) {
        if let Some(right) = borders.left {
            window.expand_left(window.left() - right);
        }
        if let Some(left) = borders.right {
            window.expand_right(left - window.right());
        }
        if let Some(bottom) = borders.top {
            window.expand_top(window.top() - bottom);
        }
        if let Some(top) = borders.bottom {
            window.expand_bottom(top - window.bottom());
        }
    }
}

type Freedoms = Sides<usize>;

#[derive(Clone, Copy, Debug)]
struct Sides<T> {
    left: T,
    right: T,
    top: T,
    bottom: T,
}

impl<T> Sides<T> {
    fn flatten(self) -> [T; 4] {
        [self.left, self.right, self.top, self.bottom]
    }
}

fn intersects<T, A, B>(x: A, y: B) -> bool
where
    A: Borrow<RangeInclusive<T>>,
    B: Borrow<RangeInclusive<T>>,
    T: std::cmp::PartialOrd,
{
    contains_either(x.borrow(), y.borrow()) || contains_either(y.borrow(), x.borrow())
}

fn contains_either<T, A, B>(x: A, y: B) -> bool
where
    A: Borrow<RangeInclusive<T>>,
    B: Borrow<RangeInclusive<T>>,
    T: std::cmp::PartialOrd,
{
    let x = x.borrow();
    let y = y.borrow();
    x.contains(y.start()) || x.contains(y.end())
}

fn div_ceil(x: usize, y: usize) -> usize {
    if x % y > 0 {
        x / y + 1
    } else {
        x / y
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::testing::ContainedWindows;

    use super::*;

    #[proptest]
    fn remove_gaps_respects_max_size(
        #[strategy(
            ContainedWindows::arbitrary().prop_flat_map(|x| {
                (
                    (
                        x.windows.iter().map(|x| x.size.width).max().unwrap_or(0)..=x.container.width,
                        x.windows.iter().map(|x| x.size.height).max().unwrap_or(0)..=x.container.height,
                    )
                        .prop_map(|(width, height)| Size::new(width, height)),
                    Just(x)
                )
            })
        )]
        params: (Size, ContainedWindows),
    ) {
        let max_size = params.0;
        let mut windows = Array::from(params.1.windows);
        remove_gaps(max_size, params.1.container, windows.view_mut());
        for window in windows {
            prop_assert!(window.size.width <= max_size.width);
            prop_assert!(window.size.height <= max_size.height);
        }
    }

    #[test]
    fn intersects_works_for_simple_cases() {
        assert!(intersects(0..=10, 5..=15));
        assert!(!intersects(0..=4, 5..=15));
    }

    #[proptest]
    fn intersects_is_symmetrical(x: RangeInclusive<usize>, y: RangeInclusive<usize>) {
        prop_assert_eq!(intersects(&x, &y), intersects(&y, &x));
    }

    #[test]
    fn div_ceil_works_for_simple_cases() {
        assert_eq!(div_ceil(11, 2), 6);
        assert_eq!(div_ceil(3, 2), 2);
        assert_eq!(div_ceil(1, 2), 1);
        assert_eq!(div_ceil(0, 2), 0);
        assert_eq!(div_ceil(2, 2), 1);
        assert_eq!(div_ceil(4, 2), 2);
        assert_eq!(div_ceil(4, 3), 2);
        assert_eq!(div_ceil(5, 3), 2);
        assert_eq!(div_ceil(6, 3), 2);
    }
}
