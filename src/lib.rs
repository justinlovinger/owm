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

use crate::objective::{Problem, Weights};
pub use crate::{
    objective::{Ratio, Weight},
    rect::{Pos, Rect, Size},
};

#[derive(Clone, Debug)]
pub struct LayoutGen {
    min_width: usize,
    min_height: usize,
    max_width: Option<usize>,
    max_height: Option<usize>,
    weights: Weights,
    area_ratio: Ratio,
}

#[derive(Clone, Debug)]
pub struct LayoutGenBuilder {
    min_width: Option<usize>,
    min_height: Option<usize>,
    max_width: Option<Option<usize>>,
    max_height: Option<Option<usize>>,
    gaps_weight: Option<Weight>,
    overlap_weight: Option<Weight>,
    area_ratio_weight: Option<Weight>,
    adjacent_close_weight: Option<Weight>,
    reading_order_weight: Option<Weight>,
    center_main_weight: Option<Weight>,
    area_ratio: Option<Ratio>,
}

impl LayoutGenBuilder {
    pub fn new() -> Self {
        Self {
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            gaps_weight: None,
            overlap_weight: None,
            area_ratio_weight: None,
            adjacent_close_weight: None,
            reading_order_weight: None,
            center_main_weight: None,
            area_ratio: None,
        }
    }

    pub fn min_width(mut self, value: usize) -> Self {
        self.min_width = Some(value);
        self
    }

    pub fn min_height(mut self, value: usize) -> Self {
        self.min_height = Some(value);
        self
    }

    pub fn max_width(mut self, value: Option<usize>) -> Self {
        self.max_width = Some(value);
        self
    }

    pub fn max_height(mut self, value: Option<usize>) -> Self {
        self.max_height = Some(value);
        self
    }

    pub fn gaps_weight(mut self, value: Weight) -> Self {
        self.gaps_weight = Some(value);
        self
    }

    pub fn overlap_weight(mut self, value: Weight) -> Self {
        self.overlap_weight = Some(value);
        self
    }

    pub fn area_ratio_weight(mut self, value: Weight) -> Self {
        self.area_ratio_weight = Some(value);
        self
    }

    pub fn adjacent_close_weight(mut self, value: Weight) -> Self {
        self.adjacent_close_weight = Some(value);
        self
    }

    pub fn reading_order_weight(mut self, value: Weight) -> Self {
        self.reading_order_weight = Some(value);
        self
    }

    pub fn center_main_weight(mut self, value: Weight) -> Self {
        self.center_main_weight = Some(value);
        self
    }

    pub fn area_ratio(mut self, value: Ratio) -> Self {
        self.area_ratio = Some(value);
        self
    }

    pub fn build(self) -> LayoutGen {
        LayoutGen {
            min_width: self.min_width.unwrap_or(320),
            min_height: self.min_height.unwrap_or(180),
            max_width: self.max_width.unwrap_or(Some(1920)),
            max_height: self.max_height.unwrap_or(None),
            weights: Weights {
                gaps_weight: self.gaps_weight.unwrap_or(Weight::new(3.0).unwrap()),
                overlap_weight: self.overlap_weight.unwrap_or(Weight::new(2.0).unwrap()),
                area_ratio_weight: self.area_ratio_weight.unwrap_or(Weight::new(1.5).unwrap()),
                adjacent_close_weight: self
                    .adjacent_close_weight
                    .unwrap_or(Weight::new(0.5).unwrap()),
                reading_order_weight: self
                    .reading_order_weight
                    .unwrap_or(Weight::new(0.5).unwrap()),
                center_main_weight: self.center_main_weight.unwrap_or(Weight::new(3.0).unwrap()),
            },
            area_ratio: self.area_ratio.unwrap_or(Ratio::new(2.0).unwrap()),
        }
    }
}

impl Default for LayoutGenBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutGen {
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
        let problem = Problem::new(self.weights, self.area_ratio, container, count);
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
}
