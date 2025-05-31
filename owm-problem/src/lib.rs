mod binary;
mod derive;
mod rect;

pub mod encoding;
pub mod objective;
pub mod post_processing;

#[cfg(test)]
pub mod testing;

pub use crate::{
    objective::{AreaRatio, AspectRatio, Weight, Weights},
    rect::{Pos, Rect, Size},
};
