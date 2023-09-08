use std::num::NonZeroUsize;

use itertools::Itertools;
use ndarray::prelude::*;

use crate::rect::{RangeExclusive, Rect, Size};

pub fn trim_outside(container: Size, mut rects: ArrayViewMut1<Rect>) {
    for rect in rects.iter_mut() {
        rect.size.width =
            NonZeroUsize::new(rect.width().get().min(container.width.get() - rect.x()))
                .unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) });
        rect.size.height =
            NonZeroUsize::new(rect.height().get().min(container.height.get() - rect.y()))
                .unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) });
    }
}

pub fn remove_gaps(max_size: Size, container: Size, mut rects: ArrayViewMut1<Rect>) {
    debug_assert!(max_size.width <= container.width);
    debug_assert!(max_size.height <= container.height);

    let flip_flop = |dist, x: usize, y: usize| {
        let x_ = x.min(div_ceil(dist, 2));
        let y = y.min(dist - x_);
        let x = x.min(dist - y);
        (x, y)
    };

    let mut freedoms = rects
        .iter()
        .map(|rect| Freedoms {
            left: rect.left(),
            right: container.width.get().saturating_sub(rect.right()),
            top: rect.top(),
            bottom: container.height.get().saturating_sub(rect.bottom()),
        })
        .collect_vec();
    loop {
        // These bounds may overestimate.
        // However,
        // for the algorithm to work
        // they only need to not underestimate,
        // as long as they are accurate
        // when freedom is zero.
        let x_rays = rects
            .iter()
            .zip(freedoms.iter())
            .map(|(rect, freedoms)| {
                let y_range = rect.y_range_exclusive();
                let max_free = max_size.width.get().saturating_sub(rect.width().get());
                let left = if freedoms.left == 0 {
                    rect.left()
                } else {
                    rects
                        .iter()
                        .filter(|other| {
                            other.right() < rect.left()
                                && y_range.intersects(other.y_range_exclusive())
                        })
                        .map(|other| other.right())
                        .max()
                        .unwrap_or(0)
                        .max(rect.left().saturating_sub(max_free))
                };
                let right = if freedoms.right == 0 {
                    rect.right()
                } else {
                    rects
                        .iter()
                        .filter(|other| {
                            rect.right() < other.left()
                                && y_range.intersects(other.y_range_exclusive())
                        })
                        .map(|other| other.left())
                        .min()
                        .unwrap_or(container.width.get())
                        .min(rect.right() + max_free)
                };
                RangeExclusive(left, right)
            })
            .collect_vec();
        let y_rays = rects
            .iter()
            .zip(freedoms.iter())
            .map(|(rect, freedoms)| {
                let x_range = rect.x_range_exclusive();
                let max_free = max_size.height.get().saturating_sub(rect.height().get());
                let top = if freedoms.top == 0 {
                    rect.top()
                } else {
                    rects
                        .iter()
                        .filter(|other| {
                            other.bottom() < rect.top()
                                && x_range.intersects(other.x_range_exclusive())
                        })
                        .map(|other| other.bottom())
                        .max()
                        .unwrap_or(0)
                        .max(rect.top().saturating_sub(max_free))
                };
                let bottom = if freedoms.bottom == 0 {
                    rect.bottom()
                } else {
                    rects
                        .iter()
                        .filter(|other| {
                            rect.bottom() < other.top()
                                && x_range.intersects(other.x_range_exclusive())
                        })
                        .map(|other| other.top())
                        .min()
                        .unwrap_or(container.height.get())
                        .min(rect.bottom() + max_free)
                };
                RangeExclusive(top, bottom)
            })
            .collect_vec();

        for (rect, freedoms) in rects.iter().zip(freedoms.iter_mut()) {
            let (left, right) = flip_flop(
                max_size.width.get().saturating_sub(rect.width().get()),
                rect.left(),
                container.width.get().saturating_sub(rect.right()),
            );
            freedoms.left = left;
            freedoms.right = right;
            let (top, bottom) = flip_flop(
                max_size.height.get().saturating_sub(rect.height().get()),
                rect.top(),
                container.height.get().saturating_sub(rect.bottom()),
            );
            freedoms.top = top;
            freedoms.bottom = bottom;
        }
        for (((i, rect), (x_ray, y_ray)), ((other_i, other_rect), (other_x_ray, other_y_ray))) in
            rects
                .iter()
                .enumerate()
                .zip(x_rays.iter().zip(y_rays.iter()))
                .tuple_combinations()
        {
            let x_range = rect.x_range_exclusive();
            let other_x_range = other_rect.x_range_exclusive();
            let y_range = rect.y_range_exclusive();
            let other_y_range = other_rect.y_range_exclusive();

            if y_ray.intersects(*other_y_ray) {
                let y_intersects = y_range.intersects(other_y_range);
                if y_intersects {
                    if other_x_range.contains(rect.left()) {
                        freedoms.get_mut(i).unwrap().left = 0;
                    }
                    if x_range.contains(other_rect.right()) {
                        freedoms.get_mut(other_i).unwrap().right = 0;
                    }
                    if other_x_range.contains(rect.right()) {
                        freedoms.get_mut(i).unwrap().right = 0;
                    }
                    if x_range.contains(other_rect.left()) {
                        freedoms.get_mut(other_i).unwrap().left = 0;
                    }
                }
                if other_rect.right() <= rect.left() {
                    let dist = rect.left() - other_rect.right();
                    if dist > 0 || y_intersects {
                        let (left, right) =
                            flip_flop(dist, freedoms[i].left, freedoms[other_i].right);
                        freedoms.get_mut(i).unwrap().left = left;
                        freedoms.get_mut(other_i).unwrap().right = right;
                    }
                }
                if rect.right() <= other_rect.left() {
                    let dist = other_rect.left() - rect.right();
                    if dist > 0 || y_intersects {
                        let (right, left) =
                            flip_flop(dist, freedoms[i].right, freedoms[other_i].left);
                        freedoms.get_mut(i).unwrap().right = right;
                        freedoms.get_mut(other_i).unwrap().left = left;
                    }
                }
            }

            if x_ray.intersects(*other_x_ray) {
                let x_intersects = x_range.intersects(other_x_range);
                if x_intersects {
                    if other_y_range.contains(rect.top()) {
                        freedoms.get_mut(i).unwrap().top = 0;
                    }
                    if y_range.contains(other_rect.bottom()) {
                        freedoms.get_mut(other_i).unwrap().bottom = 0;
                    }
                    if other_y_range.contains(rect.bottom()) {
                        freedoms.get_mut(i).unwrap().bottom = 0;
                    }
                    if y_range.contains(other_rect.top()) {
                        freedoms.get_mut(other_i).unwrap().top = 0;
                    }
                }
                if other_rect.bottom() <= rect.top() {
                    let dist = rect.top() - other_rect.bottom();
                    if dist > 0 || x_intersects {
                        let (top, bottom) =
                            flip_flop(dist, freedoms[i].top, freedoms[other_i].bottom);
                        freedoms.get_mut(i).unwrap().top = top;
                        freedoms.get_mut(other_i).unwrap().bottom = bottom;
                    }
                }
                if rect.bottom() <= other_rect.top() {
                    let dist = other_rect.top() - rect.bottom();
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
                for (rect, freedoms) in rects.iter_mut().zip(freedoms.iter()) {
                    rect.expand_left(freedoms.left.min(largest_safe_step));
                    rect.expand_right(freedoms.right.min(largest_safe_step));
                    rect.expand_top(freedoms.top.min(largest_safe_step));
                    rect.expand_bottom(freedoms.bottom.min(largest_safe_step));
                }
            }
            None => break,
        }
    }
}

pub fn overlap_borders(border_thickness: usize, container: Size, mut rects: ArrayViewMut1<Rect>) {
    let border_thickness_half_ceil = div_ceil(border_thickness, 2);
    let border_thickness_half = border_thickness / 2;

    let filter_map = |i,
                      other_i,
                      range: RangeExclusive<usize>,
                      other_range: RangeExclusive<usize>,
                      left,
                      right| {
        if i != other_i && range.intersects(other_range) && left >= right {
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

    let borders = rects
        .iter()
        .enumerate()
        .map(|(i, rect)| {
            let x_range = rect.x_range_exclusive();
            let y_range = rect.y_range_exclusive();
            Sides {
                left: {
                    rects
                        .iter()
                        .enumerate()
                        .filter_map(|(other_i, other_rect)| {
                            filter_map(
                                i,
                                other_i,
                                y_range,
                                other_rect.y_range_exclusive(),
                                rect.left(),
                                other_rect.right(),
                            )
                        })
                        .min()
                        .and_then(filter_out_of_range)
                },
                right: {
                    rects
                        .iter()
                        .enumerate()
                        .filter_map(|(other_i, other_rect)| {
                            filter_map(
                                i,
                                other_i,
                                y_range,
                                other_rect.y_range_exclusive(),
                                other_rect.left(),
                                rect.right(),
                            )
                        })
                        .min()
                        .and_then(filter_out_of_range)
                },
                top: {
                    rects
                        .iter()
                        .enumerate()
                        .filter_map(|(other_i, other_rect)| {
                            filter_map(
                                i,
                                other_i,
                                x_range,
                                other_rect.x_range_exclusive(),
                                rect.top(),
                                other_rect.bottom(),
                            )
                        })
                        .min()
                        .and_then(filter_out_of_range)
                },
                bottom: {
                    rects
                        .iter()
                        .enumerate()
                        .filter_map(|(other_i, other_rect)| {
                            filter_map(
                                i,
                                other_i,
                                x_range,
                                other_rect.x_range_exclusive(),
                                other_rect.top(),
                                rect.bottom(),
                            )
                        })
                        .min()
                        .and_then(filter_out_of_range)
                },
            }
        })
        .collect_vec();

    for ((i, rect), borders) in rects.iter_mut().enumerate().zip(borders.iter()) {
        match borders.left {
            Some(other_i) => {
                if i < other_i {
                    rect.expand_left(border_thickness_half_ceil);
                } else {
                    rect.expand_left(border_thickness_half);
                }
            }
            None => {
                rect.expand_left(border_thickness.min(rect.left()));
            }
        }
        match borders.right {
            Some(other_i) => {
                if i < other_i {
                    rect.expand_right(border_thickness_half_ceil);
                } else {
                    rect.expand_right(border_thickness_half);
                }
            }
            None => {
                rect.expand_right(
                    border_thickness.min(container.width.get().saturating_sub(rect.right())),
                );
            }
        }
        match borders.top {
            Some(other_i) => {
                if i < other_i {
                    rect.expand_top(border_thickness_half_ceil);
                } else {
                    rect.expand_top(border_thickness_half);
                }
            }
            None => {
                rect.expand_top(border_thickness.min(rect.top()));
            }
        }
        match borders.bottom {
            Some(other_i) => {
                if i < other_i {
                    rect.expand_bottom(border_thickness_half_ceil);
                } else {
                    rect.expand_bottom(border_thickness_half);
                }
            }
            None => {
                rect.expand_bottom(
                    border_thickness.min(container.height.get().saturating_sub(rect.bottom())),
                );
            }
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

    use crate::{
        rect::{covered_area, obscured_area},
        testing::{ContainedRects, NumRectsRange},
    };

    use super::*;

    #[test]
    fn remove_gaps_expands_at_same_rate() {
        let container = Size::new_checked(10, 10);
        let mut rects = arr1(&[
            Rect::new_checked(2, 2, 6, 1),
            Rect::new_checked(2, 7, 1, 1),
            Rect::new_checked(7, 7, 1, 1),
        ]);
        remove_gaps(container, container, rects.view_mut());
        assert_eq!(
            rects,
            arr1(&[
                Rect::new_checked(0, 0, 10, 5),
                Rect::new_checked(0, 5, 5, 5),
                Rect::new_checked(5, 5, 5, 5),
            ]),
        )
    }

    #[ignore = "fails when corners touch or rectangles overlap"]
    // Four or more rectangles can be in an arrangement
    // requiring overlapping
    // to remove gaps,
    //
    // ```
    // aab
    // d b
    // dcc
    // ```
    #[proptest]
    fn remove_gaps_with_no_max_size_and_1_to_3_rects_covers_container(
        #[strategy(ContainedRects::arbitrary_with(NumRectsRange(1, 3)))] args: ContainedRects,
    ) {
        let mut rects = Array::from(args.rects);
        remove_gaps(args.container, args.container, rects.view_mut());
        prop_assert_eq!(
            covered_area(rects.as_slice().unwrap()),
            args.container.area()
        )
    }

    #[ignore = "fails when corners touch"]
    // See above comment about four or more rects.
    #[proptest(max_global_rejects = 65536)]
    fn remove_gaps_with_1_to_3_rects_no_max_size_and_no_overlap_tiles_container(
        #[strategy(ContainedRects::arbitrary_with(NumRectsRange(1, 3)))] args: ContainedRects,
    ) {
        prop_assume!(obscured_area(&args.rects) == 0);
        let mut rects = Array::from(args.rects);
        remove_gaps(args.container, args.container, rects.view_mut());
        prop_assert_eq!(
            rects.into_iter().map(|x| x.area().get()).sum::<usize>(),
            args.container.area().get()
        )
    }

    #[ignore = "occasionally fails"]
    #[proptest(max_global_rejects = 65536)]
    fn remove_gaps_does_not_make_rects_overlap_if_they_did_not_already(args: RemoveGapsArgs) {
        prop_assume!(obscured_area(&args.rects) == 0);
        let mut rects = Array::from(args.rects);
        remove_gaps(args.max_size, args.container, rects.view_mut());
        prop_assert_eq!(obscured_area(rects.as_slice().unwrap()), 0)
    }

    #[proptest]
    fn remove_gaps_respects_max_size(args: RemoveGapsArgs) {
        let mut rects = Array::from(args.rects);
        remove_gaps(args.max_size, args.container, rects.view_mut());
        for rect in rects {
            prop_assert!(rect.width() <= args.max_size.width);
            prop_assert!(rect.height() <= args.max_size.height);
        }
    }

    #[proptest]
    fn overlap_borders_does_not_expand_past_container(
        #[strategy(1_usize..=32)] border_thickness: usize,
        container: Size,
    ) {
        let init = [Rect::new(0, 0, container.width, container.height)];
        let mut rects = arr1(&init);
        overlap_borders(border_thickness, container, rects.view_mut());
        assert_eq!(rects.into_raw_vec(), init)
    }

    #[test]
    fn overlap_borders_expands_evenly() {
        let container = Size::new_checked(10, 10);
        let mut rects = arr1(&[
            Rect::new_checked(0, 0, 10, 5),
            Rect::new_checked(0, 5, 5, 5),
            Rect::new_checked(5, 5, 5, 5),
        ]);
        overlap_borders(2, container, rects.view_mut());
        assert_eq!(
            rects,
            arr1(&[
                Rect::new_checked(0, 0, 10, 6),
                Rect::new_checked(0, 4, 6, 6),
                Rect::new_checked(4, 4, 6, 6),
            ]),
        )
    }

    #[test]
    fn overlap_borders_breaks_ties_in_favor_of_first() {
        let container = Size::new_checked(10, 10);
        let mut rects = arr1(&[
            Rect::new_checked(0, 0, 10, 5),
            Rect::new_checked(0, 5, 5, 5),
            Rect::new_checked(5, 5, 5, 5),
        ]);
        overlap_borders(1, container, rects.view_mut());
        assert_eq!(
            rects,
            arr1(&[
                Rect::new_checked(0, 0, 10, 6),
                Rect::new_checked(0, 5, 6, 5),
                Rect::new_checked(5, 5, 5, 5),
            ]),
        )
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

    #[derive(Clone, Debug)]
    struct RemoveGapsArgs {
        max_size: Size,
        container: Size,
        rects: Vec<Rect>,
    }

    impl Arbitrary for RemoveGapsArgs {
        type Parameters = NumRectsRange;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(range: Self::Parameters) -> Self::Strategy {
            ContainedRects::arbitrary_with(range)
                .prop_flat_map(|x| {
                    (
                        (
                            x.rects.iter().map(|x| x.width().get()).max().unwrap_or(1)
                                ..=x.container.width.get(),
                            x.rects.iter().map(|x| x.height().get()).max().unwrap_or(1)
                                ..=x.container.height.get(),
                        )
                            .prop_map(|(width, height)| Size::new_checked(width, height)),
                        Just(x),
                    )
                })
                .prop_map(|(max_size, x)| Self {
                    max_size,
                    container: x.container,
                    rects: x.rects,
                })
                .boxed()
        }
    }
}
