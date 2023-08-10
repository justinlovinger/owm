use std::ops::{Range, RangeInclusive};

use ndarray::prelude::*;

use crate::{
    binary::reversed_bits_to_frac,
    post_processing::trim_off_screen,
    types::{Pos, Size, Window},
};

#[derive(Clone, Debug)]
pub struct Decoder {
    container: Size,
    num_windows: usize,
    x_range: RangeInclusive<f64>,
    y_range: RangeInclusive<f64>,
    width_range: RangeInclusive<f64>,
    height_range: RangeInclusive<f64>,
    x_bits_range: Range<usize>,
    y_bits_range: Range<usize>,
    width_bits_range: Range<usize>,
    height_bits_range: Range<usize>,
}

impl Decoder {
    pub fn new(container: Size, num_windows: usize) -> Self {
        let x_max = container.width.saturating_sub(1);
        let y_max = container.height.saturating_sub(1);
        let bits_per_x = bits_for(x_max);
        let bits_per_y = bits_for(y_max);
        let bits_per_width = bits_for(container.width);
        let bits_per_height = bits_for(container.height);
        Self {
            container,
            num_windows,
            x_range: 0.0..=(container.width - 1) as f64,
            y_range: 0.0..=(container.height - 1) as f64,
            width_range: 1.0..=container.width as f64,
            height_range: 1.0..=container.height as f64,
            x_bits_range: 0..bits_per_x,
            y_bits_range: bits_per_x..(bits_per_x + bits_per_y),
            width_bits_range: (bits_per_x + bits_per_y)..(bits_per_x + bits_per_y + bits_per_width),
            height_bits_range: (bits_per_x + bits_per_y + bits_per_width)
                ..(bits_per_x + bits_per_y + bits_per_width + bits_per_height),
        }
    }

    pub fn bits(&self) -> usize {
        self.bits_per_window() * self.num_windows
    }

    fn bits_per_window(&self) -> usize {
        self.height_bits_range.end
    }

    #[cfg(test)]
    pub fn container(&self) -> Size {
        self.container
    }

    pub fn decode1(&self, bits: ArrayView1<bool>) -> Array1<Window> {
        Array::from_vec(
            self.decode(bits.into_shape((1, bits.len())).unwrap())
                .into_raw_vec(),
        )
    }

    pub fn decode(&self, bits: ArrayView2<bool>) -> Array2<Window> {
        let mut windows = bits
            .into_shape((
                bits.nrows(),
                bits.ncols() / self.bits_per_window(),
                self.bits_per_window(),
            ))
            .unwrap()
            .map_axis(Axis(2), |xs| Window {
                pos: Pos {
                    x: reversed_bits_to_frac(
                        self.x_range.clone(),
                        xs.slice(s![self.x_bits_range.clone()]),
                    )
                    .into_scalar() as usize,
                    y: reversed_bits_to_frac(
                        self.y_range.clone(),
                        xs.slice(s![self.y_bits_range.clone()]),
                    )
                    .into_scalar() as usize,
                },
                size: Size {
                    width: reversed_bits_to_frac(
                        self.width_range.clone(),
                        xs.slice(s![self.width_bits_range.clone()]),
                    )
                    .into_scalar() as usize,
                    height: reversed_bits_to_frac(
                        self.height_range.clone(),
                        xs.slice(s![self.height_bits_range.clone()]),
                    )
                    .into_scalar() as usize,
                },
            });
        for mut windows in windows.axis_iter_mut(Axis(0)) {
            trim_off_screen(self.container, windows.view_mut());
        }
        windows
    }
}

fn bits_for(x: usize) -> usize {
    if x == 0 {
        0
    } else if x == 1 {
        1
    } else {
        (x - 1).ilog2() as usize + 1
    }
}
