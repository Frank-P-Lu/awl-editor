//! The replay result handed from the key driver to the one-frame capture door.

pub(crate) struct ReplayResult {
    pub(crate) zoom: Option<f32>,
    pub(crate) selection: Option<((usize, usize), (usize, usize))>,
    pub(crate) search_query: Option<String>,
    pub(crate) search_case: bool,
    pub(crate) replace_active: bool,
    pub(crate) replacement: String,
    pub(crate) editing_replacement: bool,
    pub(crate) journey: crate::overlay::Journey,
    pub(crate) accept: Option<(crate::overlay::OverlayKind, String)>,
    pub(crate) notice: Option<(String, crate::actions::NoticeKind)>,
    pub(crate) buffers_open: usize,
    #[cfg(test)]
    pub(crate) background_buffers: Vec<(crate::buffers::BufferKey, String)>,
    #[allow(dead_code)]
    pub(crate) intercepts: Vec<crate::replay::Intercept>,
    pub(crate) replay_skips: Vec<crate::replay::SkippedEffect>,
    #[allow(dead_code)]
    pub(crate) warnings: Vec<String>,
}
