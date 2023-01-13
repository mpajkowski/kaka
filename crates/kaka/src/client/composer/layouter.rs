use kaka_core::shapes::Rect;

pub const fn editor(viewport: Rect) -> Rect {
    Rect {
        height: viewport.height - 1,
        y: 0,
        ..viewport
    }
}

pub const fn prompt(viewport: Rect) -> Rect {
    Rect {
        x: 0,
        y: viewport.height - 1,
        height: 1,
        width: viewport.width,
    }
}
