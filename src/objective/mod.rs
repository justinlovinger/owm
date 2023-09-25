mod adjacent_close;
mod area_ratios;
mod aspect_ratios;
mod center_main;
mod consistency;
mod gaps;
mod overlap;
mod reading_order;

use std::ops::Mul;

use derive_more::Display;
use num_traits::bounds::LowerBounded;

use crate::{
    derive::*,
    rect::{Rect, Size},
};

use self::{
    adjacent_close::PlaceAdjacentClose, area_ratios::MaintainAreaRatios,
    aspect_ratios::MaintainAspectRatios, center_main::CenterMain, consistency::MaximizeConsistency,
    gaps::MinimizeGaps, overlap::MinimizeOverlap, reading_order::PlaceInReadingOrder,
};
pub use self::{area_ratios::AreaRatio, aspect_ratios::AspectRatio};

pub struct Problem {
    weights: Weights,
    gaps: MinimizeGaps,
    overlap: MinimizeOverlap,
    area_ratios: MaintainAreaRatios,
    aspect_ratios: MaintainAspectRatios,
    adjacent_close: PlaceAdjacentClose,
    reading_order: PlaceInReadingOrder,
    center_main: CenterMain,
    consistency: MaximizeConsistency,
}

#[derive(Clone, Copy, Debug)]
pub struct Weights {
    pub gaps_weight: Weight,
    pub overlap_weight: Weight,
    pub area_ratios_weight: Weight,
    pub aspect_ratios_weight: Weight,
    pub adjacent_close_weight: Weight,
    pub reading_order_weight: Weight,
    pub center_main_weight: Weight,
    pub consistency_weight: Weight,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, PartialOrd)]
pub struct Weight(f64);

impl LowerBounded for Weight {
    fn min_value() -> Self {
        Self(0.0)
    }
}

derive_new_from_lower_bounded_float!(Weight(f64));
derive_try_from_from_new!(Weight(f64));
derive_from_str_from_try_into!(Weight(f64));

impl Mul<f64> for Weight {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        self.0 * rhs
    }
}

impl Problem {
    pub fn new(
        weights: Weights,
        area_ratios: Vec<AreaRatio>,
        aspect_ratios: Vec<AspectRatio>,
        max_size: Size,
        container: Size,
        prev_layout: Vec<Rect>,
    ) -> Self {
        let count = prev_layout.len() + 1;
        Self {
            weights,
            gaps: MinimizeGaps::new(container),
            overlap: MinimizeOverlap::new(container, count),
            area_ratios: MaintainAreaRatios::new(area_ratios, max_size, count),
            aspect_ratios: MaintainAspectRatios::new(aspect_ratios, max_size, count),
            adjacent_close: PlaceAdjacentClose::new(container, count),
            reading_order: PlaceInReadingOrder::new(count),
            center_main: CenterMain::new(container),
            consistency: MaximizeConsistency::new(container, prev_layout),
        }
    }

    pub fn evaluate(&self, rects: &[Rect]) -> f64 {
        (if self.weights.gaps_weight > Weight(0.0) {
            self.weights.gaps_weight * self.gaps.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.overlap_weight > Weight(0.0) {
            self.weights.overlap_weight * self.overlap.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.area_ratios_weight > Weight(0.0) {
            self.weights.area_ratios_weight * self.area_ratios.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.aspect_ratios_weight > Weight(0.0) {
            self.weights.aspect_ratios_weight * self.aspect_ratios.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.adjacent_close_weight > Weight(0.0) {
            self.weights.adjacent_close_weight * self.adjacent_close.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.reading_order_weight > Weight(0.0) {
            self.weights.reading_order_weight * self.reading_order.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.center_main_weight > Weight(0.0) {
            self.weights.center_main_weight * self.center_main.evaluate(rects)
        } else {
            0.0
        }) + (if self.weights.consistency_weight > Weight(0.0) {
            self.weights.consistency_weight * self.consistency.evaluate(rects)
        } else {
            0.0
        })
    }
}
