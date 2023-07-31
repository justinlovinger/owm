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
}

impl Size {
    fn area(&self) -> usize {
        self.width * self.height
    }
}

pub fn evaluate(usable_size: Size, windows: &[Window]) -> f64 {
    higher_windows_should_have_larger_area(usable_size, windows)
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
