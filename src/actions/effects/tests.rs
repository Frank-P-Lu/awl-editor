//! The depth-first effect-ordering laws — a nested `RunAction` must never
//! drop the outer transition's remainder, on both the live and headless
//! drivers. Carved out of `effects.rs` to keep the production file under its
//! size ceiling; not a semantic split.

use super::*;

fn transition(effects: Vec<Effect>) -> Transition {
    Transition {
        primary: effects.first().cloned().unwrap_or(Effect::None),
        effects,
    }
}

#[test]
fn nested_actions_are_depth_first_and_never_drop_the_outer_remainder() {
    let outer = Action::OpenCommandPalette;
    let nested = Action::Save;
    let mut work = EffectWorklist::from_transition(
        outer.clone(),
        transition(vec![
            Effect::Notice(NoticeEffect::Toast("before".into())),
            Effect::RunAction(nested.clone()),
            Effect::Notice(NoticeEffect::Sticky("after".into())),
        ]),
    );
    let mut seen = Vec::new();
    while let Some(item) = work.next() {
        match item {
            EffectWorkItem::Action(action) => {
                seen.push(format!("action:{action:?}"));
                work.expand(
                    action,
                    transition(vec![Effect::Notice(NoticeEffect::Clear)]),
                );
            }
            EffectWorkItem::Effect { effect, .. } => {
                seen.push(format!("effect:{effect:?}"));
                if let Effect::RunAction(action) = effect {
                    work.descend(action);
                }
            }
        }
    }
    assert_eq!(
        seen,
        vec![
            "effect:Notice(Toast(\"before\"))",
            "effect:RunAction(Save)",
            "action:Save",
            "effect:Notice(Clear)",
            "effect:Notice(Sticky(\"after\"))",
        ],
        "the shared worklist must visit before → nested transition → after"
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let live = std::fs::read_to_string(root.join("app/apply.rs")).unwrap();
    let headless = std::fs::read_to_string(root.join("main/run/chord.rs")).unwrap();
    assert!(
        live.contains("visit_transition_effects(transition")
            && headless.contains("EffectWorklist::root"),
        "MUTATION TRAP: live and headless interpreters must use the shared depth-first drivers"
    );
}

#[test]
fn live_driver_cannot_drop_the_outer_remainder_after_a_nested_action() {
    let nested = Action::Save;
    let seen = std::cell::RefCell::new(Vec::new());
    visit_transition_effects(
        transition(vec![
            Effect::Notice(NoticeEffect::Toast("before".into())),
            Effect::RunAction(nested),
            Effect::Notice(NoticeEffect::Sticky("after".into())),
        ]),
        |effect| {
            seen.borrow_mut().push(format!("outer:{effect:?}"));
            if matches!(effect, Effect::RunAction(Action::Save)) {
                visit_transition_effects(
                    transition(vec![Effect::Notice(NoticeEffect::Clear)]),
                    |nested_effect| {
                        seen.borrow_mut().push(format!("nested:{nested_effect:?}"));
                    },
                );
            }
        },
    );
    assert_eq!(
        seen.into_inner(),
        vec![
            "outer:Notice(Toast(\"before\"))",
            "outer:RunAction(Save)",
            "nested:Notice(Clear)",
            "outer:Notice(Sticky(\"after\"))",
        ],
        "MUTATION TRAP: live nested dispatch returned before the outer effect remainder"
    );
}
