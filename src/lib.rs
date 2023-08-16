mod binary;
mod encoding;
mod objective;
mod post_processing;
mod rect;

#[cfg(test)]
mod testing;

use encoding::Decoder;
use ndarray::Axis;
use optimal::{optimizer::derivative_free::pbil::*, prelude::*};
use post_processing::overlap_borders;
use rand::prelude::*;
use rand_xoshiro::SplitMix64;

use crate::objective::Problem;
pub use crate::rect::{Pos, Rect, Size};

pub fn layout(width: usize, height: usize, count: usize) -> Vec<Rect> {
    let container = Size { width, height };
    let max_size = Size::new(1920.min(container.width), container.height);
    let decoder = Decoder::new(
        Size::new(320.min(container.width), 180.min(container.height)),
        max_size,
        container,
        count,
    );
    let problem = Problem::new(container, count);
    let mut rects = decoder.decode1(
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
            .start_using(
                decoder.bits(),
                |points| {
                    decoder
                        .decode2(points)
                        .map_axis(Axis(1), |rects| problem.evaluate(rects.as_slice().unwrap()))
                },
                &mut SplitMix64::seed_from_u64(0),
            ),
        )
        .view(),
    );
    overlap_borders(1, container, rects.view_mut());
    rects.into_raw_vec()
}
