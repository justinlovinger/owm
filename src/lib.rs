mod binary;
mod derive;
mod encoding;
mod objective;
mod post_processing;
mod rect;

#[cfg(test)]
mod testing;

use encoding::Decoder;
use optimal::{optimizer::derivative_free::pbil::*, prelude::*};
use post_processing::overlap_borders;
use rand::prelude::*;
use rand_xoshiro::SplitMix64;
use rayon::prelude::*;

use crate::objective::Problem;
pub use crate::{
    objective::{AreaRatio, AspectRatio, Weight, Weights},
    rect::{Pos, Rect, Size},
};

#[derive(Clone, Debug)]
pub struct LayoutGen {
    min_width: usize,
    min_height: usize,
    max_width: Option<usize>,
    max_height: Option<usize>,
    overlap_borders_by: usize,
    weights: Weights,
    area_ratios: Vec<AreaRatio>,
    aspect_ratios: Vec<AspectRatio>,
}

impl LayoutGen {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        min_width: usize,
        min_height: usize,
        max_width: Option<usize>,
        max_height: Option<usize>,
        overlap_borders_by: usize,
        weights: Weights,
        area_ratios: Vec<AreaRatio>,
        aspect_ratios: Vec<AspectRatio>,
    ) -> Self {
        Self {
            min_width,
            min_height,
            max_width,
            max_height,
            overlap_borders_by,
            weights,
            area_ratios,
            aspect_ratios,
        }
    }

    pub fn layout(&self, container: Size, count: usize) -> Vec<Rect> {
        let max_size = Size::new(
            self.max_width
                .map_or(container.width, |x| x.min(container.width)),
            self.max_height
                .map_or(container.height, |x| x.min(container.height)),
        );
        let decoder = Decoder::new(
            Size::new(
                self.min_width.min(container.width),
                self.min_height.min(container.height),
            ),
            max_size,
            container,
            count,
        );
        let problem = Problem::new(
            self.weights,
            self.area_ratios.clone(),
            self.aspect_ratios.clone(),
            max_size,
            container,
            count,
        );
        let mut rects = decoder.decode1(
            UntilConvergedConfig {
                threshold: ProbabilityThreshold::new(Probability::new(0.9).unwrap()).unwrap(),
            }
            .argmin(
                &mut Config {
                    num_samples: NumSamples::new(
                        500 * std::thread::available_parallelism().map_or(1, |x| x.into()),
                    )
                    .unwrap(),
                    adjust_rate: AdjustRate::new(0.1).unwrap(),
                    mutation_chance: MutationChance::new(0.0).unwrap(),
                    mutation_adjust_rate: MutationAdjustRate::new(0.05).unwrap(),
                }
                .start_using(
                    decoder.bits(),
                    |points| {
                        (0..points.nrows())
                            .into_par_iter()
                            .map(|i| {
                                problem.evaluate(decoder.decode1(points.row(i)).as_slice().unwrap())
                            })
                            .collect::<Vec<_>>()
                            .into()
                    },
                    &mut SplitMix64::seed_from_u64(0),
                ),
            )
            .view(),
        );
        if self.overlap_borders_by > 0 {
            overlap_borders(self.overlap_borders_by, container, rects.view_mut());
        }
        rects.into_raw_vec()
    }
}
