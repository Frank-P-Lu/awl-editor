// A single oversized triangle covering the whole clip space — the standard
// fullscreen-triangle vertex trick (one triangle, no vertex buffer, no seam
// at the diagonal a two-triangle quad would draw). Shared by every pipeline
// that draws one full-viewport pass (background/lava/blur); `gpu_cache::
// Shader::source` prepends this file to theirs.
var<private> TRI_NDC: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0), vec2<f32>( 3.0, -1.0), vec2<f32>(-1.0,  3.0),
);
