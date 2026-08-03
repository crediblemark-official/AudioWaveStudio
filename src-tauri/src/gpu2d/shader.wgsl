struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tex_id: f32,
};

// Must match crate::gpu2d::renderer::NOISE_LAYER (21).
const NOISE_LAYER: u32 = 21u;
// The 128x128 film-grain tile is stored in the top-left of the 1024x1024
// atlas layer, so its UV sub-rect is [0, 128/1024) in each axis.
const NOISE_TILE_FRAC: f32 = 128.0 / 1024.0;

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
        let layer = u32(in.tex_id - 1.0);
        var uv = in.uv;
        // The noise quad's UVs span [0, w/128] x [0, h/128]; fold them back
        // into the tile's sub-rect so the grain tiles 1:1 across the whole
        // canvas (matches TS createPattern(noiseCanvas, 'repeat')).
        if (layer == NOISE_LAYER) {
            uv = vec2(fract(uv.x) * NOISE_TILE_FRAC, fract(uv.y) * NOISE_TILE_FRAC);
        }
        let t = textureSample(tex, samp, uv, layer);
        c = c * t;
    }
    return c;
}
