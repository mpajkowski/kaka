#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Rect {
    const MAX_AREA: u16 = u16::MAX;

    pub fn new(x: u16, y: u16, mut width: u16, mut height: u16) -> Self {
        let width_u32 = width as u32;
        let height_u32 = height as u32;

        if width_u32 * height_u32 > Self::MAX_AREA as u32 {
            let aspect_ratio = width as f32 / height as f32;
            let max_area = Self::MAX_AREA as f32;

            let height_f = (max_area / aspect_ratio).sqrt();
            let width_f = max_area / height_f;

            width = width_f as u16;
            height = height_f as u16;
        }

        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn width(self) -> u16 {
        self.width
    }

    pub const fn height(self) -> u16 {
        self.height
    }

    pub const fn x(self) -> u16 {
        self.x
    }

    pub const fn y(self) -> u16 {
        self.y
    }

    pub const fn left(self) -> u16 {
        self.x
    }

    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub const fn top(self) -> u16 {
        self.y
    }

    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub const fn area(self) -> u16 {
        self.width * self.height
    }
}
