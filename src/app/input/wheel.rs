//! Native wheel-unit conversion and picker packet accumulation.

use crate::app::{WHEEL_LINES_PER_NOTCH, WHEEL_PIXELS_PER_LINE, render};

pub(in crate::app) fn initial_sensitivity(configured: Option<f32>) -> f32 {
    configured.unwrap_or(crate::range::SCROLL_SENSITIVITY.default)
}

pub(super) fn line_wheel_document_px(y: f32, zoom: f32, dpi: f32) -> f32 {
    -y * WHEEL_LINES_PER_NOTCH * render::LINE_HEIGHT * zoom * dpi
}

pub(super) fn pixel_wheel_document_px(y: f32, sensitivity: f32) -> f32 {
    -y * sensitivity
}

pub(super) fn pixel_wheel_axes(x: f32, y: f32, sensitivity: f32) -> (f32, f32) {
    (x * sensitivity, y * sensitivity)
}

pub(super) fn accumulate_picker_pixels(accum: &mut f32, delta_px: f32) -> f32 {
    *accum += delta_px;
    let whole = (*accum / WHEEL_PIXELS_PER_LINE).trunc();
    *accum -= whole * WHEEL_PIXELS_PER_LINE;
    whole
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_wheel_units_and_sensitivity_are_isolated() {
        let base = render::LINE_HEIGHT;
        assert_eq!(line_wheel_document_px(-1.0, 1.0, 1.0), 3.0 * base);
        assert_eq!(line_wheel_document_px(-1.0, 2.0, 1.5), 9.0 * base);
        assert_eq!(pixel_wheel_document_px(-3.0, 2.0), 6.0);
        assert_eq!(pixel_wheel_axes(2.0, -3.0, 2.0), (4.0, -6.0));
    }

    #[test]
    fn picker_three_pixel_packet_advances_zero_rows() {
        let mut accum = 0.0;
        assert_eq!(accumulate_picker_pixels(&mut accum, 3.0), 0.0);
        assert_eq!(accum, 3.0);
    }
}
