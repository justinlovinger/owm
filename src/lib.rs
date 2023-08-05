mod binary;
mod encoding;
mod objective;
mod post_processing;
mod types;

use cached::proc_macro::cached;
use encoding::Decoder;
use ndarray::Axis;
use optimal::{optimizer::derivative_free::pbil::*, prelude::*};

use crate::{
    objective::evaluate,
    post_processing::{overlap_borders, remove_gaps},
    types::{Size, Window},
};

#[cached(sync_writes = true)]
pub fn layout(width: usize, height: usize, count: usize) -> Vec<Window> {
    let container = Size { width, height };
    let decoder = Decoder::new(16, container, count);
    let mut windows = decoder.decode1(
        UntilConvergedConfig::default()
            .argmin(&mut Config::start_default_for(decoder.bits(), |points| {
                decoder.decode(points).map_axis(Axis(1), |windows| {
                    evaluate(container, windows.as_slice().unwrap())
                })
            }))
            .view(),
    );
    remove_gaps(container, windows.view_mut());
    overlap_borders(1, container, windows.view_mut());
    windows.into_raw_vec()
}
