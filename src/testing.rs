use proptest::prelude::{prop::collection::vec, *};

use crate::{rect::RangeExclusive, Rect, Size};

#[derive(Debug, Clone)]
pub struct ContainedRects {
    pub container: Size,
    pub rects: Vec<Rect>,
}

pub struct NumRectsRange(pub usize, pub usize);

impl Default for NumRectsRange {
    fn default() -> Self {
        Self(0, 16)
    }
}

impl Arbitrary for ContainedRects {
    type Parameters = NumRectsRange;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(range: Self::Parameters) -> Self::Strategy {
        (Size::arbitrary(), range.0..=range.1)
            .prop_flat_map(|(container, count)| {
                vec(
                    (0..container.width.get(), 0..container.height.get()).prop_flat_map(
                        move |(x, y)| {
                            (
                                1..=container.width.get() - x,
                                1..=container.height.get() - y,
                            )
                                .prop_map(move |(width, height)| {
                                    Rect::new_checked(x, y, width, height)
                                })
                        },
                    ),
                    count,
                )
                .prop_map(move |rects| ContainedRects { container, rects })
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
