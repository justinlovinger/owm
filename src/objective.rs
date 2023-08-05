use itertools::Itertools;

use crate::types::{Size, Window};

pub fn evaluate(container: Size, windows: &[Window]) -> f64 {
    minimize_overlapping(container, windows)
        + higher_windows_should_have_larger_area(container, windows)
}

fn minimize_overlapping(container: Size, windows: &[Window]) -> f64 {
    let max_overlap = container.area() as f64;
    windows
        .iter()
        .tuple_combinations()
        .map(|(window, other)| window.overlap(other) as f64 / max_overlap)
        .sum()
}

fn higher_windows_should_have_larger_area(container: Size, windows: &[Window]) -> f64 {
    let max_area = container.area() as f64;
    windows
        .iter()
        .map(|w| w.area() as f64 / max_area)
        .tuple_windows()
        .map(|(x, y)| {
            let diff = x - y;
            diff.powi(2).copysign(diff)
        })
        .sum::<f64>()
}
