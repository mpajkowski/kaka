use ropey::RopeSlice;

pub type SelectionInclusiveRange = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    anchor: usize,
    head: usize,
}

impl Selection {
    pub const fn new(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    pub const fn at_pos(pos: usize) -> Self {
        Self {
            anchor: pos,
            head: pos,
        }
    }

    pub const fn anchor(self) -> usize {
        self.anchor
    }

    pub const fn head(self) -> usize {
        self.head
    }

    pub fn start(self) -> usize {
        self.head.min(self.anchor)
    }

    pub fn end(self) -> usize {
        self.head.max(self.anchor)
    }

    pub const fn range(self) -> SelectionInclusiveRange {
        if self.anchor >= self.head {
            (self.head, self.anchor)
        } else {
            (self.anchor, self.head)
        }
    }

    pub fn slice(self, rope: RopeSlice<'_>) -> RopeSlice<'_> {
        let range = self.range();
        rope.slice(range.0..=range.1)
    }

    pub fn update_head(&mut self, pos: usize) {
        self.head = pos;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn start_end() {
        let head_after_anchor = Selection::new(0, 5);

        assert_eq!(head_after_anchor.start(), 0);
        assert_eq!(head_after_anchor.end(), 5);

        let head_before_anchor = Selection::new(5, 0);

        assert_eq!(head_before_anchor.start(), 0);
        assert_eq!(head_before_anchor.end(), 5);
    }

    #[test]
    fn range() {
        let head_after_anchor = Selection::new(0, 5);
        assert_eq!(head_after_anchor.range(), (0, 5));

        let head_before_anchor = Selection::new(5, 0);
        assert_eq!(head_before_anchor.range(), (0, 5));
    }

    #[test]
    fn update() {
        let mut selection = Selection::new(5, 5);
        assert_eq!(selection.range(), (5, 5));

        selection.update_head(6);
        assert_eq!(selection.range(), (5, 6));

        selection.update_head(4);
        assert_eq!(selection.range(), (4, 5));
    }
}
