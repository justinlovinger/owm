use std::{num::NonZeroUsize, ops::Range};

use ndarray::prelude::*;

use crate::{
    binary::ToFracLE,
    post_processing::{remove_gaps, trim_outside},
    rect::{Rect, Size},
};

#[derive(Clone, Debug)]
pub struct Decoder {
    max_size: Size,
    container: Size,
    count: usize,
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
    pub fn new(min_size: Size, max_size: Size, container: Size, count: usize) -> Self {
        debug_assert!(min_size.width <= max_size.width);
        debug_assert!(min_size.height <= max_size.height);
        debug_assert!(max_size.width <= container.width);
        debug_assert!(max_size.height <= container.height);

        let x_max = container.width.get().saturating_sub(min_size.width.get());
        let y_max = container.height.get().saturating_sub(min_size.height.get());
        let width_range = min_size.width.get()..=max_size.width.get();
        let height_range = min_size.height.get()..=max_size.height.get();
        let bits_per_x = reduced_bits_for(x_max);
        let bits_per_y = reduced_bits_for(y_max);
        let bits_per_width = reduced_bits_for(width_range.end() - width_range.start());
        let bits_per_height = reduced_bits_for(height_range.end() - height_range.start());
        Self {
            max_size,
            container,
            count,
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
        self.bits_per_rect() * self.count
    }

    fn bits_per_rect(&self) -> usize {
        self.height_bits_range.end
    }

    pub fn decode1(&self, bits: ArrayView1<bool>) -> Array1<Rect> {
        Array::from_vec(
            self.decode2(bits.into_shape((1, bits.len())).unwrap())
                .into_raw_vec(),
        )
    }

    pub fn decode2(&self, bits: ArrayView2<bool>) -> Array2<Rect> {
        let mut rects = bits
            .into_shape((bits.nrows(), self.count, self.bits_per_rect()))
            .unwrap()
            .map_axis(Axis(2), |xs| {
                let width = self.width_decoder.decode(
                    xs.slice(s![self.width_bits_range.clone()])
                        .into_iter()
                        .copied(),
                ) as usize;
                let height = self.height_decoder.decode(
                    xs.slice(s![self.height_bits_range.clone()])
                        .into_iter()
                        .copied(),
                ) as usize;
                Rect::new(
                    self.x_decoder
                        .decode(xs.slice(s![self.x_bits_range.clone()]).into_iter().copied())
                        as usize,
                    self.y_decoder
                        .decode(xs.slice(s![self.y_bits_range.clone()]).into_iter().copied())
                        as usize,
                    // The decoder should ensure these invariants.
                    unsafe { NonZeroUsize::new_unchecked(width) },
                    unsafe { NonZeroUsize::new_unchecked(height) },
                )
            });
        for mut rects in rects.axis_iter_mut(Axis(0)) {
            trim_outside(self.container, rects.view_mut());
            remove_gaps(self.max_size, self.container, rects.view_mut());
        }
        rects
    }
}

fn reduced_bits_for(x: usize) -> usize {
    // 128 was empirically chosen.
    // It has no special meaning,
    // other than being a power of 2.
    bits_for((x as f64 / 128.0).ceil() as usize)
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
