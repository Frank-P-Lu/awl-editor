//! Subtract painted marker envelopes from a vertical guide's visible intervals.
pub(super) fn exclude_marker(
    segments: &mut Vec<(f32, f32)>,
    left: f32,
    width: f32,
    bounds: [f32; 4],
    padding: f32,
) {
    let [x, y, w, h] = bounds;
    if left + width <= x - padding || left >= x + w + padding {
        return;
    }
    let (cut_top, cut_bottom) = (y - padding, y + h + padding);
    let mut remaining = Vec::with_capacity(segments.len() + 1);
    for &(top, bottom) in segments.iter() {
        if bottom <= cut_top || top >= cut_bottom {
            remaining.push((top, bottom));
        } else {
            if top < cut_top {
                remaining.push((top, cut_top));
            }
            if bottom > cut_bottom {
                remaining.push((cut_bottom, bottom));
            }
        }
    }
    *segments = remaining;
}
