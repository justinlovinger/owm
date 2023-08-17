mod binary;
mod derive;
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
pub use crate::{
    objective::{Ratio, Weight, Weights},
    rect::{Pos, Rect, Size},
};

#[derive(Clone, Debug)]
pub struct LayoutGen {
    min_width: usize,
    min_height: usize,
    max_width: Option<usize>,
    max_height: Option<usize>,
    weights: Weights,
    area_ratios: Vec<Ratio>,
}

impl LayoutGen {
    pub fn new(
        min_width: usize,
        min_height: usize,
        max_width: Option<usize>,
        max_height: Option<usize>,
        weights: Weights,
        area_ratios: Vec<Ratio>,
    ) -> Self {
        Self {
            min_width,
            min_height,
            max_width,
            max_height,
            weights,
            area_ratios,
        }
    }

    pub fn layout(&self, container: Size, count: usize) -> Vec<Rect> {
        let decoder = Decoder::new(
            Size::new(
                self.min_width.min(container.width),
                self.min_height.min(container.height),
            ),
            Size::new(
                self.max_width
                    .map_or(container.width, |x| x.min(container.width)),
                self.max_height
                    .map_or(container.height, |x| x.min(container.height)),
            ),
            container,
            count,
        );
        let problem = Problem::new(self.weights, self.area_ratios.clone(), container, count);
        let mut rects = decoder.decode1(
            UntilConvergedConfig {
                threshold: ProbabilityThreshold::new(Probability::new(0.9).unwrap()).unwrap(),
            }
            .argmin(
                &mut Config {
                    num_samples: NumSamples::new(500).unwrap(),
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
}
