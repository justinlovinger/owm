use ndarray::prelude::*;

use crate::{
    binary::reversed_bits_to_fracs,
    post_processing::trim_off_screen,
    types::{Pos, Size, Window},
};

#[derive(Clone, Debug)]
pub struct Decoder {
    bits_per_num: usize,
    container: Size,
    num_windows: usize,
}

impl Decoder {
    pub fn new(bits_per_num: usize, container: Size, num_windows: usize) -> Self {
        Self {
            bits_per_num,
            container,
            num_windows,
        }
    }

    pub fn bits(&self) -> usize {
        4 * self.bits_per_num * self.num_windows
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
        let mut windows = reversed_bits_to_fracs(
            [
                0.0..=(self.container.width - 1) as f64,
                0.0..=(self.container.height - 1) as f64,
                1.0..=self.container.width as f64,
                1.0..=self.container.height as f64,
            ],
            bits.into_shape((
                bits.nrows(),
                bits.ncols() / (4 * self.bits_per_num),
                4,
                self.bits_per_num,
            ))
            .unwrap(),
        )
        .map_axis(Axis(2), |xs| Window {
            pos: Pos {
                x: xs[0].round() as usize,
                y: xs[1].round() as usize,
            },
            size: Size {
                width: xs[2].round() as usize,
                height: xs[3].round() as usize,
            },
        });
        for mut windows in windows.axis_iter_mut(Axis(0)) {
            trim_off_screen(self.container, windows.view_mut());
        }
        windows
    }
}
