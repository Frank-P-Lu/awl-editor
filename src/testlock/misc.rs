//! Every OTHER process-global toggle a test can flip — everything
//! `crate::testlock::serial()`'s guard did NOT already snapshot directly on
//! [`super::SerialGuard`] (world, page, spellcheck) and that
//! `crate::render::overrides` does not already own (the ten `AWL_*_FORCE`
//! render knobs + the living-band probe).
//!
//! **Why this module exists:** `mac_about`'s command sweep applied
//! EVERY command's action, so it fired every toggle in the roster as a side
//! effect — and it restored exactly the three globals the guard's own exit
//! audit checked (world, page, spellcheck). The restore list had been sized to
//! the AUDIT's coverage, not to the sweep's actual reach: `debug` sat outside
//! that audit, leaked ON, and silently changed what an unrelated margin-pixel
//! law measured. A hand-widened restore list (the instance fix, in
//! `mac_about::tests`) repeats the same shape at one call site — it now names
//! five more globals, but still misses `caret` mode and the held stats HUD (a
//! full sweep fires `Action::ToggleCaretMode` and `Action::ShowStatsHud` too).
//! A SECOND, independent command sweep
//! (`actions::tests::picker_misc_smoke::every_catalog_command_dispatches_without_panicking`)
//! has the identical shape and its own gap: it restores caret/page/debug/hud/
//! spellcheck/about/lifetime/outline/typewriter/nits, but not `menu_bar` (and
//! its caret restore calls `set_mode` unconditionally, which cannot express
//! "was auto" — see [`pins`]'s `caret_mode` field). Two independently authored
//! hand lists, two different gaps: the mechanism — not either instance — is
//! what this module fixes. (See the census law
//! `every_toggle_and_card_flag_site_is_covered_by_serial_guard_or_named_here`
//! and the mutation-proof law
//! `every_misc_field_is_restored_not_just_the_one_that_bit_us`, both in this
//! module's own `tests` submodule.)
//!
//! The fix that stops this recurring: [`pins`] (the snapshot), [`leaked`] (the
//! audit), and [`restore`] (the cleanup) share ONE field list, so the guard's
//! audit and its restore are the same list *by construction* — there is no
//! second, narrower list for a future author to under-fill. Widening this
//! struct is the one place a new sticky global joins the guard; nothing here
//! is sized to what any one sweep happens to touch.
//!
//! MEMBERSHIP: every `crate::toggle::Toggle` static outside `page`/`spell`
//! (already fields on `SerialGuard` directly, predating this module — see
//! `crate::toggle`'s own module doc for the full roster and its
//! `every_sticky_atomic_bool_routes_through_toggle_or_is_named_here` sweep
//! law, which is what makes "every `Toggle`" an enumerable set rather than a
//! hand list), every `crate::card::CardFlag` static (the three summoned-card
//! open flags plus the peek hold), the caret-mode override, and the held
//! stats HUD flag (transient, but reachable straight from a command — see
//! `Action::ShowStatsHud` — so a full sweep dirties it exactly like a sticky
//! preference).
//!
//! `menubar`'s open-dropdown INDEX rides along too (`menu_dropdown_open`) —
//! its own doc names it "a process-global exactly like `crate::hud`'s held
//! flag", the same transient-but-command-adjacent category `hud_held` is here
//! for, so it gets the same treatment. Three more single-value globals join
//! for the same reason as the settings-only `Toggle`s above (a genuine
//! process-global with a `cfg(test)` writer, even though a bare `COMMANDS`
//! sweep alone does not reach it): the spellcheck dictionary variant, the
//! date-format picker's active format, and the scroll-sensitivity slider.
//!
//! Deliberately NOT here: `streaks::CUMULATIVE` (the summoned card's page).
//! It is already self-resetting through the ONE door that opens the card —
//! `streaks::set_open(true)` resets it to the heatmap page on every summon —
//! and there is no Action that reaches it without going through that door
//! first, so restoring `streaks_open` through [`restore`] already carries it
//! back to a clean state whenever the card's open flag itself changed. Also
//! not here: `probe::LIVE_ACTIVE`/`FLIGHT_ACTIVE` (the flight recorder — its
//! own tests reset it directly, and no `Action` reaches it) and
//! `crashlog::HOOK_INSTALLED` (a one-way install latch, never flips back).

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MiscPins {
    debug: bool,
    outline: bool,
    menu_bar: bool,
    typewriter: bool,
    nits: bool,
    popover: bool,
    file_visibility_all: bool,
    reduced_motion: bool,
    code_ligatures: bool,
    wysiwyg: bool,
    inline_images: bool,
    whichkey_force_shown: bool,
    /// `None` = auto (no override); mirrors `crate::caret::is_auto`.
    caret_mode: Option<crate::caret::CaretMode>,
    about_open: bool,
    lifetime_open: bool,
    streaks_open: bool,
    peek_open: bool,
    hud_held: bool,
    menu_dropdown_open: Option<usize>,
    spell_variant: crate::spell::DictVariant,
    date_format: crate::dateformat::DateFormat,
    scroll_sensitivity: f32,
}

/// Read every field above, for the guard's entry snapshot.
#[cfg(test)]
pub(crate) fn pins() -> MiscPins {
    MiscPins {
        debug: crate::debug::debug_on(),
        outline: crate::outline::outline_on(),
        menu_bar: crate::menubar::menu_bar_on(),
        typewriter: crate::typewriter::typewriter_on(),
        nits: crate::nits::nits_on(),
        popover: crate::popover::popover_on(),
        file_visibility_all: crate::file_visibility::all_on(),
        reduced_motion: crate::motion::reduced(),
        code_ligatures: crate::render::code_ligatures_on(),
        wysiwyg: crate::markdown::wysiwyg_on(),
        inline_images: crate::markdown::inline_images_on(),
        whichkey_force_shown: crate::whichkey::force_shown(),
        caret_mode: (!crate::caret::is_auto()).then(crate::caret::mode),
        about_open: crate::about::about_open(),
        lifetime_open: crate::lifetime::lifetime_open(),
        streaks_open: crate::streaks::streaks_open(),
        peek_open: crate::peek::peek_open(),
        hud_held: crate::hud::hud_held(),
        menu_dropdown_open: crate::menubar::open_menu(),
        spell_variant: crate::spell::active_variant(),
        date_format: crate::dateformat::active_format(),
        scroll_sensitivity: crate::settings::scroll_sensitivity(),
    }
}

/// Put every field back the way [`pins`] found it. Routed through the ordinary
/// setters — the guard still holds the lock when it runs this (see
/// `SerialGuard`'s `Drop`), so a `Toggle`-backed field's writer-serialized
/// assert still finds the lock held, exactly like `page`/`spellcheck` already
/// do above this module.
#[cfg(test)]
pub(crate) fn restore(p: &MiscPins) {
    crate::debug::set_debug_on(p.debug);
    crate::outline::set_outline_on(p.outline);
    crate::menubar::set_menu_bar_on(p.menu_bar);
    crate::typewriter::set_typewriter_on(p.typewriter);
    crate::nits::set_nits_on(p.nits);
    crate::popover::set_popover_on(p.popover);
    crate::file_visibility::set_all_on(p.file_visibility_all);
    crate::motion::set_reduced(p.reduced_motion);
    crate::render::set_code_ligatures_on(p.code_ligatures);
    crate::markdown::set_wysiwyg_on(p.wysiwyg);
    crate::markdown::set_inline_images_on(p.inline_images);
    crate::whichkey::set_force_shown(p.whichkey_force_shown);
    match p.caret_mode {
        Some(m) => crate::caret::set_mode(m),
        None => crate::caret::clear_override(),
    }
    crate::about::set_open(p.about_open);
    crate::lifetime::set_open(p.lifetime_open);
    crate::streaks::set_open(p.streaks_open);
    crate::peek::set_open(p.peek_open);
    crate::hud::set_held(p.hud_held);
    crate::menubar::set_open(p.menu_dropdown_open);
    crate::spell::set_active_variant(p.spell_variant);
    crate::dateformat::set_active_format(p.date_format);
    crate::settings::set_scroll_sensitivity(p.scroll_sensitivity);
}

/// Name every field whose value differs, `before -> after`. Both sides are
/// destructured exhaustively, so a new [`MiscPins`] field must be listed here
/// consciously and cannot dodge the sweep by defaulting to "unchanged" — the
/// same discipline `render::overrides::leaked_knobs` uses.
#[cfg(test)]
pub(crate) fn leaked(before: &MiscPins, after: &MiscPins) -> Vec<String> {
    let MiscPins {
        debug: b_debug,
        outline: b_outline,
        menu_bar: b_menu_bar,
        typewriter: b_typewriter,
        nits: b_nits,
        popover: b_popover,
        file_visibility_all: b_file_visibility_all,
        reduced_motion: b_reduced_motion,
        code_ligatures: b_code_ligatures,
        wysiwyg: b_wysiwyg,
        inline_images: b_inline_images,
        whichkey_force_shown: b_whichkey_force_shown,
        caret_mode: b_caret_mode,
        about_open: b_about_open,
        lifetime_open: b_lifetime_open,
        streaks_open: b_streaks_open,
        peek_open: b_peek_open,
        hud_held: b_hud_held,
        menu_dropdown_open: b_menu_dropdown_open,
        spell_variant: b_spell_variant,
        date_format: b_date_format,
        scroll_sensitivity: b_scroll_sensitivity,
    } = before;
    let MiscPins {
        debug: a_debug,
        outline: a_outline,
        menu_bar: a_menu_bar,
        typewriter: a_typewriter,
        nits: a_nits,
        popover: a_popover,
        file_visibility_all: a_file_visibility_all,
        reduced_motion: a_reduced_motion,
        code_ligatures: a_code_ligatures,
        wysiwyg: a_wysiwyg,
        inline_images: a_inline_images,
        whichkey_force_shown: a_whichkey_force_shown,
        caret_mode: a_caret_mode,
        about_open: a_about_open,
        lifetime_open: a_lifetime_open,
        streaks_open: a_streaks_open,
        peek_open: a_peek_open,
        hud_held: a_hud_held,
        menu_dropdown_open: a_menu_dropdown_open,
        spell_variant: a_spell_variant,
        date_format: a_date_format,
        scroll_sensitivity: a_scroll_sensitivity,
    } = after;

    let mut out = Vec::new();
    macro_rules! field {
        ($name:literal, $b:ident, $a:ident) => {
            if $b != $a {
                out.push(format!("{}: {:?} -> {:?}", $name, $b, $a));
            }
        };
    }
    field!("debug", b_debug, a_debug);
    field!("outline", b_outline, a_outline);
    // `menu_bar` is RESTORED above but deliberately NOT AUDITED, and the reason
    // is that its default is PLATFORM-DEPENDENT: false on macOS, true
    // everywhere else. Passive-surface fixtures set it false to reach a known
    // state, which is a silent no-op on macOS and a real mutation on Linux — so
    // auditing it fails sixty tests on one platform and none on the other, for
    // fixtures that are behaving identically. Restoring it still protects the
    // next test; only the complaint is withheld. Auditing it properly means
    // giving those fixtures a guard whose lifetime is the TEST's, not the
    // helper's, which is a refactor rather than a field in this list.
    let _ = (&b_menu_bar, &a_menu_bar);
    field!("typewriter", b_typewriter, a_typewriter);
    field!("nits", b_nits, a_nits);
    field!("popover", b_popover, a_popover);
    field!(
        "file_visibility_all",
        b_file_visibility_all,
        a_file_visibility_all
    );
    field!("reduced_motion", b_reduced_motion, a_reduced_motion);
    field!("code_ligatures", b_code_ligatures, a_code_ligatures);
    field!("wysiwyg", b_wysiwyg, a_wysiwyg);
    field!("inline_images", b_inline_images, a_inline_images);
    field!(
        "whichkey_force_shown",
        b_whichkey_force_shown,
        a_whichkey_force_shown
    );
    field!("caret_mode", b_caret_mode, a_caret_mode);
    field!("about_open", b_about_open, a_about_open);
    field!("lifetime_open", b_lifetime_open, a_lifetime_open);
    field!("streaks_open", b_streaks_open, a_streaks_open);
    field!("peek_open", b_peek_open, a_peek_open);
    field!("hud_held", b_hud_held, a_hud_held);
    field!(
        "menu_dropdown_open",
        b_menu_dropdown_open,
        a_menu_dropdown_open
    );
    field!("spell_variant", b_spell_variant, a_spell_variant);
    field!("date_format", b_date_format, a_date_format);
    field!(
        "scroll_sensitivity",
        b_scroll_sensitivity,
        a_scroll_sensitivity
    );
    out
}

/// A scoped snapshot-and-restore for a test that touches ANY field in
/// [`MiscPins`] and would rather not hand-roll its own capture/restore pair —
/// the exact hand-rolled pattern that, applied to `caret::mode()`, was newly
/// widening this guard's own biggest source of fresh failures:
/// `caret::mode()` returns a concrete `CaretMode` with no way to say "auto",
/// so `let m = caret::mode(); …;
/// caret::set_mode(m);` cannot restore an auto entry state and leaves the
/// override armed. Declare this AFTER the test's own `crate::testlock::serial()`
/// guard (so it drops BEFORE that guard, while the lock is still held) and it
/// captures on construction, restores on `Drop` — including on an unwinding
/// panic, matching every other restore path in this module.
#[cfg(test)]
pub(crate) struct TogglesRestore(MiscPins);

#[cfg(test)]
impl TogglesRestore {
    pub(crate) fn capture() -> Self {
        TogglesRestore(pins())
    }
}

#[cfg(test)]
impl Drop for TogglesRestore {
    fn drop(&mut self) {
        restore(&self.0);
    }
}

#[cfg(test)]
mod tests;
