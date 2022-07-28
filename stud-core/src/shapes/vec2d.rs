#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2d {
    x: u16,
    y: u16,
}

impl Vec2d {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.x
    }

    pub fn set_x(&mut self, x: u16) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: u16) {
        self.y = y;
    }
}

impl From<(u16, u16)> for Vec2d {
    fn from((x, y): (u16, u16)) -> Self {
        Self { x, y }
    }
}
