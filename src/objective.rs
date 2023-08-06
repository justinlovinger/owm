use itertools::Itertools;

use crate::types::{Size, Window};

pub fn evaluate(container: Size, windows: &[Window]) -> f64 {
    10.0 * minimize_overlapping(container, windows)
        + higher_windows_should_have_larger_area(container, windows)
        + 2.0
            * windows_should_have_minimum_size(
                Size {
                    width: 800,
                    height: 600,
                },
                windows,
            )
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
            let diff = y - x;
            diff.abs().sqrt().copysign(diff)
        })
        .sum::<f64>()
}

fn windows_should_have_minimum_size(size: Size, windows: &[Window]) -> f64 {
    windows
        .iter()
        .map(|window| {
            match (
                window.size.width >= size.width,
                window.size.height >= size.height,
            ) {
                (true, true) => 1.0,
                (true, false) => 0.5,
                (false, true) => 0.5,
                (false, false) => 0.0,
            }
        })
        .sum::<f64>()
        / windows.len() as f64
}
