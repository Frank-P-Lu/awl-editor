//! Continuous-scroll laws: packetisation must not change semantic geometry.

use super::super::*;

#[test]
fn scroll_pos_fixed_point_has_exact_value_semantics() {
    let _serial = crate::testlock::serial();
    let a = ScrollPos { row: 3, px_q: 17 };
    assert_eq!(a, ScrollPos { row: 3, px_q: 17 });
    assert_ne!(a, ScrollPos { row: 3, px_q: 18 });
    assert_eq!(a.px(), 17.0 / 64.0);
}
