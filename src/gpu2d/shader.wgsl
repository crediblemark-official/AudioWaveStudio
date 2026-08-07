struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tex_id: f32,
};

// Must match crate::gpu2d::renderer::NOISE_LAYER (21).
const NOISE_LAYER: u32 = 21u;
// Must match crate::gpu2d::renderer::IMAGE_LAYER (20) / RADIAL_CENTER_IMAGE_LAYER (22).
// These two image layers are sampled from dedicated native-resolution 2D
// textures (bindings 2/3) instead of the fixed-size atlas array.
const IMAGE_LAYER: u32 = 20u;
const RADIAL_CENTER_IMAGE_LAYER: u32 = 22u;
// The 128x128 film-grain tile is stored in the top-left of the LAYER_SIZE x
// LAYER_SIZE atlas layer (2048), so its UV sub-rect is [0, 128/2048).
const NOISE_TILE_FRAC: f32 = 128.0 / 2048.0;

@group(0) @binding(0) var tex: texture_2d_array<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var bg_tex: texture_2d<f32>;
@group(0) @binding(3) var radial_tex: texture_2d<f32>;

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
        // Background images live in dedicated native-resolution textures; the
        // quad UVs span the full [0,1]^2 of that texture (see upload_background_image).
        var t: vec4<f32>;
        if (layer == IMAGE_LAYER) {
            t = textureSample(bg_tex, samp, uv);
        } else if (layer == RADIAL_CENTER_IMAGE_LAYER) {
            t = textureSample(radial_tex, samp, uv);
        } else {
            t = textureSample(tex, samp, uv, layer);
        }
        c = c * t;
    }
    return c;
}
