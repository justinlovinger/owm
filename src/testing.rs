use ndarray::prelude::*;
use proptest::prelude::{prop::collection::vec, *};

use crate::{encoding::Decoder, Size, Window};

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
pub struct ContainedWindows {
    pub container: Size,
    pub windows: Vec<Window>,
}

impl Arbitrary for ContainedWindows {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (Size::arbitrary(), 0_usize..=16)
            .prop_map(|(size, count)| Decoder::new(size, count))
            .prop_flat_map(|decoder| {
                vec(bool::arbitrary(), decoder.bits()).prop_map(move |bits| ContainedWindows {
                    windows: decoder.decode1(Array::from_vec(bits).view()).into_raw_vec(),
                    container: decoder.container(),
                })
            })
            .boxed()
    }
}
