use itertools::Itertools;

#[derive(Clone, Copy, Debug)]
pub struct Window {
    pub pos: Pos,
    pub size: Size,
}

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl Window {
    fn area(&self) -> usize {
        self.size.area()
    }

    pub fn overlap(&self, other: &Window) -> usize {
        let x_overlap = self
            .right()
            .min(other.right())
            .saturating_sub(self.left().max(other.left()));
        let y_overlap = self
            .bottom()
            .min(other.bottom())
            .saturating_sub(self.top().max(other.top()));
        x_overlap * y_overlap
    }
}

impl Size {
    fn area(&self) -> usize {
        self.width * self.height
    }
}

pub fn evaluate(usable_size: Size, windows: &[Window]) -> f64 {
    minimize_overlapping(usable_size, windows)
        + higher_windows_should_have_larger_area(usable_size, windows)
}

fn minimize_overlapping(usable_size: Size, windows: &[Window]) -> f64 {
    let max_overlap = usable_size.area() as f64;
    windows
        .iter()
        .tuple_combinations()
        .map(|(window, other)| window.overlap(other) as f64 / max_overlap)
        .sum()
}

fn higher_windows_should_have_larger_area(usable_size: Size, windows: &[Window]) -> f64 {
    let max_area = usable_size.area() as f64;
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
