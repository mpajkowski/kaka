use kaka_core::shapes::Rect;

pub const fn editor(viewport: Rect) -> Rect {
    Rect {
        height: viewport.height - 1,
        ..viewport
    }
}

pub const fn prompt(viewport: Rect) -> Rect {
    Rect {
        height: 1,
        y: viewport.height - 1,
        ..viewport
    }
}
