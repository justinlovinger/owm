use std::ops::RangeInclusive;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Window {
    pub pos: Pos,
    pub size: Size,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl Window {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            pos: Pos { x, y },
            size: Size { width, height },
        }
    }

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

    pub fn center(&self) -> Pos {
        Pos::new(self.center_x(), self.center_y())
    }

    pub fn center_x(&self) -> usize {
        self.left() + self.size.width / 2
    }

    pub fn center_y(&self) -> usize {
        self.top() + self.size.height / 2
    }

    pub fn top_left(&self) -> Pos {
        Pos {
            y: self.top(),
            x: self.left(),
        }
    }

    pub fn top_right(&self) -> Pos {
        Pos {
            y: self.top(),
            x: self.right(),
        }
    }

    pub fn bottom_left(&self) -> Pos {
        Pos {
            y: self.bottom(),
            x: self.left(),
        }
    }

    pub fn bottom_right(&self) -> Pos {
        Pos {
            y: self.bottom(),
            x: self.right(),
        }
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

    pub fn x_range(&self) -> RangeInclusive<usize> {
        self.left()..=self.right()
    }

    pub fn y_range(&self) -> RangeInclusive<usize> {
        self.top()..=self.bottom()
    }

    pub fn area(&self) -> usize {
        self.size.area()
    }

    pub fn overlap(&self, other: &Window) -> Option<Window> {
        let left = self.left().max(other.left());
        let right = self.right().min(other.right());
        let top = self.top().max(other.top());
        let bottom = self.bottom().min(other.bottom());

        if left < right && top < bottom {
            Some(Window {
                pos: Pos { x: left, y: top },
                size: Size {
                    width: right - left,
                    height: bottom - top,
                },
            })
        } else {
            None
        }
    }
}

impl Pos {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// Return manhattan distance between positions.
    pub fn dist(self, other: Pos) -> usize {
        (if self.x > other.x {
            self.x - other.x
        } else {
            other.x - self.x
        }) + (if self.y > other.y {
            self.y - other.y
        } else {
            other.y - self.y
        })
    }
}

impl Size {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn area(&self) -> usize {
        self.width * self.height
    }
}

impl From<Size> for Pos {
    fn from(value: Size) -> Self {
        Pos {
            x: value.width,
            y: value.height,
        }
    }
}
