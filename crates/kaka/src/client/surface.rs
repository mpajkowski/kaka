use std::ops::{Index, IndexMut};

use kaka_core::shapes::{Point, Rect};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::style::{Color, Modifier, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn resize(&mut self, area: Rect) {
        if self.area != area {
            self.area = area;
            self.content.resize(area.area() as usize, Cell::default());
            self.reset();
        }
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
            cell.reset();
        }
    }

    /// Print at most the first n characters of a string if enough space is available
    /// until the end of the line
    pub fn set_stringn(&mut self, pos: Point, string: impl AsRef<str>, width: usize, style: Style) {
        let mut idx = self.index_of(pos);
        let mut x_offset = pos.x as usize;

        let graphemes = UnicodeSegmentation::graphemes(string.as_ref(), true);

        let max_offset = (self.area.right() as usize).min(width.saturating_add(pos.x as usize));

        for s in graphemes {
            let width = s.width();

            if width == 0 {
                continue;
            }

            if width > max_offset.saturating_sub(x_offset) {
                break;
            }

            self.content[idx].set_symbol(s);
            self.content[idx].set_style(style);

            for i in idx + 1..idx + width {
                self.content[i].reset();
            }

            idx += width;
            x_offset += width;
        }

        Point::new(x_offset as u16, pos.y);
    }

    pub fn index_of(&self, pos: Point) -> usize {
        debug_assert!(
            pos.x >= self.area.left()
                && pos.x < self.area.right()
                && pos.y >= self.area.top()
                && pos.y < self.area.bottom(),
            "Trying to access position outside the buffer: x={}, y={}, area={:?}",
            pos.x,
            pos.y,
            self.area
        );

        ((pos.y - self.area.y) * self.area.width + (pos.x - self.area.x)) as usize
    }

    pub fn diff<'a>(&'a self, other: &'a Self) -> Diff<'a> {
        let previous_buffer = &self.content;
        let next_buffer = &other.content;
        let width = self.area.width;

        debug_assert_eq!(width, other.area.width);

        Diff::new(previous_buffer, next_buffer, width)
    }
}

pub struct Diff<'a> {
    width: u16,
    previous_buffer: &'a [Cell],
    next_buffer: &'a [Cell],
    invalidated: usize,
    to_skip: usize,
    done: usize,
}

impl<'a> Diff<'a> {
    pub const fn new(previous_buffer: &'a [Cell], next_buffer: &'a [Cell], width: u16) -> Self {
        Self {
            width,
            previous_buffer,
            next_buffer,
            invalidated: 0,
            to_skip: 0,
            done: 0,
        }
    }
}

impl<'a> Iterator for Diff<'a> {
    type Item = (Point, &'a Cell);

    #[track_caller]
    fn next(&mut self) -> Option<Self::Item> {
        let mut update = None;

        for (i, (current, previous)) in self
            .next_buffer
            .iter()
            .zip(self.previous_buffer.iter())
            .enumerate()
        {
            if (current != previous || self.invalidated > 0) && self.to_skip == 0 {
                let position = self.done;
                let x = position as u16 % self.width;
                let y = position as u16 / self.width;
                update = Some((Point::new(x, y), &self.next_buffer[i]));
            }

            self.to_skip = current.symbol.width().saturating_sub(1);

            let affected_width = current.symbol.width().max(previous.symbol.width());
            //println!("affected_width: {affected_width:?}");
            self.invalidated = affected_width.max(self.invalidated).saturating_sub(1);

            self.done += 1;

            if update.is_some() {
                // pop checked elements from both slices and yield an update
                self.previous_buffer = &self.previous_buffer[i + 1..];
                self.next_buffer = &self.next_buffer[i + 1..];

                return update;
            }
        }

        None
    }
}

impl Index<usize> for Surface {
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.content[index]
    }
}

impl IndexMut<usize> for Surface {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.content[index]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn surface_10x10(sym: &str) -> Surface {
        let mut surface = Surface::empty(Rect::new(0, 0, 10, 10));

        let width = sym.width();

        for i in 0..100 {
            if i % width != 0 {
                continue;
            }

            surface.set_stringn(
                Point::new(i as u16 % 10, i as u16 / 10),
                sym,
                usize::MAX,
                Style::default(),
            );
        }

        surface
    }

    #[test]
    fn diff_iterator_full_buffer_changed() {
        let old = surface_10x10("a");
        let new = surface_10x10("b");

        let diff = Diff::new(&old.content, &new.content, old.area.area() as _);

        assert_eq!(diff.count(), 100);
    }

    #[test]
    fn diff_iterator_no_changes() {
        let old = surface_10x10("a");
        let new = surface_10x10("a");

        let diff = Diff::new(&old.content, &new.content, old.area.width as _);

        assert_eq!(diff.count(), 0);
    }

    #[test]
    fn diff_iterator_one_change_begin() {
        let old = surface_10x10("a");
        let mut new = surface_10x10("a");

        new.content[0] = Cell::default();

        let mut diff = Diff::new(&old.content, &new.content, old.area.width as _);

        assert_eq!(diff.next(), Some((Point::new(0, 0), &Cell::default())));
        assert_eq!(diff.next(), None);
    }

    #[test]
    fn diff_iterator_one_change_middle() {
        let old = surface_10x10("a");
        let mut new = surface_10x10("a");

        new.content[42] = Cell::default();

        let mut diff = Diff::new(&old.content, &new.content, old.area.width as _);

        assert_eq!(diff.next(), Some((Point::new(2, 4), &Cell::default())));
        assert_eq!(diff.next(), None);
    }

    #[test]
    fn diff_iterator_one_change_end() {
        let old = surface_10x10("a");
        let mut new = surface_10x10("a");
        new.content[99] = Cell::default();

        let mut diff = Diff::new(&old.content, &new.content, old.area.width as _);

        assert_eq!(diff.next(), Some((Point::new(9, 9), &Cell::default())));
        assert_eq!(diff.next(), None);
    }

    #[test]
    fn diff_iterator_one_change_long_symbol() {
        let old = surface_10x10("🚀");
        let new = surface_10x10("⛄");

        let diff = Diff::new(&old.content, &new.content, old.area.width as _);

        assert_eq!(diff.count(), 50);
    }
}
