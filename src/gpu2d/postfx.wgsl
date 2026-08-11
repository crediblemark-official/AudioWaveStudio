struct FxParams {
  mode: u32,
  intensity: f32,
  time: f32,
  beat: f32,
  width: f32,
  height: f32,
  fps: f32,
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

// Bit-identical port of the mulberry32 PRNG from screenEffects.ts.
fn mulberry32(state: ptr<function, u32>) -> f32 {
  *state = *state + 0x6D2B79F5u;
  let a = *state;
  var t = a ^ (a >> 15u);
  t = t * (a | 1u);
  t = (t + (t ^ (t >> 7u)) * (t | 61u)) ^ t;
  return f32(t ^ (t >> 14u)) / 4294967296.0;
}

// W3C compositing helpers (compositing-1: hue blend).
fn lum3(c: vec3<f32>) -> f32 { return dot(c, vec3(0.3, 0.59, 0.11)); }
fn sat3(c: vec3<f32>) -> f32 { return max(max(c.r, c.g), c.b) - min(min(c.r, c.g), c.b); }
fn mid3(c: vec3<f32>) -> f32 { return max(min(c.r, c.g), min(max(c.r, c.g), c.b)); }
fn setSat(c: vec3<f32>, s: f32) -> vec3<f32> {
  let d = max(max(c.r, c.g), c.b) - min(min(c.r, c.g), c.b);
  if (d < 1e-6) { return vec3(0.0); }
  let m = mid3(c);
  return clamp(m + (c - m) * (s / d), vec3(0.0), vec3(1.0));
}
fn setLum(c: vec3<f32>, l: f32) -> vec3<f32> { return c + vec3(l - lum3(c)); }
fn hueBlend(cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
  return setLum(setSat(cs, sat3(cb)), lum3(cb));
}

fn hueComponent(n: f32, h: f32, l: f32, a: f32) -> f32 {
  let k = (n + h * 12.0) - floor((n + h * 12.0) / 12.0) * 12.0;
  return l - a * max(-1.0, min(min(k - 3.0, 9.0 - k), 1.0));
}

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
  let h = hsl.x / 360.0;
  let s = hsl.y;
  let l = hsl.z;
  let a = s * min(l, 1.0 - l);
  return vec3(
    hueComponent(0.0, h, l, a),
    hueComponent(8.0, h, l, a),
    hueComponent(4.0, h, l, a),
  );
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let uv = in.uv;
  let w = p.width;
  let h = p.height;
  let amount = p.intensity;
  var col: vec4<f32> = textureSample(src, samp, uv);

  if (p.mode == 1u) {
    // glitch — mirrors applyGlitch (screenEffects.ts): N random horizontal
    // slices displaced by ±(0..20)px at 0.6 alpha, plus a color bar that
    // opens periodically when intensity > 0.3.
    let now = floor(p.time * 1000.0);
    var seed = u32(now);
    let sliceCount = 3u + u32(amount * 12.0);
    let y = uv.y * h;
    var off = 0.0;
    var inSlice = false;
    for (var i = 0u; i < sliceCount; i++) {
      let r0 = mulberry32(&seed);
      let r1 = mulberry32(&seed);
      let r2 = mulberry32(&seed);
      let sliceY = r0 * h;
      let sliceH = 2.0 + r1 * 8.0 * amount;
      if (y >= sliceY && y < sliceY + sliceH) {
        off = (r2 - 0.5) * 40.0 * amount;
        inSlice = true;
      }
    }
    // A slice is only painted where its shifted destination overlaps the
    // canvas (drawImage clips the dest rect); elsewhere the original shows.
    let shifted = uv.x - off / w;
    if (inSlice && shifted >= 0.0 && shifted <= 1.0) {
      col = mix(textureSample(src, samp, uv), textureSample(src, samp, vec2(shifted, uv.y)), 0.6);
    }
    if (amount > 0.3) {
      // The color bar is drawn on the first frame where now - lastGlitchTime
      // > 200 (applyGlitch). lastGlitchTime = now of the previous bar, so with
      // a fixed fps the bar fires on frames k = N, 2N, ... where
      // N = floor(0.2 * fps) + 1 (k0 = N since the first fire needs now > 200).
      let k = round(p.time * p.fps);
      let n = floor(0.2 * p.fps) + 1.0;
      if (k >= n && k - n * floor(k / n) == 0.0) {
        let gH = 1.0 + mulberry32(&seed) * 4.0 * amount;
        let gY = mulberry32(&seed) * h;
        let gX = mulberry32(&seed) * w * 0.3;
        let gW = w * (0.3 + mulberry32(&seed) * 0.7);
        let cr = mulberry32(&seed);
        let cg = mulberry32(&seed);
        let cb = mulberry32(&seed);
        let ba = 0.3 + mulberry32(&seed) * 0.4;
        let x = uv.x * w;
        if (x >= gX && x < gX + gW && y >= gY && y < gY + gH) {
          let bc = vec3<f32>(
            select(0.0, 1.0, cr > 0.5),
            select(0.0, 1.0, cg > 0.5),
            select(0.0, 1.0, cb > 0.5));
          col = mix(col, vec4(bc, 1.0), ba);
        }
      }
    }
  } else if (p.mode == 2u) {
    // chromatic — mirrors applyChromatic: two shifted snapshot ghosts drawn
    // with the canvas 'screen' composite at alpha min(0.7, amount).
    let offset = max(2.0, amount * 14.0);
    let a = min(0.7, amount);
    let ox = offset / w;
    var acc = textureSample(src, samp, uv);
    // Each ghost is only painted where its shifted dest rect overlaps the
    // canvas (drawImage clips); elsewhere the frame shows through.
    if (uv.x <= 1.0 - ox) {
      let cL = textureSample(src, samp, uv + vec2(ox, 0.0));
      acc = 1.0 - (1.0 - a * cL) * (1.0 - acc);
    }
    if (uv.x >= ox) {
      let cR = textureSample(src, samp, uv - vec2(ox, 0.0));
      acc = 1.0 - (1.0 - a * cR) * (1.0 - acc);
    }
    col = vec4(acc.rgb, 1.0);
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
    // bars: slightly zoomed snapshot with black letterbox bands at 0.96 alpha
    // (mirrors applyBars: rgba(0,0,0,0.96) drawn over the zoomed snapshot).
    let scale = 1.0 + amount * 0.12;
    let barH = (amount * h * 0.22) / h;
    col = textureSample(src, samp, vec2(0.5) + (uv - vec2(0.5)) / scale);
    if (uv.y < barH || uv.y > 1.0 - barH) {
      col = mix(col, vec4(0.0, 0.0, 0.0, 1.0), 0.96);
    }
  } else if (p.mode == 6u) {
    // shockwave: radial ripple. TS warps a 0.25-scale snapshot then upscales:
    // phase and pull live on the small-canvas grid (so the ring frequency
    // matches), and shifted samples wrap like the TS `%` indexing.
    let c = vec2(w, h) * 0.5;
    let d = uv * vec2(w, h) - c;
    let dist = length(d);
    let maxDist = max(length(c), 1.0);
    let sw = max(2.0, round(w * 0.25));
    let sh = max(2.0, round(h * 0.25));
    let ds = dist * sw / w;
    let phase = ds * 0.13 - p.time * 7.0;
    let pull = amount * 5.0 * (dist / maxDist) * sin(phase * 6.2831853);
    let dir = select(vec2(0.0), d / max(dist, 0.0001), dist > 0.0001);
    let uv2 = uv + dir * pull / vec2(sw, sh);
    col = textureSample(src, samp, uv2 - floor(uv2));
  } else if (p.mode == 7u) {
    // pixelate: replicate applyPixelate's two-step nearest scaling (downsample
    // to ceil(w/block), upsample back). Each output block shows the source
    // pixel at floor(floor(x*sw/w) * w/sw), not the block center.
    let block = max(2.0, round(4.0 + amount * 44.0));
    let sw = max(1.0, ceil(w / block));
    let sh = max(1.0, ceil(h / block));
    let sx = floor(floor(uv.x * sw) * w / sw);
    let sy = floor(floor(uv.y * sh) * h / sh);
    col = textureSample(src, samp, vec2(sx, sy) / vec2(w, h));
  } else if (p.mode == 8u) {
    // tilt: rotate around the center (mirrors applyTilt: mulberry32 angle,
    // cleared canvas outside the rotated frame -> black).
    var randState = u32(floor(p.time * 1000.0));
    let angle = (mulberry32(&randState) - 0.5) * amount * 0.08;
    let d = uv - vec2(0.5);
    let ca = cos(angle);
    let sa = sin(angle);
    let u2 = vec2(d.x * ca - d.y * sa, d.x * sa + d.y * ca) + vec2(0.5);
    if (u2.x < 0.0 || u2.x > 1.0 || u2.y < 0.0 || u2.y > 1.0) {
      col = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
      col = textureSample(src, samp, u2);
    }
  } else if (p.mode == 9u) {
    // heat haze: horizontal strips shifted by a slow sine; the shifted
    // destination leaves cleared (black) edges (mirrors applyHeatHaze).
    let y = uv.y * h;
    let xOff = sin((y + p.time * 1000.0 / 28.0) * 0.05) * amount * 18.0;
    let u = uv.x - xOff / w;
    if (u < 0.0 || u > 1.0) {
      col = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
      col = textureSample(src, samp, vec2(u, uv.y));
    }
  } else if (p.mode == 10u) {
    // hue shift — mirrors applyHueShift: canvas 'hue' composite of a diagonal
    // hue->hue+180 gradient (hsl 0.85/0.5) at alpha `amount` over the frame.
    let hue = p.time * 25.0 - floor(p.time * 25.0 / 360.0) * 360.0;
    let t = clamp((uv.x * w * w + uv.y * h * h) / (w * w + h * h), 0.0, 1.0);
    let hue2 = hue + 180.0 * t;
    let cs = hsl_to_rgb(vec3(hue2 - floor(hue2 / 360.0) * 360.0, 0.85, 0.5));
    let cb = textureSample(src, samp, uv).rgb;
    col = vec4(mix(cb, hueBlend(cb, cs), p.intensity), 1.0);
  } else if (p.mode == 11u) {
    // glass crack — organic jagged fault lines, micro-shards, and bright fracture line highlights.
    let beatshake = select(0.0, p.beat * 0.015, p.beat > 0.15);
    let uvt = uv + vec2(beatshake * sin(p.time * 30.0), beatshake * cos(p.time * 25.0));

    // Primary sweeping jagged fault lines traversing across the canvas
    let seam1 = abs(uvt.y - 0.46 + sin(uvt.x * 5.0 + 0.8) * 0.14 + sin(uvt.x * 22.0) * 0.02 + cos(uvt.x * 45.0) * 0.008);
    let seam2 = abs(uvt.y - (uvt.x * 0.55 + 0.18) + sin(uvt.x * 12.0) * 0.035 + cos(uvt.x * 33.0) * 0.012);
    let seam3 = abs(uvt.y - (0.95 - uvt.x * 0.7) + cos(uvt.x * 9.0 + 1.1) * 0.04 + sin(uvt.x * 28.0) * 0.015);
    let seam4 = abs(uvt.x - 0.72 + sin(uvt.y * 14.0 + 0.5) * 0.03 + cos(uvt.y * 38.0) * 0.01);

    // Minimum distance to nearest main fault seam
    let minSeam = min(min(seam1, seam2), min(seam3, seam4));

    // Micro-shard slivers clustered along primary fault seams
    let microNoise = sin(uvt.y * 60.0 + sin(uvt.x * 50.0) * 4.0) * 0.5 + 0.5;
    let microSeam = select(1.0, microNoise * 0.02, minSeam < 0.08);

    let crackDist = min(minSeam, microSeam);

    // Refractive shard displacement per glass plate
    let shardId = floor(uvt.y * 6.0 + sin(uvt.x * 8.0) * 3.0 + select(0.0, 5.0, minSeam < 0.05));
    let shardShift = vec2(sin(shardId * 17.1), cos(shardId * 11.3)) * amount * 0.022;

    let sampleUv = clamp(uv + shardShift, vec2(0.0), vec2(1.0));
    var sampled = textureSample(src, samp, sampleUv);

    let widthThreshold = (0.006 + amount * 0.012);
    if (crackDist < widthThreshold) {
      let edgeStrength = clamp(1.0 - crackDist / widthThreshold, 0.0, 1.0) * amount;
      let whiteHighlight = vec4(1.0, 1.0, 1.0, 1.0);
      col = mix(sampled, whiteHighlight, edgeStrength * 0.9);
    } else {
      col = sampled;
    }
  }

  return col;
}
