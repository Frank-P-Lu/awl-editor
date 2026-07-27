pub(super) fn numeric_ranges(table: &toml::Table) -> (Option<f32>, Option<f32>) {
    let number = |key: &str| table.get(key).and_then(super::model::toml_as_f32);
    (
        number("zoom"),
        number("scroll_sensitivity").map(|s| crate::range::SCROLL_SENSITIVITY.quantize(s)),
    )
}
