struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tex_id: f32,
};

@group(0) @binding(0) var tex: texture_2d_array<f32>;
@group(0) @binding(1) var samp: sampler;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tex_id: f32,
) -> VsOut {
    var o: VsOut;
    o.pos = vec4(position, 0.0, 1.0);
    o.color = color;
    o.uv = uv;
    o.tex_id = tex_id;
    return o;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var c = in.color;
    if (in.tex_id > 0.5) {
        let t = textureSample(tex, samp, in.uv, u32(in.tex_id - 1.0));
        c = c * t;
    }
    return c;
}
