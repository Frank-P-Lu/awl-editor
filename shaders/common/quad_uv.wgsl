// A 6-vertex triangle-list unit quad in UV space, corners at [0,1]. Shared by
// every pipeline whose vertex stage samples a texture over the whole instance
// rect (caret_glyph/image/rotated_label); `gpu_cache::Shader::source`
// prepends this file to theirs.
var<private> QUAD_UV: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
);
