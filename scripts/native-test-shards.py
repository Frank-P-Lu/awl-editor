#!/usr/bin/env python3
"""Discover and partition awl's binary unit tests for native-gate.sh."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys


# These are balance hints from a measured six-process run, not the coverage
# roster. Every test prefix is derived from the binary's live --list output;
# prefixes absent here fall into the remainder shard and therefore still run.
HINTS = (
    (
        "render::tests::list_surfaces:: render::tests::chrome_panels:: render::tests::images:: "
        "render::tests::geometry:: render::tests::caret:: render::tests::overlay_personality:: "
        "render::tests::firetail_showcase:: render::tests::markdown:: render::tests::washes:: "
        "render::tests::wysiwyg:: render::tests::outline:: render::tests::nits:: "
        "render::tests::chrome_overlay:: render::tests::caret_transition:: "
        "render::tests::zigzag_ground:: render::tests::markdown_headings:: "
        "render::tests::one_bit:: render::tests::geometry_reshape:: render::tests::cjk:: "
        "render::tests::deckle_ground:: render::tests::tables:: render::tests::folds:: "
        "render::tests::warped_grid:: render::tests::theme:: render::tests::dither:: "
        "render::tests::caret_ink_box:: render::tests::scroll_pos:: render::tests::pixeldiff:: "
        "render::tests::bowerbird_finds:: render::tests::bands_waves:: "
        "render::tests::syntax_roles:: render::tests::rules_composition:: "
        "render::tests::raked_location:: render::tests::facepitch::"
    ).split(),
    (
        "render::tests::chrome_pixel_space:: "
        "render::tests::overlay_plan_law:: render::tests::rotated_rail:: "
        "render::tests::oracle:: render::tests::split_pane:: "
        "render::tests::bowerbird_spacing:: render::tests::foot_hint_lean:: "
        "render::tests::warp_tunnel:: render::tests::column_left_dpi:: "
        "render::tests::gpu_cache_law:: render::tests::page_frame:: "
        "render::tests::text_top_dpi:: render::tests::float_surface_law:: "
        "render::tests::glide_anchor_law:: render::tests::notice:: "
        "render::tests::settings_row_reach_law:: render::tests::chip_plate_floor:: "
        "render::tests::foot_band_no_clip:: render::tests::palette_shortcuts:: "
        "render::tests::selected_secondary_ink_law:: render::tests::workspace_stage_reach:: "
        "render::tests::font_licence:: "
        "render::tests::popover:: render::tests::workspace_footer_plate:: render::plan:: "
        "render::geometry:: render::chrome:: render::blur:: render::rowlayout:: "
        "render::benchsuite:: render::livingband:: render::dither:: render::image_cache:: "
        "render::overrides:: render::framebench::"
    ).split(),
    (
        "render::tests::card_texture_shape:: render::tests::diagonal_composition:: "
        "render::tests::overlay_right_hug_law:: render::tests::visual_selection_law:: "
        "render::tests::overlay_height_clamp_law:: render::tests::syntax_ligatures:: "
        "render::tests::comparison_viewport:: render::tests::frost:: "
        "render::tests::writing_column_decor_dpi:: render::tests::comparison_composite:: "
        "render::tests::grapheme_click:: render::tests::palette_location:: "
        "render::tests::timeline_workspace:: render::tests::frost_feather:: "
        "render::tests::hint_gap:: render::tests::overlay_header_band_law:: "
        "render::tests::theme_caps_law:: render::tests::date_picker_ink:: "
        "render::tests::frost_parallelogram:: render::tests::plan_pass_law:: "
        "render::tests::selection_contrast_law:: render::tests::wrap_affinity:: "
        "render::tests::frost_card_ink:: render::tests::overlay_hover_stability_law:: "
        "render::tests::selection_token_routing_law:: render::tests::webgl_shader_validation:: "
        "render::tests::distinguishability:: render::tests::overlay_align_law:: "
        "render::tests::rotated_location:: render::tests::ground_space:: "
        "render::tests::marker_side:: render::tests::paperbark_retina:: "
        "render::tests::bowerbird_breathe:: render::tests::fold_chevron_direction:: "
        "render::tests::row_offset:: render::tests::caret_visual_body:: "
        "render::tests::frost_footprint:: render::tests::magpie_bands:: "
        "render::tests::stars:: render::tests::cluster_mirror:: "
        "render::tests::frost_width:: render::tests::marker_chevron_owner:: "
        "render::tests::row_pitch_dpi_law:: render::tests::accessory_ink:: "
        "render::tests::fold_chevron_center:: render::tests::palette_scroll_anchor:: "
        "render::tests::reanchor_crossing_law:: render::tests::workspace_plate:: "
        "render::tests::eotf_bit_identity:: render::tests::hover_slop_law:: "
        "render::tests::page_ground_law:: render::tests::query_field::"
    ).split(),
    (
        "render::tests::caret_block:: render::tests::hybrid_band_snap:: "
        "render::tests::range_rail:: render::tests::workspace:: "
        "render::tests::overlay_rail_thirds_law:: render::tests::zoom_anchor:: "
        "render::tests::diagonal_pixel_composition:: render::tests::rotated_label:: "
        "render::tests::alloc_bound_law:: render::tests::organic_ground:: "
        "render::tests::frost_context:: render::tests::hud:: "
        "render::tests::selection_clip_law:: render::tests::warp_one_tunnel:: "
        "render::tests::layout_oracle:: render::tests::potoroo_pane:: "
        "render::tests::workspace_shape:: render::tests::facet_mark_dpi:: "
        "render::tests::overlay_rhythm:: render::tests::quote_orientation:: "
        "render::tests::waves_drift:: render::tests::ambient_wrap_law:: "
        "render::tests::build_integrity:: render::tests::hit_test:: "
        "render::tests::overlay_location_plate:: render::tests::settings_fixture_law::"
    ).split(),
    "app:: syntax:: actions:: run:: overlay:: buffer:: markdown:: capture:: theme::".split(),
)


def test_names(path: pathlib.Path) -> list[str]:
    names = []
    for line in path.read_text().splitlines():
        if line.endswith(": test"):
            names.append(line.removesuffix(": test"))
    if not names:
        raise SystemExit(f"native-test-shards: no tests found in {path}")
    return names


def prefix_for(name: str) -> str:
    return name.rsplit("::", 1)[0] + "::"


def collision_skips(groups: list[set[str]], index: int) -> set[str]:
    others = set().union(*(group for number, group in enumerate(groups) if number != index))
    return {other for own in groups[index] for other in others if own in other}


def partition(list_path: pathlib.Path, output: pathlib.Path, shards: int) -> None:
    names = test_names(list_path)
    output.mkdir(parents=True, exist_ok=True)
    stale = [prefix for hints in HINTS for prefix in hints if not any(name.startswith(prefix) for name in names)]
    if stale:
        raise SystemExit(
            "native-test-shards: stale balance hint needs review: " + stale[0]
        )
    if shards == 1:
        groups = [{prefix_for(name) for name in names}]
    elif shards == 6:
        groups = [set() for _ in range(6)]
        for name in names:
            matched = next(
                (index for index, hints in enumerate(HINTS) if any(name.startswith(p) for p in hints)),
                5,
            )
            groups[matched].add(prefix_for(name))
    else:
        raise SystemExit("native-test-shards: AWL_NATIVE_GATE_SHARDS must be 1 or 6")

    # Libtest filters are substring matches. A short filter such as `run::`
    # also selects `firstrun::`; derive skip prefixes from the live roster so
    # those collisions cannot duplicate coverage across processes.
    for offset, prefixes in enumerate(groups):
        index = offset + 1
        skips = collision_skips(groups, offset)
        (output / f"shard-{index}.filters").write_text("\n".join(sorted(prefixes)) + "\n")
        (output / f"shard-{index}.skips").write_text("\n".join(sorted(skips)) + ("\n" if skips else ""))
    assigned = [sum(any(name.startswith(prefix) for prefix in group) for name in names) for group in groups]
    print(
        f"native-test-shards partition tests={len(names)} shards={shards} "
        f"hinted_counts={','.join(map(str, assigned))}"
    )


def verify(full_path: pathlib.Path, listed_paths: list[pathlib.Path]) -> None:
    full = test_names(full_path)
    seen: dict[str, int] = {}
    shard_counts = []
    for path in listed_paths:
        shard = test_names(path)
        shard_counts.append(len(shard))
        for name in shard:
            seen[name] = seen.get(name, 0) + 1
    missing = [name for name in full if seen.get(name, 0) == 0]
    duplicate = [name for name in full if seen.get(name, 0) > 1]
    foreign = [name for name in seen if name not in set(full)]
    if missing or duplicate or foreign or sum(shard_counts) != len(full):
        detail = []
        if missing:
            detail.append(f"missing={missing[0]}")
        if duplicate:
            detail.append(f"duplicate={duplicate[0]}")
        if foreign:
            detail.append(f"foreign={foreign[0]}")
        raise SystemExit(
            "native-test-shards: completeness refusal "
            f"full={len(full)} shard_sum={sum(shard_counts)} " + " ".join(detail)
        )
    print(
        f"native-test-shards verified full={len(full)} "
        f"shard_sum={sum(shard_counts)} counts={','.join(map(str, shard_counts))}"
    )


def artifacts(json_path: pathlib.Path, output: pathlib.Path) -> None:
    binary = None
    integrations = []
    for line in json_path.read_text().splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("reason") != "compiler-artifact" or not event.get("executable"):
            continue
        target = event["target"]
        kinds = target.get("kind", [])
        if target.get("name") == "awl" and "bin" in kinds and event["profile"].get("test"):
            binary = event["executable"]
        elif "test" in kinds:
            integrations.append(target["name"])
    if binary is None:
        raise SystemExit("native-test-shards: Cargo JSON named no awl binary test executable")
    if not integrations:
        raise SystemExit("native-test-shards: Cargo JSON named no integration-test targets")
    output.write_text("binary=" + binary + "\n" + "\n".join(f"integration={n}" for n in sorted(set(integrations))) + "\n")
    print(f"native-test-shards artifacts binary={binary} integrations={len(set(integrations))}")


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    p = sub.add_parser("partition")
    p.add_argument("list", type=pathlib.Path)
    p.add_argument("output", type=pathlib.Path)
    p.add_argument("shards", type=int)
    v = sub.add_parser("verify")
    v.add_argument("full", type=pathlib.Path)
    v.add_argument("listed", nargs="+", type=pathlib.Path)
    a = sub.add_parser("artifacts")
    a.add_argument("json", type=pathlib.Path)
    a.add_argument("output", type=pathlib.Path)
    args = parser.parse_args()
    if args.command == "partition":
        partition(args.list, args.output, args.shards)
    elif args.command == "verify":
        verify(args.full, args.listed)
    else:
        artifacts(args.json, args.output)


if __name__ == "__main__":
    main()
