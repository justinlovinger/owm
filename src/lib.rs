mod binary;
mod encoding;
mod objective;

use encoding::Decoder;
use ndarray::Axis;
use objective::{evaluate, Size, Window};
use optimal::{optimizer::derivative_free::pbil::*, prelude::*};

pub fn layout(width: usize, height: usize, count: usize) -> impl Iterator<Item = Window> {
    let decoder = Decoder::new(16, width, height);
    let decoder = Decoder::new(16, width, height, count);
    let size = Size {
        width: decoder.width(),
        height: decoder.height(),
    };
    decoder
        .decode1(
            UntilConvergedConfig::default()
                .argmin(&mut Config::start_default_for(decoder.bits(), |points| {
                    decoder.decode(points).map_axis(Axis(1), |windows| {
                        evaluate(size, windows.as_slice().unwrap())
                    })
                }))
                .view(),
        )
        .into_raw_vec()
}
