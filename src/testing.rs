use proptest::prelude::{prop::collection::vec, *};

use crate::{types::RangeExclusive, Size, Window};

#[derive(Debug, Clone)]
pub struct ContainedWindows {
    pub container: Size,
    pub windows: Vec<Window>,
}

pub struct NumWindowsRange(pub usize, pub usize);

impl Default for NumWindowsRange {
    fn default() -> Self {
        Self(0, 16)
    }
}

impl Arbitrary for ContainedWindows {
    type Parameters = NumWindowsRange;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(range: Self::Parameters) -> Self::Strategy {
        (Size::arbitrary(), range.0..=range.1)
            .prop_flat_map(|(container, count)| {
                vec(
                    (0..container.width, 0..container.height).prop_flat_map(move |(x, y)| {
                        (1..=container.width - x, 1..=container.height - y)
                            .prop_map(move |(width, height)| Window::new(x, y, width, height))
                    }),
                    count,
                )
                .prop_map(move |windows| ContainedWindows { container, windows })
            })
            .boxed()
    }
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
