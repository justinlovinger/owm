mod binary;
mod encoding;
mod objective;
mod post_processing;
mod types;

use encoding::Decoder;
use ndarray::Axis;
use optimal::{optimizer::derivative_free::pbil::*, prelude::*};

pub use crate::types::{Pos, Size, Window};
use crate::{
    objective::Problem,
    post_processing::{overlap_borders, remove_gaps},
};

pub fn layout(width: usize, height: usize, count: usize) -> Vec<Window> {
    let container = Size { width, height };
    let decoder = Decoder::new(container, count);
    let problem = Problem::new(container, count);
    let mut windows = decoder.decode1(
        UntilConvergedConfig {
            threshold: ProbabilityThreshold::new(Probability::new(0.9).unwrap()).unwrap(),
        }
        .argmin(
            &mut Config {
                num_samples: NumSamples::new(100).unwrap(),
                adjust_rate: AdjustRate::new(0.1).unwrap(),
                mutation_chance: MutationChance::new(0.0).unwrap(),
                mutation_adjust_rate: MutationAdjustRate::new(0.05).unwrap(),
            }
            .start(decoder.bits(), |points| {
                decoder.decode(points).map_axis(Axis(1), |windows| {
                    problem.evaluate(windows.as_slice().unwrap())
                })
            }),
        )
        .view(),
    );
    remove_gaps(container, windows.view_mut());
    overlap_borders(1, container, windows.view_mut());
    windows.into_raw_vec()
}
