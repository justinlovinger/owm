use std::{num::NonZeroUsize, ops::RangeInclusive};

use proptest::prelude::{prop::collection::vec, *};

use crate::{rect::RangeExclusive, Rect, Size};

#[derive(Debug, Clone)]
pub struct ContainedRects {
    pub container: Size,
    pub rects: Vec<Rect>,
}

pub struct ContainedRectsParams {
    pub width_range: RangeInclusive<NonZeroUsize>,
    pub height_range: RangeInclusive<NonZeroUsize>,
    pub len_range: RangeInclusive<usize>,
}

impl Default for ContainedRectsParams {
    fn default() -> Self {
        Self {
            width_range: NonZeroUsize::new(1).unwrap()..=NonZeroUsize::new(5120).unwrap(),
            height_range: NonZeroUsize::new(1).unwrap()..=NonZeroUsize::new(2160).unwrap(),
            len_range: 0..=16,
        }
    }
}

impl ContainedRectsParams {
    pub fn from_len_range(range: RangeInclusive<usize>) -> Self {
        Self {
            len_range: range,
            ..Self::default()
        }
    }

    fn width_range_usize(&self) -> RangeInclusive<usize> {
        self.width_range.start().get()..=self.width_range.end().get()
    }

    fn height_range_usize(&self) -> RangeInclusive<usize> {
        self.height_range.start().get()..=self.height_range.end().get()
    }
}

impl Arbitrary for ContainedRects {
    type Parameters = ContainedRectsParams;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
        (
            params.width_range_usize(),
            params.height_range_usize(),
            params.len_range,
        )
            .prop_flat_map(|(width, height, count)| {
                vec(
                    (0..width, 0..height).prop_flat_map(move |(x, y)| {
                        (1..=width - x, 1..=height - y)
                            .prop_map(move |(width, height)| Rect::new_checked(x, y, width, height))
                    }),
                    count,
                )
                .prop_map(move |rects| ContainedRects {
                    container: Size::new_checked(width, height),
                    rects,
                })
            })
            .boxed()
    }
}

impl Arbitrary for Size {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (1_usize..=5120, 1_usize..=2160)
            .prop_map(|(width, height)| Size::new_checked(width, height))
            .boxed()
    }
}

impl<T> Arbitrary for RangeExclusive<T>
where
    T: Arbitrary,
    T::Parameters: Clone,
    T::Strategy: 'static,
{
    type Parameters = T::Parameters;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        (T::arbitrary_with(args.clone()), T::arbitrary_with(args))
            .prop_map(|(x, y)| RangeExclusive(x, y))
            .boxed()
    }
}
