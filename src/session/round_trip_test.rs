use super::*;

#[test]
fn round_trips_through_toml() {
    let state = SessionState {
        root: Some(PathBuf::from("/proj")),
        document_active: Some(true),
        active: Some(PathBuf::from("/proj/a.md")),
        buffers: vec![
            (
                PathBuf::from("/proj/a.md"),
                BufferPos {
                    line: 3,
                    col: 5,
                    scroll: 2,
                    scroll_px_q: 0,
                },
            ),
            (
                PathBuf::from("/proj/b.rs"),
                BufferPos {
                    line: 0,
                    col: 0,
                    scroll: 0,
                    scroll_px_q: 0,
                },
            ),
        ],
        window: Some(WindowFrame {
            x: 10,
            y: 20,
            width: 1200,
            height: 800,
        }),
    };
    let text = to_toml(&state);
    assert_eq!(from_toml(&text), state);
}
