use std::ops::Range;

use ndarray::prelude::*;

use crate::{
    binary::ToFracLE,
    post_processing::{overlap_borders, remove_gaps, trim_off_screen},
    types::{Size, Window},
};

#[derive(Clone, Debug)]
pub struct Decoder {
    max_size: Size,
    container: Size,
    num_windows: usize,
    x_decoder: ToFracLE<f64>,
    y_decoder: ToFracLE<f64>,
    width_decoder: ToFracLE<f64>,
    height_decoder: ToFracLE<f64>,
    x_bits_range: Range<usize>,
    y_bits_range: Range<usize>,
    width_bits_range: Range<usize>,
    height_bits_range: Range<usize>,
}

impl Decoder {
    pub fn new(min_size: Size, max_size: Size, container: Size, num_windows: usize) -> Self {
        debug_assert!(min_size.width <= max_size.width);
        debug_assert!(min_size.height <= max_size.height);
        debug_assert!(max_size.width <= container.width);
        debug_assert!(max_size.height <= container.height);

        let x_max = container.width.saturating_sub(min_size.width);
        let y_max = container.height.saturating_sub(min_size.height);
        let width_range = min_size.width..=max_size.width;
        let height_range = min_size.height..=max_size.height;
        let bits_per_x = bits_for(x_max);
        let bits_per_y = bits_for(y_max);
        let bits_per_width = bits_for(width_range.end() - width_range.start());
        let bits_per_height = bits_for(height_range.end() - height_range.start());
        Self {
            max_size,
            container,
            num_windows,
            x_decoder: ToFracLE::new(0.0..=(x_max as f64), bits_per_x),
            y_decoder: ToFracLE::new(0.0..=(y_max as f64), bits_per_y),
            width_decoder: ToFracLE::new(
                (*width_range.start() as f64)..=(*width_range.end() as f64),
                bits_per_width,
            ),
            height_decoder: ToFracLE::new(
                (*height_range.start() as f64)..=(*height_range.end() as f64),
                bits_per_height,
            ),
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
            self.decode2(bits.into_shape((1, bits.len())).unwrap())
                .into_raw_vec(),
        )
    }

    pub fn decode2(&self, bits: ArrayView2<bool>) -> Array2<Window> {
        let mut windows = bits
            .into_shape((
                bits.nrows(),
                bits.ncols() / self.bits_per_window(),
                self.bits_per_window(),
            ))
            .unwrap()
            .map_axis(Axis(2), |xs| {
                Window::new(
                    self.x_decoder
                        .decode(xs.slice(s![self.x_bits_range.clone()]).into_iter().copied())
                        as usize,
                    self.y_decoder
                        .decode(xs.slice(s![self.y_bits_range.clone()]).into_iter().copied())
                        as usize,
                    self.width_decoder.decode(
                        xs.slice(s![self.width_bits_range.clone()])
                            .into_iter()
                            .copied(),
                    ) as usize,
                    self.height_decoder.decode(
                        xs.slice(s![self.height_bits_range.clone()])
                            .into_iter()
                            .copied(),
                    ) as usize,
                )
            });
        for mut windows in windows.axis_iter_mut(Axis(0)) {
            trim_off_screen(self.container, windows.view_mut());
            remove_gaps(self.max_size, self.container, windows.view_mut());
            overlap_borders(1, self.container, windows.view_mut());
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
