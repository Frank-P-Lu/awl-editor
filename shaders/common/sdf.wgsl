// Signed distance to a rounded rectangle centered at origin with half-size
// `b` and corner radius `r`. Negative inside, positive outside. Shared by
// every pipeline that draws a soft or hard rounded-rect silhouette
// (caret/image/selection); `gpu_cache::Shader::source` prepends this file to
// theirs, so there is exactly one copy for naga to compile against.
fn sd_round_rect(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r, r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - r;
}
