//! The public world gallery is generated from the real product roster and
//! real capture sidecars. This independent snapshot makes regeneration a
//! conscious part of adding, removing, renaming, or reordering a world.
mod common;

use std::path::Path;

fn awl_bin() -> std::process::Command {
    common::awl(&common::shared_sandbox())
}

fn generated_worlds(html: &str) -> Vec<&str> {
    html.lines()
        .filter_map(|line| {
            line.split_once("data-world=\"")
                .and_then(|(_, tail)| tail.split_once('"'))
                .map(|(name, _)| name)
        })
        .collect()
}

#[test]
fn public_gallery_matches_the_product_roster_in_order() {
    let out = awl_bin()
        .arg("--list-worlds")
        .output()
        .expect("spawn awl --list-worlds");
    assert!(out.status.success(), "awl --list-worlds should succeed");
    let stdout = String::from_utf8(out.stdout).expect("world roster is UTF-8");
    let expected: Vec<&str> = stdout.lines().collect();
    let html = include_str!("../site/themes.html");
    let actual = generated_worlds(html);

    for index in 0..expected.len().max(actual.len()) {
        match (expected.get(index), actual.get(index)) {
            (Some(product), Some(gallery)) if product == gallery => {}
            (Some(product), Some(gallery)) => panic!(
                "public world gallery is stale at position {index}: product has {product:?}, page has {gallery:?}; run scripts/public-world-gallery.sh"
            ),
            (Some(product), None) => panic!(
                "public world gallery is missing {product:?} at position {index}; run scripts/public-world-gallery.sh"
            ),
            (None, Some(gallery)) => panic!(
                "public world gallery has retired or duplicate world {gallery:?} at position {index}; run scripts/public-world-gallery.sh"
            ),
            (None, None) => unreachable!(),
        }
    }

    for world in expected {
        let png = format!("site/img/worlds/{world}.png");
        assert!(
            Path::new(&png).is_file(),
            "public world gallery is missing {world}: {png}"
        );
    }
}
