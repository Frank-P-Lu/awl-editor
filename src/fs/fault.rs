use std::path::Path;

pub(super) fn after_tmp_write(_tmp: &Path) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if std::env::var_os("AWL_FAULT_OBSERVED_WRITE").is_some() {
            use std::io::Write;
            let mut out = std::io::stdout().lock();
            let _ = writeln!(out, "fault-observed tmp-write {}", _tmp.display());
            let _ = out.flush();
        }
        if let Ok(ms) = std::env::var("AWL_FAULT_DELAY_MS")
            && let Ok(ms) = ms.parse::<u64>()
        {
            std::thread::sleep(std::time::Duration::from_millis(ms));
        }
    }
}
