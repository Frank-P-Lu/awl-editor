//! Mandatory rail rows participate in the workspace's chrome budget.

use super::fit_workspace_item_rows;

#[test]
fn workspace_chrome_yields_to_its_mandatory_row_before_clamping_capacity() {
    let _guard = crate::testlock::serial();
    for card_h in [150.0, 70.0, 50.0] {
        let lh = 42.0;
        let fit = fit_workspace_item_rows(card_h, 12.0, lh, 1, 65.0, 0, 0.0, 0.0, false, 1);
        assert_eq!(
            fit.header_gap, 0.0,
            "{card_h}: beat must yield to the rail row"
        );
        assert!(
            fit.item_cap >= 1,
            "{card_h}: the mandatory row must survive"
        );
        let first_top = fit.pad + fit.header_rows as f32 * lh + fit.header_gap;
        let last_bottom = card_h - fit.pad;
        assert!(
            first_top + lh <= last_bottom + 0.01,
            "{card_h}: the mandatory row lies outside its resolved inner bounds"
        );
    }
}
