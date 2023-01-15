use std::ops::Range;

use ropey::RopeSlice;

use crate::graphemes::next_grapheme_boundary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub kind: SpanKind,
    pub range: Range<usize>,
}

bitflags::bitflags! {
    pub struct SpanKind: u64 {
        const SELECTION = 1 << 0;
    }
}

pub struct SpanIterator<'a> {
    done: bool,
    cursor: usize,
    selection_idx: usize,
    rope: RopeSlice<'a>,
    selections: Vec<(usize, usize)>,
}

impl<'a> SpanIterator<'a> {
    pub fn new(rope: RopeSlice<'a>, selections: impl IntoIterator<Item = (usize, usize)>) -> Self {
        let mut selections = selections.into_iter().collect::<Vec<_>>();
        selections.sort_by_key(|(start, _end)| *start);

        Self {
            done: rope.len_chars() == 0,
            rope,
            cursor: 0,
            selection_idx: 0,
            selections,
        }
    }
}

impl<'a> Iterator for SpanIterator<'a> {
    type Item = Span;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // fast path - no selections
        if self.selections.is_empty() {
            self.done = true;

            return Some(Span {
                kind: SpanKind::empty(),
                range: (0..self.rope.len_chars()),
            });
        }

        let selection = &self.selections[self.selection_idx];

        let start = selection.0;
        // selections are inclusive
        let end = next_grapheme_boundary(self.rope, selection.1);

        let span = if self.cursor == start {
            let old_cursor = self.cursor;

            self.cursor = end;

            Span {
                kind: SpanKind::SELECTION,
                range: (old_cursor..self.cursor),
            }
        } else if self.cursor == end {
            // end of selection, move to next character
            let start = self.cursor;

            self.selection_idx += 1;

            let end = if self.selection_idx == self.selections.len() {
                // all selections exhausted
                self.done = true;

                self.rope.len_chars()
            } else {
                self.selections[self.selection_idx].0
            };

            self.cursor = end;

            Span {
                kind: SpanKind::empty(),
                range: (start..end),
            }
        } else if self.cursor == 0 {
            // we are at the beginning and *before* selection

            self.cursor = start;

            Span {
                kind: SpanKind::empty(),
                range: (0..start),
            }
        } else {
            unreachable!();
        };

        if span.range.is_empty() {
            self.done = true;
            None
        } else {
            Some(span)
        }
    }
}

#[cfg(test)]
mod test {
    use ropey::Rope;

    use crate::selection::Selection;

    use super::*;

    #[test]
    fn no_selections() {
        let rope = Rope::from_str("0123456789");
        let len = rope.len_chars();

        let mut iter = SpanIterator::new(rope.slice(..), vec![]);

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::empty(),
                range: (0..len)
            })
        );

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_selection() {
        let rope = Rope::from_str("0123456789");
        let len = rope.len_chars();
        let selection = Selection::new(1, 2);

        let mut iter = SpanIterator::new(rope.slice(..), [selection].map(|s| s.range()));

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::empty(),
                range: (0..1)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::SELECTION,
                range: (1..3)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::empty(),
                range: (3..len)
            })
        );

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_big_selection() {
        let rope = Rope::from_str("0123456789");
        let len = rope.len_chars();
        let selection = Selection::new(0, 9);

        let mut iter = SpanIterator::new(rope.slice(..), [selection].map(|s| s.range()));

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::SELECTION,
                range: (0..len)
            })
        );

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn selection_on_empty_line() {
        let rope = Rope::from_str("0123456789");
        let len = rope.len_chars();
        let selection = Selection::new(0, 9);

        let mut iter = SpanIterator::new(rope.slice(..), [selection].map(|s| s.range()));

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::SELECTION,
                range: (0..len)
            })
        );

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_selection_at_start() {
        let rope = Rope::from_str("0123456789");
        let len = rope.len_chars();
        let selection = Selection::new(0, 1);

        let mut iter = SpanIterator::new(rope.slice(..), [selection].map(|s| s.range()));

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::SELECTION,
                range: (0..2)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::empty(),
                range: (2..len)
            })
        );

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_selection_at_end() {
        let rope = Rope::from_str("0123456789");
        let len = rope.len_chars();
        let selection = Selection::new(8, 9);

        let mut iter = SpanIterator::new(rope.slice(..), [selection].map(|s| s.range()));

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::empty(),
                range: (0..8)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::SELECTION,
                range: (8..len)
            })
        );

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn two_selections_at_start_end() {
        let rope = Rope::from_str("0123456789");
        let len = rope.len_chars();
        let selection1 = Selection::new(0, 1);
        let selection2 = Selection::new(8, 9);

        let mut iter =
            SpanIterator::new(rope.slice(..), [selection1, selection2].map(|s| s.range()));

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::SELECTION,
                range: (0..2)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::empty(),
                range: (2..8)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::SELECTION,
                range: (8..len)
            })
        );

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn three_selections() {
        let rope = Rope::from_str("0123456789");
        let len = rope.len_chars();
        let selection1 = Selection::new(1, 2);
        let selection2 = Selection::new(4, 5);
        let selection3 = Selection::new(8, 9);

        let mut iter = SpanIterator::new(
            rope.slice(..),
            [selection1, selection2, selection3].map(|s| s.range()),
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::empty(),
                range: (0..1)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::SELECTION,
                range: (1..3)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::empty(),
                range: (3..4)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::SELECTION,
                range: (4..6)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::empty(),
                range: (6..8)
            })
        );

        assert_eq!(
            iter.next(),
            Some(Span {
                kind: SpanKind::SELECTION,
                range: (8..len)
            })
        );

        assert_eq!(iter.next(), None);
    }
}
