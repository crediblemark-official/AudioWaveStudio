// Native 3D pipeline for GpuRenderer's Scene3D pass.
// Vertex layout: position (vec3), normal (vec3), color (vec4) — matches
// crate::gpu2d::scene3d::V3 and crate::gpu2d::renderer::RawVertex3.
// Uniform layout: see RawCamUniform in renderer.rs (view_proj mat4, then
// three vec4s: light_dir / light_col / ambient) — mat4x4<f32> is column-major
// to match glam::Mat4.

struct Uniforms {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_col: vec4<f32>,
    ambient: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
) -> VsOut {
    var o: VsOut;
    o.pos = u.view_proj * vec4(position, 1.0);
    o.color = color;
    o.normal = normal;
    return o;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let l = normalize(u.light_dir.xyz);
    let diff = max(dot(n, l), 0.0);
    // Wrap lighting (0.2 floor) so back faces and under-lit sides still read —
    // keeps the neon look instead of half the geometry falling into shadow.
    let diffuse = u.light_col.rgb * (diff * 0.8 + 0.2);
    let amb = u.ambient.rgb * u.ambient.w;
    return vec4(in.color.rgb * (amb + diffuse), in.color.a);
}
