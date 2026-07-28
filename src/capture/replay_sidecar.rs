use super::opts::CaptureOpts;
use super::sidecar::json_string;

pub(super) fn replay_skips_json(opts: &CaptureOpts) -> String {
    let items = opts
        .replay_skips
        .iter()
        .map(|skip| {
            format!(
                "{{ \"effect\": {}, \"action\": {} }}",
                json_string(skip.effect),
                json_string(&skip.action),
            )
        })
        .collect::<Vec<_>>();
    format!("[{}]", items.join(", "))
}
