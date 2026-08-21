use super::*;

pub(super) fn placard_origin(
    corner: theme::PlacardCorner,
    anchor: (f32, f32, f32, f32),
    w: f32,
    h: f32,
    inset: f32,
) -> (f32, f32) {
    let (ax, ay, aw, ah) = anchor;
    let x = match corner {
        theme::PlacardCorner::TL | theme::PlacardCorner::BL | theme::PlacardCorner::Auto => {
            (ax + inset).min((ax + aw - w).max(ax))
        }
        theme::PlacardCorner::TR | theme::PlacardCorner::BR => (ax + aw - inset - w).max(ax),
    };
    let y = match corner {
        theme::PlacardCorner::TL | theme::PlacardCorner::TR => {
            (ay + inset).min((ay + ah - h).max(ay))
        }
        theme::PlacardCorner::BL | theme::PlacardCorner::BR | theme::PlacardCorner::Auto => {
            (ay + ah - inset - h).max(ay)
        }
    };
    (x, y)
}

pub(super) fn apply_placard_placement(
    (x, y): (f32, f32),
    font_size: f32,
    placement: theme::PlacardPlacement,
) -> (f32, f32) {
    match placement {
        theme::PlacardPlacement::Contained => (x, y),
        theme::PlacardPlacement::Bleed { x_em, y_em } => {
            (x + x_em * font_size, y + y_em * font_size)
        }
    }
}
