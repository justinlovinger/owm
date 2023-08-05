use std::ops::RangeInclusive;

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
    pub fn left(&self) -> usize {
        self.pos.x
    }

    pub fn right(&self) -> usize {
        self.pos.x + self.size.width
    }

    pub fn top(&self) -> usize {
        self.pos.y
    }

    pub fn bottom(&self) -> usize {
        self.pos.y + self.size.height
    }

    pub fn expand_left(&mut self, value: usize) {
        self.pos.x -= value;
        self.size.width += value;
    }

    pub fn expand_right(&mut self, value: usize) {
        self.size.width += value;
    }

    pub fn expand_top(&mut self, value: usize) {
        self.pos.y -= value;
        self.size.height += value;
    }

    pub fn expand_bottom(&mut self, value: usize) {
        self.size.height += value;
    }

    pub fn center_x(&self) -> usize {
        self.left() + self.size.width / 2
    }

    pub fn center_y(&self) -> usize {
        self.top() + self.size.height / 2
    }

    pub fn x_range(&self) -> RangeInclusive<usize> {
        self.left()..=self.right()
    }

    pub fn y_range(&self) -> RangeInclusive<usize> {
        self.top()..=self.bottom()
    }

    pub fn area(&self) -> usize {
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
    pub fn area(&self) -> usize {
        self.width * self.height
    }
}
