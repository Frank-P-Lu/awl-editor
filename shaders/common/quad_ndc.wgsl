// A 6-vertex triangle-list unit quad in clip-space NDC, corners at +/-1.
// Shared by every pipeline whose vertex stage places a quad directly in NDC
// (caret/selection/spellunderline); `gpu_cache::Shader::source` prepends this
// file to theirs.
var<private> QUAD_NDC: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>( 1.0, -1.0), vec2<f32>( 1.0,  1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>( 1.0,  1.0), vec2<f32>(-1.0,  1.0),
);
