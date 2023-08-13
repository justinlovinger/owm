mod binary;
mod encoding;
mod objective;
mod post_processing;
mod types;

#[cfg(test)]
mod testing;

use encoding::Decoder;
use ndarray::Axis;
use optimal::{optimizer::derivative_free::pbil::*, prelude::*};

use crate::objective::Problem;
pub use crate::types::{Pos, Size, Window};

pub fn layout(width: usize, height: usize, count: usize) -> Vec<Window> {
    let container = Size { width, height };
    let max_size = Size::new(1920.min(container.width), container.height);
    let decoder = Decoder::new(
        Size::new(320.min(container.width), 180.min(container.height)),
        max_size,
        container,
        count,
    );
    let problem = Problem::new(container, count);
    decoder
        .decode1(
            UntilConvergedConfig {
                threshold: ProbabilityThreshold::new(Probability::new(0.9).unwrap()).unwrap(),
            }
            .argmin(
                &mut Config {
                    num_samples: NumSamples::new(200).unwrap(),
                    adjust_rate: AdjustRate::new(0.1).unwrap(),
                    mutation_chance: MutationChance::new(0.0).unwrap(),
                    mutation_adjust_rate: MutationAdjustRate::new(0.05).unwrap(),
                }
                .start(decoder.bits(), |points| {
                    decoder.decode2(points).map_axis(Axis(1), |windows| {
                        problem.evaluate(windows.as_slice().unwrap())
                    })
                }),
            )
            .view(),
        )
        .into_raw_vec()
}
