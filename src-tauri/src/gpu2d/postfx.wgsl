struct FxParams {
  mode: u32,
  intensity: f32,
  time: f32,
  beat: f32,
  width: f32,
  height: f32,
  _pad: vec2<f32>,
}

@group(0) @binding(0) var samp: sampler;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var<uniform> p: FxParams;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
  var pos = array<vec2<f32>, 3>(
    vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0)
  );
  var out: VsOut;
  out.pos = vec4(pos[vi], 0.0, 1.0);
  out.uv = pos[vi] * 0.5 + 0.5;
  out.uv.y = 1.0 - out.uv.y;
  return out;
}

fn hash1(n: f32) -> f32 {
  let x = sin(n * 127.1 + 311.7) * 43758.5453;
  return x - floor(x);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let uv = in.uv;
  let w = p.width;
  let h = p.height;
  let amount = p.intensity;
  var col: vec4<f32> = textureSample(src, samp, uv);

  if (p.mode == 1u) {
    // glitch: row-band horizontal displacement + RGB split + bar flashes
    let seed = floor(p.time * 1000.0);
    let row = floor(uv.y * h);
    let band = floor(row / (2.0 + hash1(seed + 3.7) * 8.0 * amount));
    let inSlice = hash1(seed + band * 1.7) < min(1.0, 0.05 + amount * 0.5);
    let off = (hash1(seed + band * 2.3) - 0.5) * 40.0 * amount;
    var u2 = uv;
    if (inSlice) { u2.x += off / w; }
    col.r = textureSample(src, samp, u2 + vec2(2.0 / w, 0.0)).r;
    col.g = textureSample(src, samp, u2).g;
    col.b = textureSample(src, samp, u2 - vec2(2.0 / w, 0.0)).b;
    let barY = hash1(seed + 4.2) * h;
    if (abs(uv.y * h - barY) < 1.0 + hash1(seed + 5.1) * 3.0 * amount &&
        hash1(seed + 6.3) < amount) {
      let c = hash1(seed + row);
      col = vec4(select(0.0, 1.0, c > 0.5),
                 select(0.0, 1.0, c < 0.25),
                 select(0.0, 1.0, c > 0.25 && c < 0.5),
                 1.0);
    }
  } else if (p.mode == 2u) {
    // chromatic: RGB channel offsets
    let offset = max(2.0, amount * 14.0) / w;
    col.r = textureSample(src, samp, uv + vec2(offset, 0.0)).r;
    col.g = textureSample(src, samp, uv).g;
    col.b = textureSample(src, samp, uv - vec2(offset, 0.0)).b;
    col.a = 1.0;
  } else if (p.mode == 3u) {
    // zoom: pull the image toward the viewer around the center
    let scale = 1.0 + amount;
    col = textureSample(src, samp, vec2(0.5) + (uv - vec2(0.5)) / scale);
  } else if (p.mode == 4u) {
    // invert: mix toward a negated frame
    let c0 = textureSample(src, samp, uv);
    col = mix(c0, vec4(1.0) - c0, amount);
    col.a = 1.0;
  } else if (p.mode == 5u) {
    // bars: slightly zoomed snapshot with black letterbox bands
    let scale = 1.0 + amount * 0.12;
    let barH = (amount * h * 0.22) / h;
    col = textureSample(src, samp, vec2(0.5) + (uv - vec2(0.5)) / scale);
    if (uv.y < barH || uv.y > 1.0 - barH) {
      col = vec4(0.0, 0.0, 0.0, 1.0);
    }
  } else if (p.mode == 6u) {
    // shockwave: radial ripple pull
    let c = vec2(w, h) * 0.5;
    let d = uv * vec2(w, h) - c;
    let dist = length(d);
    let maxDist = max(length(c), 1.0);
    let phase = dist * 0.13 - p.time * 7.0;
    let pull = amount * 5.0 * (dist / maxDist) * sin(phase * 6.2831853);
    let dir = select(vec2(0.0), d / max(dist, 0.0001), dist > 0.0001);
    col = textureSample(src, samp, uv + dir * pull / vec2(w, h));
  } else if (p.mode == 7u) {
    // pixelate: block-snapped sampling
    let block = max(2.0, round(4.0 + amount * 44.0));
    let px = floor(uv * vec2(w, h) / block) * block + block * 0.5;
    col = textureSample(src, samp, px / vec2(w, h));
  } else if (p.mode == 8u) {
    // tilt: rotate around the center
    let seed = floor(p.time * 1000.0);
    let angle = (hash1(seed) - 0.5) * amount * 0.08;
    let d = uv - vec2(0.5);
    let ca = cos(angle);
    let sa = sin(angle);
    let u2 = vec2(d.x * ca - d.y * sa, d.x * sa + d.y * ca) + vec2(0.5);
    col = textureSample(src, samp, u2);
  } else if (p.mode == 9u) {
    // heat haze: horizontal strips shifted by a slow sine
    let y = uv.y * h;
    let xOff = sin((y + p.time * 1000.0 / 28.0) * 0.05) * amount * 18.0;
    col = textureSample(src, samp, uv + vec2(xOff / w, 0.0));
  }

  return col;
}
