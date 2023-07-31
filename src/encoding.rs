use ndarray::prelude::*;

use crate::{
    binary::reversed_bits_to_fracs,
    objective::{Pos, Size, Window},
};

#[derive(Clone, Debug)]
pub struct Decoder {
    bits_per_num: usize,
    width: usize,
    height: usize,
}

impl Decoder {
    pub fn new(bits_per_num: usize, width: usize, height: usize) -> Self {
        Self {
            bits_per_num,
            width,
            height,
        }
    }

    pub fn bits_per_window(&self) -> usize {
        4 * self.bits_per_num()
    }

    pub fn bits_per_num(&self) -> usize {
        self.bits_per_num
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn decode1(&self, bits: ArrayView1<bool>) -> Array1<Window> {
        Array::from_vec(
            self.decode(bits.into_shape((1, bits.len())).unwrap())
                .into_raw_vec(),
        )
    }

    pub fn decode(&self, bits: ArrayView2<bool>) -> Array2<Window> {
        reversed_bits_to_fracs(
            [
                0.0..=(self.width - 1) as f64,
                0.0..=(self.height - 1) as f64,
                1.0..=self.width as f64,
                1.0..=self.height as f64,
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
        })
    }
}
