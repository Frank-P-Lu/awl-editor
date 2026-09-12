#[derive(Debug)]
pub(super) struct PipelineOwnership {
    pub(super) raw_lines: Vec<usize>,
    pub(super) failures: Vec<String>,
}

/// Inspect lexical ownership, not matching-line counts. Comments are removed
/// before balancing each cache wrapper's whole build closure.
pub(super) fn render_pipeline_ownership(
    source: &str,
    delegated_builder: Option<&str>,
) -> PipelineOwnership {
    // Receiver-agnostic so the UFCS spelling is a raw constructor too.
    const RAW: &str = "create_render_pipeline(";
    const WRAPPER: &str = "gpu_cache::render_pipeline(";
    let code = source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .collect::<Vec<_>>()
        .join("\n");
    let raw_sites: Vec<_> = code.match_indices(RAW).map(|(at, _)| at).collect();
    let mut failures = Vec::new();
    let wrappers = cache_wrapper_ranges(&code, WRAPPER, &mut failures);
    let delegated = delegated_builder.and_then(|builder| {
        let declaration = format!("fn {builder}(");
        let definition = code.find(&declaration)?;
        let body_start = code[definition..].find('{')? + definition;
        let body_end = matching_delimiter(&code, body_start, b'{', b'}')?;
        let name_at = definition + 3;
        let needle = format!("{builder}(");
        let calls: Vec<_> = code
            .match_indices(&needle)
            .map(|(at, _)| at)
            .filter(|&at| at != name_at)
            .collect();
        Some((body_start, body_end, calls))
    });
    if delegated_builder.is_some() && delegated.is_none() {
        failures.push("delegated cache builder has no complete function body".to_owned());
    }
    if let Some((body_start, body_end, calls)) = &delegated {
        let raw_in_builder = raw_sites
            .iter()
            .filter(|&&raw| *body_start < raw && raw < *body_end)
            .count();
        if raw_in_builder != 1 || calls.len() != 1 {
            failures.push(format!(
                "delegated builder owns {raw_in_builder} raw constructors and has {} calls",
                calls.len()
            ));
        }
        for &call in calls {
            let owners = wrappers
                .iter()
                .filter(|&&(start, end)| start < call && call < end)
                .count();
            if owners != 1 {
                failures.push(format!(
                    "line {} delegated builder call has {owners} cache owners",
                    source_line(&code, call)
                ));
            }
        }
    }
    for &raw in &raw_sites {
        let direct_owners = wrappers
            .iter()
            .filter(|&&(start, end)| start < raw && raw < end)
            .count();
        let delegated_owner = delegated
            .as_ref()
            .is_some_and(|(start, end, calls)| *start < raw && raw < *end && calls.len() == 1);
        let owners = direct_owners + usize::from(delegated_owner);
        if owners != 1 {
            failures.push(format!(
                "line {} raw constructor has {owners} cache owners",
                source_line(&code, raw)
            ));
        }
    }
    for &(start, end) in &wrappers {
        let direct = raw_sites
            .iter()
            .filter(|&&raw| start < raw && raw < end)
            .count();
        let delegated_calls = delegated.as_ref().map_or(0, |(_, _, calls)| {
            calls.iter().filter(|&&at| start < at && at < end).count()
        });
        let owned = direct + delegated_calls;
        if owned != 1 {
            failures.push(format!(
                "line {} cache wrapper owns {owned} constructor seams",
                source_line(&code, start)
            ));
        }
    }
    PipelineOwnership {
        raw_lines: raw_sites
            .into_iter()
            .map(|at| source_line(&code, at))
            .collect(),
        failures,
    }
}

fn cache_wrapper_ranges(
    source: &str,
    wrapper: &str,
    failures: &mut Vec<String>,
) -> Vec<(usize, usize)> {
    source
        .match_indices(wrapper)
        .filter_map(|(start, _)| {
            let open = start + wrapper.len() - 1;
            matching_delimiter(source, open, b'(', b')').map_or_else(
                || {
                    failures.push(format!(
                        "line {} cache wrapper has no closing delimiter",
                        source_line(source, start)
                    ));
                    None
                },
                |end| Some((start, end)),
            )
        })
        .collect()
}

fn matching_delimiter(source: &str, open: usize, opening: u8, closing: u8) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[open..].iter().copied().enumerate() {
        if byte == opening {
            depth += 1;
        } else if byte == closing {
            depth -= 1;
            if depth == 0 {
                return Some(open + offset);
            }
        }
    }
    None
}

fn source_line(source: &str, at: usize) -> usize {
    1 + source.as_bytes()[..at]
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count()
}
