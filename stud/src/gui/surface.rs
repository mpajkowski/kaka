use stud_core::shapes::{Point, Rect};
use unicode_width::UnicodeWidthStr;

use super::{style::Style, Color, Modifier};

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub symbol: String,
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
}

impl Cell {
    pub fn set_symbol(&mut self, sym: &str) -> &mut Self {
        self.symbol.clear();
        self.symbol.push_str(sym);
        self
    }

    pub fn set_fg(&mut self, fg: Color) -> &mut Self {
        self.fg = fg;
        self
    }

    pub fn set_bg(&mut self, bg: Color) -> &mut Self {
        self.bg = bg;
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        if let Some(fg) = style.fg {
            self.fg = fg;
        }

        if let Some(bg) = style.bg {
            self.bg = bg;
        }

        self.modifier.insert(style.add_modifier);
        self.modifier.remove(style.sub_modifier);

        self
    }

    pub fn style(&self) -> Style {
        Style::default()
            .fg(self.fg)
            .bg(self.bg)
            .add_modifier(self.modifier)
    }

    pub fn reset(&mut self) {
        self.symbol.clear();
        self.symbol.push(' ');
        self.fg = Color::Reset;
        self.bg = Color::Reset;
        self.modifier = Modifier::empty();
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: " ".to_string(),
            fg: Color::Reset,
            bg: Color::Reset,
            modifier: Modifier::empty(),
        }
    }
}

pub struct Surface {
    pub area: Rect,
    pub content: Vec<Cell>,
}

impl Surface {
    pub fn empty(area: Rect) -> Self {
        let cell = Cell::default();
        Self::filled(area, &cell)
    }

    pub fn filled(area: Rect, cell: &Cell) -> Self {
        let size = area.area() as usize;
        let mut content: Vec<Cell> = Vec::with_capacity(size);

        for _ in 0..size {
            content.push(cell.clone());
        }

        Self { area, content }
    }

    pub fn reset(&mut self) {
        for cell in &mut self.content {
            cell.reset()
        }
    }

    pub(crate) fn diff<'a>(&self, other: &'a Surface) -> Vec<(Point, &'a Cell)> {
        let previous_buffer = &self.content;
        let next_buffer = &other.content;
        let width = self.area.width();

        let mut updates: Vec<(Point, &Cell)> = vec![];
        // Cells invalidated by drawing/replacing preceeding multi-width characters:
        let mut invalidated: usize = 0;
        // Cells from the current buffer to skip due to preceeding multi-width characters taking their
        // place (the skipped cells should be blank anyway):
        let mut to_skip: usize = 0;
        for (i, (current, previous)) in next_buffer.iter().zip(previous_buffer.iter()).enumerate() {
            if (current != previous || invalidated > 0) && to_skip == 0 {
                let x = i as u16 % width;
                let y = i as u16 / width;
                updates.push((Point::new(x, y), &next_buffer[i]));
            }

            to_skip = current.symbol.width().saturating_sub(1);

            let affected_width = std::cmp::max(current.symbol.width(), previous.symbol.width());
            invalidated = std::cmp::max(affected_width, invalidated).saturating_sub(1);
        }

        updates
    }
}
