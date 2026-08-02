//! Phase 5: complex particle + 3D-style renderers.
//! Ports `flameFire`, `spiralGalaxy`, `threeD`, `api3D`, `neonCity3D`,
//! `speaker3D`, `speakerTrio`, `speakerSplatter` from
//! `src/services/renderers/*.ts`. Canvas composite ops (`lighter`/`screen`)
//! and offset shadows are approximated with alpha/glow where noted.

use std::f32::consts::TAU;

use crate::gpu2d::text::{TextAlign, TextOpts};
use crate::gpu2d::{Color, Fill, GpuCanvas, LineCap};

use super::RenderContext;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct FireParticle {
  x: f32,
  y: f32,
  vx: f32,
  vy: f32,
  size: f32,
  alpha: f32,
  life: f32,
  max_life: f32,
}

pub struct GalaxyParticle {
  angle: f32,
  radius: f32,
  speed: f32,
  size: f32,
  arm: u32,
}

pub struct Spark {
  x: f32,
  y: f32,
  vx: f32,
  vy: f32,
  life: f32,
  max_life: f32,
  size: f32,
  color: Color,
  decay: f32,
  trail: Vec<(f32, f32)>,
}

pub struct LightMote {
  x: f32,
  y: f32,
  vx: f32,
  vy: f32,
  size: f32,
  alpha: f32,
  phase: f32,
}

pub struct Peak {
  x: f32,
  y: f32,
  alpha: f32,
}

pub struct Ember {
  x: f32,
  y: f32,
  vx: f32,
  vy: f32,
  size: f32,
  life: f32,
  max_life: f32,
}

pub struct FloatingNote {
  x: f32,
  y: f32,
  vx: f32,
  vy: f32,
  symbol: char,
  size: f32,
  alpha: f32,
  rotation: f32,
  rot_speed: f32,
}

pub struct SplatterDot {
  x: f32,
  y: f32,
  r: f32,
}

pub struct AdvancedState {
  pub fire: Vec<FireParticle>,
  pub galaxy: Vec<GalaxyParticle>,
  pub galaxy_init: bool,
  pub sparks: Vec<Spark>,
  pub motes: Vec<LightMote>,
  pub peaks: Vec<Peak>,
  pub three_d_prev_beat: f32,
  pub three_d_rot: f32,
  pub embers: Vec<Ember>,
  pub api_time: f32,
  pub frame_history: Vec<Vec<u8>>,
  pub notes: Vec<FloatingNote>,
  pub splatter: Vec<SplatterDot>,
  pub arc_rotation: f32,
}

impl Default for AdvancedState {
  fn default() -> Self {
    AdvancedState {
      fire: Vec::new(),
      galaxy: Vec::new(),
      galaxy_init: false,
      sparks: Vec::new(),
      motes: Vec::new(),
      peaks: Vec::new(),
      three_d_prev_beat: 0.0,
      three_d_rot: 0.0,
      embers: Vec::new(),
      api_time: 0.0,
      frame_history: Vec::new(),
      notes: Vec::new(),
      splatter: Vec::new(),
      arc_rotation: 0.0,
    }
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + (b - a) * t
}

fn mix(p: Color, s: Color, t: f32) -> Color {
  Color::rgba(lerp(p.r, s.r, t), lerp(p.g, s.g, t), lerp(p.b, s.b, t), 1.0)
}

fn bright(c: Color, f: f32) -> Color {
  Color::rgba(
    (c.r * f).clamp(0.0, 1.0),
    (c.g * f).clamp(0.0, 1.0),
    (c.b * f).clamp(0.0, 1.0),
    1.0,
  )
}

fn bin_sum(freq: &[u8], step: usize, idx: usize) -> f32 {
  let mut sum = 0usize;
  let mut n = 0;
  for j in 0..step {
    let k = idx * step + j;
    if k < freq.len() {
      sum += freq[k] as usize;
      n += 1;
    }
  }
  if n == 0 {
    return 0.0;
  }
  sum as f32 / (n as f32 * 255.0)
}

/// Canvas `quadraticCurveTo(prev, mid)` woven polyline.
fn quadratic_wave(raw: &[(f32, f32)], steps: u32) -> Vec<(f32, f32)> {
  if raw.is_empty() {
    return Vec::new();
  }
  let mut out = Vec::with_capacity(raw.len() * (steps as usize + 1));
  let mut prev_end = raw[0];
  out.push(prev_end);
  for i in 1..raw.len() {
    let ctrl = raw[i - 1];
    let end = ((raw[i - 1].0 + raw[i].0) / 2.0, (raw[i - 1].1 + raw[i].1) / 2.0);
    let seg = GpuCanvas::sample_quadratic(prev_end, ctrl, end, steps);
    for p in seg.iter().skip(1) {
      out.push(*p);
    }
    prev_end = end;
  }
  out
}

// ---------------------------------------------------------------------------
// flameFire
// ---------------------------------------------------------------------------

pub fn flame_fire(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let sensitivity = ctx.config.reactivity.sensitivity;
  let theme = &ctx.config.theme;
  let p = super::theme_primary(theme);
  let s = super::theme_secondary(theme);
  let glow = super::theme_glow(theme);
  let be = ctx.bass_energy;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  if !st.fire.is_empty() || be > 0.02 {
    let spawn = (2.0 + be * 8.0 * sensitivity).floor() as usize;
    for _ in 0..spawn {
      st.fire.push(FireParticle {
        x: rng.next() * width,
        y: height - rng.next() * height * 0.05,
        vy: -(0.5 + rng.next() + be * 3.0),
        vx: (rng.next() - 0.5) * 0.5,
        size: 2.0 + rng.next() * 4.0 + be * 4.0,
        alpha: 0.5 + rng.next() * 0.5,
        life: 0.0,
        max_life: 40.0 + rng.next() * 30.0 + be * 30.0,
      });
    }
  }
  while st.fire.len() > 300 {
    st.fire.remove(0);
  }

  let mut i = st.fire.len();
  while i > 0 {
    i -= 1;
    let remove = {
      let fp = &mut st.fire[i];
      fp.life += 1.0;
      if fp.life >= fp.max_life {
        true
      } else {
        fp.x += fp.vx + (rng.next() - 0.5) * 0.3;
        fp.y += fp.vy;
        fp.vy += 0.02;
        fp.alpha *= 0.99;
        false
      }
    };
    if remove {
      st.fire.remove(i);
      continue;
    }
    let (fx, fy, size, alpha) = {
      let fp = &st.fire[i];
      let t = fp.life / fp.max_life;
      (fp.x, fp.y, fp.size * (1.0 - t * 0.7), fp.alpha * (1.0 - t))
    };
    let t = (st.fire[i].life / st.fire[i].max_life).min(1.0);
    let col = Color::rgba(
      lerp(p.r, s.r, t),
      lerp(p.g, s.g, t) * 0.5,
      lerp(p.b, s.b, t) * 0.2,
      1.0,
    );
    c.set_fill(Fill::Solid(col));
    c.set_global_alpha(alpha.clamp(0.0, 1.0));
    c.set_shadow(glow, 15.0);
    c.fill_circle(fx, fy, size);
  }

  let high_sum: f32 =
    ctx.freq_data.get(24..48).map(|w| w.iter().map(|&b| b as f32).sum::<f32>()).unwrap_or(0.0)
      / (24.0 * 255.0);
  if high_sum > 0.2 {
    let n = (high_sum * 5.0 * sensitivity).floor() as usize;
    for _ in 0..n {
      st.fire.push(FireParticle {
        x: rng.next() * width,
        y: height - 10.0,
        vy: -(1.0 + rng.next() * 2.0 + high_sum * 4.0),
        vx: (rng.next() - 0.5) * 1.5,
        size: 1.0 + rng.next() * 2.0,
        alpha: 1.0,
        life: 0.0,
        max_life: 15.0 + rng.next() * 10.0,
      });
    }
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
}

// ---------------------------------------------------------------------------
// spiralGalaxy
// ---------------------------------------------------------------------------

pub fn spiral_galaxy(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = super::theme_primary(theme);
  let s = super::theme_secondary(theme);
  let glow = super::theme_glow(theme);
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  if !st.galaxy_init {
    st.galaxy_init = true;
    for _ in 0..400 {
      let arm = (rng.next() * 3.0) as usize;
      let r = rng.next();
      st.galaxy.push(GalaxyParticle {
        angle: rng.next() * TAU + arm as f32 * 2.1,
        radius: r,
        speed: 0.002 + (1.0 - r) * 0.008,
        size: 0.5 + r * 2.5,
        arm: arm as u32,
      });
    }
  }

  let cx = width / 2.0;
  let cy = height / 2.0;
  let max_r = width.min(height) * 0.45;
  let rot_speed = 0.003 + be * 0.01 + bs * 0.02;
  let glow_intensity = 0.5 + be * 1.5;

  for gp in st.galaxy.iter_mut() {
    gp.angle += gp.speed + rot_speed;
    let dist = gp.radius * max_r;
    let spiral_offset = gp.radius * 0.5;
    let a = gp.angle + gp.arm as f32 * 2.1 + gp.radius * 3.0;
    let x = cx + a.cos() * (dist + (gp.angle * 3.0 + gp.arm as f32).sin() * spiral_offset);
    let y = cy + a.sin() * (dist + (gp.angle * 3.0 + gp.arm as f32).cos() * spiral_offset);
    let alpha = (0.3 + gp.radius * 0.4) * (0.5 + be * 0.5);
    let size = gp.size * (1.0 + be * 0.5);
    let col = mix(p, s, gp.radius);
    c.set_global_alpha(alpha.clamp(0.0, 1.0));
    c.set_fill(Fill::Solid(col));
    c.set_shadow(glow, size * 3.0 * glow_intensity);
    c.fill_circle(x, y, size);
  }

  c.set_fill(Fill::Solid(Color::WHITE));
  c.set_shadow(glow, 20.0 * glow_intensity);
  c.set_global_alpha((0.8 + be * 0.2).clamp(0.0, 1.0));
  c.fill_circle(cx, cy, 2.0 + be * 4.0);

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
}

// ---------------------------------------------------------------------------
// threeD
// ---------------------------------------------------------------------------

struct BarInfo {
  x: f32,
  by: f32,
  bh: f32,
  bw: f32,
  dy: f32,
  val: f32,
}

pub fn three_d(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = super::theme_primary(theme);
  let s = super::theme_secondary(theme);
  let a = super::theme_accent(theme);
  let g = super::theme_glow(theme);
  let bar_count = 48.min(ctx.config.reactivity.bar_count.max(1));
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  let center_x = width / 2.0;
  let floor_y = height * 0.78;
  let persp = 0.3;
  let max_bar_w = 18.0;
  let gap = 2.0;
  let bar_step = max_bar_w + gap;
  let step = (ctx.freq_data.len() / bar_count).max(1);

  if st.motes.is_empty() {
    for _ in 0..50 {
      st.motes.push(LightMote {
        x: rng.next() * width,
        y: rng.next() * height * 0.7,
        vx: (rng.next() - 0.5) * 0.2,
        vy: -0.05 - rng.next() * 0.15,
        size: 1.0 + rng.next() * 2.5,
        alpha: 0.15 + rng.next() * 0.35,
        phase: rng.next() * TAU,
      });
    }
  }

  if bs > 0.1 && bs > st.three_d_prev_beat {
    let bar_hs: Vec<f32> = (0..bar_count).map(|i| bin_sum(ctx.freq_data, step, i) * sensitivity).collect();
    let spark_count = (8.0 + bs * 25.0).floor() as usize;
    for _ in 0..spark_count {
      let bi = (rng.next() * bar_count as f32) as usize % bar_count;
      let bh = bar_hs[bi] * height * 0.38;
      let x = center_x - (bar_count as f32 * bar_step) / 2.0 + bi as f32 * bar_step
        + (rng.next() - 0.5) * 12.0;
      let cd = (x - center_x) / center_x;
      let ps = 1.0 - cd.abs() * persp;
      let py = floor_y - bh * ps - 5.0;
      st.sparks.push(Spark {
        x,
        y: py,
        vx: (rng.next() - 0.5) * 4.0,
        vy: -3.0 - rng.next() * 5.0,
        life: 0.0,
        max_life: 25.0 + rng.next() * 40.0,
        size: 1.5 + rng.next() * 3.0,
        color: mix(p, s, rng.next()),
        decay: 0.96 + rng.next() * 0.03,
        trail: Vec::new(),
      });
    }
  }
  st.three_d_prev_beat = bs;

  let mut i = st.sparks.len();
  while i > 0 {
    i -= 1;
    let remove = {
      let sp = &mut st.sparks[i];
      sp.trail.push((sp.x, sp.y));
      if sp.trail.len() > 6 {
        sp.trail.remove(0);
      }
      sp.life += 1.0;
      sp.x += sp.vx;
      sp.vy *= sp.decay;
      sp.vy += 0.04;
      sp.y += sp.vy;
      sp.vx *= 0.99;
      sp.life > sp.max_life || sp.y > floor_y || sp.y < -30.0
    };
    if remove {
      st.sparks.remove(i);
    }
  }

  st.three_d_rot += 0.002 + be * 0.004;
  let rot = st.three_d_rot;

  for m in st.motes.iter_mut() {
    m.x += m.vx + (rot + m.phase).sin() * 0.15;
    m.y += m.vy + (rot * 0.5 + m.phase).cos() * 0.05;
    if m.y < -15.0 {
      m.y = height * 0.7;
      m.x = rng.next() * width;
    }
    if m.x < -15.0 {
      m.x = width + 15.0;
    }
    if m.x > width + 15.0 {
      m.x = -15.0;
    }
  }

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);

  let floor_grad = Fill::linear_gradient(0.0, floor_y, 0.0, height, &[
    (0.0, p.with_alpha(0.3)),
    (0.4, s.with_alpha(0.12)),
    (1.0, g.with_alpha(0.0)),
  ]);
  c.set_fill(floor_grad);
  c.fill_rect(0.0, floor_y, width, height - floor_y);

  c.set_stroke(Fill::Solid(g.with_alpha(0.05)));
  c.set_line_width(1.0);
  for gi in -35i32..=35 {
    let gx = center_x + gi as f32 * 12.0;
    if gx < 0.0 || gx > width {
      continue;
    }
    c.stroke_line(gx, floor_y + gi.unsigned_abs() as f32 * 0.25, gx, height);
  }

  let total_w = (max_bar_w + gap) * bar_count as f32;
  let start_x = center_x - total_w / 2.0;

  let mut bars: Vec<BarInfo> = Vec::with_capacity(bar_count);
  for i in 0..bar_count {
    let val = bin_sum(ctx.freq_data, step, i) * sensitivity;
    let bar_h = 2.0f32.max(val * height * 0.38);
    let x = start_x + i as f32 * (max_bar_w + gap);
    let cd = (x - center_x) / (total_w / 2.0);
    let ps = 1.0 - cd.abs() * persp;
    let bw = 2.0f32.max(max_bar_w * ps);
    let bh = bar_h * ps;
    let by = floor_y - bh;
    let depth = 1.0f32.max(bw * 0.4 * ps);
    let dx = depth * 0.7;
    let dy = depth * 0.5;
    bars.push(BarInfo { x, by, bh, bw, dy, val });

    let freq_ratio = i as f32 / bar_count as f32;
    let base = mix(p, s, freq_ratio);
    let bright_f = 0.4 + val * 0.4;
    let bar_c = bright(base, bright_f);
    let top_c = bright(base, bright_f * 1.3);
    let side_c = bright(base, bright_f * 0.6);
    let bright_boost = 0.3 + be * 0.3 + if bs > 0.12 { bs * 0.5 } else { 0.0 };

    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_fill(Fill::Solid(bar_c));
    c.fill_rect(x, by, bw, bh);

    let pulse = 1.0 + be * 0.2 + (rot + i as f32 * 0.3).sin() * be * 0.05;

    c.set_fill(Fill::Solid(top_c));
    c.set_global_alpha((0.6 + be * 0.15).clamp(0.0, 1.0));
    c.fill_polygon(&[
      (x, by),
      (x + dx, by - dy * pulse),
      (x + bw + dx, by - dy * pulse),
      (x + bw, by),
    ]);

    c.set_fill(Fill::Solid(side_c));
    c.set_global_alpha((0.4 + be * 0.15).clamp(0.0, 1.0));
    c.fill_polygon(&[
      (x + bw, by),
      (x + bw + dx, by - dy * pulse),
      (x + bw + dx, floor_y - dy * pulse),
      (x + bw, floor_y),
    ]);

    c.set_shadow(g, 12.0 + be * 18.0);
    c.set_fill(Fill::Solid(bar_c));
    c.set_global_alpha((bright_boost * 0.4).clamp(0.0, 1.0));
    c.fill_rect(x, by, bw, bh.min(5.0));
    c.set_shadow(g, 6.0 + be * 8.0);
    c.fill_rect(x, by, bw, bh);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_global_alpha(1.0);

    if be > 0.25 {
      c.set_fill(Fill::Solid(a.with_alpha((be - 0.25) * 0.35)));
      c.fill_rect(x, by, bw, bh.min(2.0));
    }
  }

  st.peaks.retain(|pk| pk.alpha > 0.01);
  for b in &bars {
    if b.val * 0.35 > 0.8 {
      let cx = b.x + b.bw / 2.0;
      if !st.peaks.iter().any(|pk| (pk.x - cx).abs() < 5.0) {
        st.peaks.push(Peak { x: cx, y: b.by, alpha: 1.0 });
      }
    }
  }
  for pk in st.peaks.iter_mut() {
    pk.alpha *= 0.94;
    pk.y -= 0.3;
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);

  let mut tips = Vec::with_capacity(bars.len());
  for (i, b) in bars.iter().enumerate() {
    tips.push((
      b.x + b.bw / 2.0,
      b.by - b.dy * (1.0 + be * 0.2 + (rot + i as f32 * 0.3).sin() * be * 0.05),
    ));
  }
  c.set_stroke(Fill::Solid(p));
  c.set_global_alpha((0.2 + be * 0.2).clamp(0.0, 1.0));
  c.set_line_width(2.0);
  c.set_shadow(g, 10.0);
  c.stroke_polyline(&tips);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);

  for (i, b) in bars.iter().enumerate() {
    if b.val < 0.05 {
      continue;
    }
    let tip_x = b.x + b.bw / 2.0;
    let tip_y = b.by - b.dy * (1.0 + be * 0.2 + (rot + i as f32 * 0.3).sin() * be * 0.05);
    c.set_fill(Fill::Solid(g));
    c.set_global_alpha((b.val * 0.15 + be * 0.1).clamp(0.0, 1.0));
    c.set_shadow(g, 20.0);
    c.fill_circle(tip_x, tip_y, 2.0 + b.val * 3.0);
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);

  for pk in &st.peaks {
    c.set_fill(Fill::Solid(a));
    c.set_global_alpha((pk.alpha * 0.6).clamp(0.0, 1.0));
    c.set_shadow(a, 8.0);
    c.fill_circle(pk.x, pk.y, 2.5);
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);

  for sp in &st.sparks {
    let progress = (sp.life / sp.max_life).min(1.0);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    for (ti, tp) in sp.trail.iter().enumerate() {
      let tfrac = ti as f32 / sp.trail.len() as f32;
      c.set_fill(Fill::Solid(sp.color));
      c.set_global_alpha(((1.0 - progress) * tfrac * 0.4).clamp(0.0, 1.0));
      c.fill_circle(tp.0, tp.1, sp.size * tfrac * 0.5);
    }
    c.set_fill(Fill::Solid(sp.color));
    c.set_global_alpha((1.0 - progress).clamp(0.0, 1.0));
    c.set_shadow(sp.color, 10.0);
    c.fill_circle(sp.x, sp.y, sp.size * (1.0 - progress * 0.4));
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);

  c.set_global_alpha(0.5);
  for (i, b) in bars.iter().enumerate() {
    if b.val < 0.03 {
      continue;
    }
    let ref_alpha = (0.08 - (i as f32 - bar_count as f32 / 2.0).abs() * 0.003).max(0.0);
    c.set_fill(Fill::Solid(mix(p, s, i as f32 / bar_count as f32)));
    c.set_global_alpha(ref_alpha);
    c.fill_rect(b.x, floor_y + 4.0, b.bw, b.bh * 0.25);
  }
  c.set_global_alpha(1.0);

  for m in &st.motes {
    c.set_global_alpha(
      (m.alpha * (0.4 + (rot * 2.0 + m.phase).sin() * 0.25)).clamp(0.0, 1.0),
    );
    c.set_fill(Fill::Solid(p));
    c.set_shadow(g, 4.0);
    c.fill_circle(m.x, m.y, m.size * (0.8 + be * 0.3));
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);
  c.restore();
}

// ---------------------------------------------------------------------------
// api3D
// ---------------------------------------------------------------------------

pub fn api_3d(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let sensitivity = ctx.config.reactivity.sensitivity;
  let theme = &ctx.config.theme;
  let a = super::theme_accent(theme);
  let p = super::theme_primary(theme);
  let s = super::theme_secondary(theme);
  let glow = super::theme_glow(theme);
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  let fire_width_ratio = 0.94;
  let fire_height_scale = 1.0;

  st.api_time += 0.03 + be * 0.02;
  let time = st.api_time;

  let center_y = height * 0.5;
  let half_margin = (1.0 - fire_width_ratio) / 2.0;
  let start_x = width * 0.01f32.max(half_margin);
  let end_x = width * 0.99f32.min(1.0 - half_margin);
  let wave_width = end_x - start_x;
  let point_count = 220usize;

  let mut disp = vec![0.0f32; point_count];
  let mut sub1 = vec![0.0f32; point_count];
  let mut sub2 = vec![0.0f32; point_count];

  let bin_count = (ctx.freq_data.len() / 4).clamp(1, 48);
  let step = (ctx.freq_data.len() / bin_count).max(1);

  for b in 0..bin_count {
    let val = bin_sum(ctx.freq_data, step, b) * sensitivity;
    if val < 0.04 {
      continue;
    }
    let bin_ratio = b as f32 / bin_count as f32;
    let peak_x = start_x + bin_ratio * wave_width;
    let is_downward = b % 5 == 2 || b % 7 == 4;
    let sign = if is_downward { 1.0 } else { -1.0 };
    let peak_h = val * height * 0.32 * fire_height_scale * sign;
    let sigma = 16.0 + val * 10.0;
    for i in 0..point_count {
      let px = start_x + (i as f32 / (point_count - 1) as f32) * wave_width;
      let d = px - peak_x;
      let gaussian = (-(d * d) / (2.0 * sigma * sigma)).exp();
      disp[i] += peak_h * gaussian;
      let d1 = d - 12.0;
      sub1[i] += peak_h * 0.65 * (-(d1 * d1) / (2.0 * sigma * sigma)).exp();
      let d2 = d + 15.0;
      sub2[i] += peak_h * 0.45 * (-(d2 * d2) / (2.0 * sigma * sigma)).exp();
    }
    if val > 0.25 && rng.next() < 0.25 + bs * 0.3 {
      st.embers.push(Ember {
        x: peak_x + (rng.next() - 0.5) * 12.0,
        y: center_y + peak_h * 0.8 + (rng.next() - 0.5) * 10.0,
        vx: (rng.next() - 0.5) * 1.5,
        vy: (rng.next() - 0.5) * 1.5,
        size: 0.6 + rng.next() * 2.0,
        life: 0.0,
        max_life: 25.0 + rng.next() * 35.0,
      });
    }
  }

  let px_of = |i: usize| start_x + (i as f32 / (point_count - 1) as f32) * wave_width;

  c.save();
  c.set_shadow(glow, 40.0 + be * 20.0);
  let cloud = Fill::linear_gradient(0.0, center_y - height * 0.2, 0.0, center_y + height * 0.2, &[
    (0.0, Color::TRANSPARENT),
    (0.5, s.with_alpha(0.2)),
    (1.0, Color::TRANSPARENT),
  ]);
  c.set_fill(cloud);
  c.fill_rect(start_x, center_y - height * 0.25, wave_width, height * 0.5);

  c.set_shadow(glow, 14.0);
  c.set_stroke(Fill::Solid(p));
  c.set_global_alpha(0.75);
  c.set_line_width(1.6);
  c.stroke_line(start_x, center_y, end_x, center_y);
  c.set_global_alpha(1.0);

  c.set_shadow(glow, 10.0);
  c.set_line_width(1.2);
  let raw1: Vec<(f32, f32)> = (0..point_count)
    .map(|i| (px_of(i), center_y + sub1[i] + (i as f32 * 0.1 + time * 4.0).sin() * 4.0))
    .collect();
  c.set_stroke(Fill::Solid(s));
  c.set_global_alpha(0.45);
  c.stroke_polyline(&quadratic_wave(&raw1, 5));

  c.set_stroke(Fill::Solid(a));
  c.set_global_alpha(0.35);
  c.set_line_width(1.0);
  let raw2: Vec<(f32, f32)> = (0..point_count)
    .map(|i| (px_of(i), center_y + sub2[i] + (i as f32 * 0.12 - time * 3.5).cos() * 5.0))
    .collect();
  c.stroke_polyline(&quadratic_wave(&raw2, 5));
  c.set_global_alpha(1.0);

  c.set_shadow(glow, 15.0);
  c.set_line_width(1.2);
  for i in (0..point_count).step_by(3) {
    let abs_disp = disp[i].abs();
    if abs_disp > 35.0 {
      let px = px_of(i);
      let needle_h = abs_disp * 1.4;
      let needle_grad = Fill::linear_gradient(
        0.0,
        center_y - needle_h * 0.5,
        0.0,
        center_y + needle_h * 0.5,
        &[
          (0.0, Color::rgba(1.0, 1.0, 1.0, 0.0)),
          (0.5, a.with_alpha(0.8)),
          (1.0, Color::rgba(1.0, 1.0, 1.0, 0.0)),
        ],
      );
      c.set_stroke(needle_grad);
      let y0 = if disp[i] < 0.0 { center_y - needle_h } else { center_y - 10.0 };
      let y1 = if disp[i] > 0.0 { center_y + needle_h } else { center_y + 10.0 };
      c.stroke_line(px, y0, px, y1);
    }
  }

  let raw_hero: Vec<(f32, f32)> =
    (0..point_count).map(|i| (px_of(i), center_y + disp[i])).collect();
  let hero = quadratic_wave(&raw_hero, 5);

  c.set_shadow(glow, 28.0 + be * 18.0);
  c.set_stroke(Fill::Solid(s));
  c.set_global_alpha(0.65);
  c.set_line_width(7.5);
  c.stroke_polyline(&hero);
  c.set_global_alpha(1.0);

  c.set_shadow(glow, 16.0);
  c.set_stroke(Fill::Solid(p));
  c.set_line_width(3.6);
  c.stroke_polyline(&hero);

  c.set_shadow(a, 8.0);
  c.set_stroke(Fill::Solid(a));
  c.set_line_width(1.8);
  c.stroke_polyline(&hero);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let raw_refl: Vec<(f32, f32)> =
    (0..point_count).map(|i| (px_of(i), center_y - disp[i] * 0.45)).collect();
  c.set_shadow(glow, 12.0);
  c.set_stroke(Fill::Solid(s));
  c.set_global_alpha(0.2);
  c.set_line_width(2.2);
  c.stroke_polyline(&quadratic_wave(&raw_refl, 5));
  c.set_global_alpha(1.0);

  let mut i = st.embers.len();
  while i > 0 {
    i -= 1;
    let remove = {
      let e = &mut st.embers[i];
      e.life += 1.0;
      e.x += e.vx;
      e.y += e.vy;
      e.life / e.max_life >= 1.0
    };
    if remove {
      st.embers.remove(i);
      continue;
    }
    let (ex, ey, size, alpha) = {
      let e = &st.embers[i];
      let progress = e.life / e.max_life;
      (e.x, e.y, e.size * (1.0 - progress * 0.3), (1.0 - progress) * 0.85)
    };
    c.set_shadow(glow, 6.0);
    c.set_fill(Fill::Solid(p));
    c.set_global_alpha(alpha.clamp(0.0, 1.0));
    c.fill_circle(ex, ey, size);
    c.set_global_alpha(1.0);
  }
  if st.embers.len() > 180 {
    let n = st.embers.len() - 180;
    st.embers.drain(0..n);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);
  c.restore();
}

// ---------------------------------------------------------------------------
// neonCity3D
// ---------------------------------------------------------------------------

const HISTORY_DEPTH: usize = 12;

pub fn neon_city_3d(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = super::theme_primary(theme);
  let s = super::theme_secondary(theme);
  let bar_count = 64.min(36.max(ctx.config.reactivity.bar_count));
  let sensitivity = ctx.config.reactivity.sensitivity;
  let st = &mut ctx.state.advanced;

  let center_x = width / 2.0;
  let floor_y = height * 0.58;
  let step = (ctx.freq_data.len() / bar_count).max(1);

  if st.frame_history.first().map(|f| f.len()) != Some(ctx.freq_data.len()) {
    st.frame_history.clear();
  }
  st.frame_history.insert(0, ctx.freq_data.to_vec());
  if st.frame_history.len() > HISTORY_DEPTH {
    st.frame_history.pop();
  }

  let rows = st.frame_history.len();
  let cols = bar_count;
  let total_available_w = width * 0.88;
  let gap = 2.0;
  let max_bar_w = 4.0f32.max(18.0f32.min((total_available_w - cols as f32 * gap) / cols as f32));
  let total_w = cols as f32 * (max_bar_w + gap);
  let start_x = center_x - total_w / 2.0;

  let vals: Vec<Vec<f32>> = st
    .frame_history
    .iter()
    .map(|data| {
      (0..cols)
        .map(|i| {
          let mut sum = 0usize;
          for j in 0..step {
            sum += *data.get(i * step + j).unwrap_or(&0) as usize;
          }
          (sum as f32 / (step as f32 * 255.0)) * sensitivity
        })
        .collect()
    })
    .collect();

  let get_color = |ratio: f32, bright_val: f32| {
    let base = mix(p, s, ratio);
    let f = 0.5 + bright_val * 0.5;
    Color::rgba(
      (base.r * f).min(1.0),
      (base.g * f).min(1.0),
      (base.b * f).min(1.0),
      1.0,
    )
  };

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  for row in (0..rows).rev() {
    let depth_ratio = row as f32 / rows as f32;
    let z_offset = (rows - 1 - row) as f32 * 8.0;
    let row_y = floor_y - z_offset * 0.5;
    let scale = 1.0 - depth_ratio * 0.35;
    let row_alpha = 1.0 - depth_ratio * 0.45;

    for i in 0..cols {
      let val = vals[row][i];
      if val < 0.005 {
        continue;
      }
      let col = get_color(i as f32 / cols as f32, val);
      let max_h = height * 0.45 * scale;
      let bh = 2.0f32.max(val * max_h);
      let bw = 2.0f32.max(max_bar_w * scale);
      let x = start_x + i as f32 * (max_bar_w + gap) * scale + (1.0 - scale) * (total_w / 2.0);
      let by = row_y - bh;
      let dx = 1.0f32.max(bw * 0.4);
      let dy = 1.0f32.max(bw * 0.3);

      let front = col.with_alpha(row_alpha * 0.85);
      let top = Color::rgba(
        (col.r + 40.0 / 255.0).min(1.0),
        (col.g + 40.0 / 255.0).min(1.0),
        (col.b + 40.0 / 255.0).min(1.0),
        row_alpha,
      );
      let side = Color::rgba(col.r * 0.5, col.g * 0.5, col.b * 0.5, row_alpha * 0.75);
      let stroke_col = Color::rgba(
        (col.r + 60.0 / 255.0).min(1.0),
        (col.g + 60.0 / 255.0).min(1.0),
        (col.b + 60.0 / 255.0).min(1.0),
        row_alpha * 0.6,
      );

      c.set_fill(Fill::Solid(front));
      c.fill_rect(x, by, bw, bh);
      c.set_stroke(Fill::Solid(stroke_col));
      c.set_line_width(0.7);
      c.stroke_rect(x, by, bw, bh);

      c.set_fill(Fill::Solid(top));
      c.fill_polygon(&[
        (x, by),
        (x + dx, by - dy),
        (x + bw + dx, by - dy),
        (x + bw, by),
      ]);

      c.set_fill(Fill::Solid(side));
      c.fill_polygon(&[
        (x + bw, by),
        (x + bw + dx, by - dy),
        (x + bw + dx, row_y - dy),
        (x + bw, row_y),
      ]);

      if val > 0.45 && row == 0 {
        let beam = Fill::linear_gradient(0.0, by, 0.0, 0.0, &[
          (0.0, col.with_alpha(val * 0.35)),
          (1.0, col.with_alpha(0.0)),
        ]);
        c.set_fill(beam);
        c.fill_rect(x - 1.0, 0.0, bw + 2.0, by);
      }
    }
  }

  let h_span = (height - floor_y).max(1.0);
  for row in 0..rows {
    let depth_ratio = row as f32 / rows as f32;
    let z_offset = (rows - 1 - row) as f32 * 8.0;
    let row_y = floor_y + z_offset * 0.3;
    let scale = 1.0 - depth_ratio * 0.35;
    let ref_alpha = ((0.35 - depth_ratio * 0.2) * (1.0 - (row_y - floor_y) / h_span)).max(0.05);

    for i in 0..cols {
      let val = vals[row][i];
      if val < 0.01 {
        continue;
      }
      let col = get_color(i as f32 / cols as f32, val);
      let max_h = height * 0.38 * scale;
      let bh = 2.0f32.max(val * max_h * 0.8);
      let bw = 2.0f32.max(max_bar_w * scale);
      let x = start_x + i as f32 * (max_bar_w + gap) * scale + (1.0 - scale) * (total_w / 2.0);
      let ref_by = row_y;
      let dx = 1.0f32.max(bw * 0.4);
      let dy = 1.0f32.max(bw * 0.3);

      let front = col.with_alpha(ref_alpha * 0.6);
      let bottom = Color::rgba(col.r * 0.7, col.g * 0.7, col.b * 0.7, ref_alpha * 0.4);
      let side = Color::rgba(col.r * 0.3, col.g * 0.3, col.b * 0.3, ref_alpha * 0.4);

      c.set_fill(Fill::Solid(front));
      c.fill_rect(x, ref_by, bw, bh);

      c.set_fill(Fill::Solid(bottom));
      c.fill_polygon(&[
        (x, ref_by + bh),
        (x + dx, ref_by + bh + dy),
        (x + bw + dx, ref_by + bh + dy),
        (x + bw, ref_by + bh),
      ]);

      c.set_fill(Fill::Solid(side));
      c.fill_polygon(&[
        (x + bw, ref_by),
        (x + bw + dx, ref_by + dy),
        (x + bw + dx, ref_by + bh + dy),
        (x + bw, ref_by + bh),
      ]);
    }
  }

  c.restore();
}

// ---------------------------------------------------------------------------
// speaker3D
// ---------------------------------------------------------------------------

pub fn speaker_3d(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = super::theme_primary(theme);
  let s = super::theme_secondary(theme);
  let a = super::theme_accent(theme);
  let glow = super::theme_glow(theme);
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;

  let center_x = width / 2.0;
  let center_y = height / 2.0;
  let base_radius = width.min(height) * 0.27;
  let bass_pulse = 1.0 + be * 0.12 + bs * 0.08;
  let speaker_r = base_radius * bass_pulse;

  let bar_grad = Fill::linear_gradient(0.0, 0.0, 0.0, height, &[
    (0.0, p.with_alpha(0.85)),
    (0.3, s.with_alpha(0.95)),
    (0.6, a.with_alpha(0.98)),
    (0.85, s.with_alpha(0.95)),
    (1.0, p.with_alpha(0.85)),
  ]);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let half_bars = 48;
  let step = ((freq.len() as f32 * 0.7) as usize / half_bars).max(1);
  let left_start = width * 0.02;
  let left_end = (left_start + 20.0).max(center_x - speaker_r * 0.85);
  let left_width = left_end - left_start;
  let right_start = (width * 0.98 - 20.0).min(center_x + speaker_r * 0.85);
  let right_end = width * 0.98;
  let right_width = right_end - right_start;
  let bar_w = ((left_width / half_bars as f32) - 2.5).max(2.5);

  c.set_shadow(glow, 20.0 + be * 20.0);
  c.set_stroke(Fill::Solid(p));
  c.set_line_width(2.2);
  c.stroke_line(0.0, center_y, width, center_y);
  c.set_shadow(glow, 15.0);

  for i in 0..half_bars {
    let val = bin_sum(freq, step, i) * sensitivity;
    if val < 0.01 {
      continue;
    }
    let bar_h = val * height * 0.36;
    let top_y = center_y - bar_h;
    let bot_y = center_y + bar_h * 0.82;
    let f = i as f32 / (half_bars - 1) as f32;
    let x_left = left_end - f * left_width - bar_w;
    let x_right = right_start + f * right_width;
    c.set_fill(bar_grad.clone());
    c.fill_rect(x_left, top_y, bar_w, bar_h * 1.82);
    c.fill_rect(x_right, top_y, bar_w, bar_h * 1.82);
    c.set_fill(Fill::Solid(a));
    c.fill_rect(x_left - 0.5, top_y - 1.5, bar_w + 1.0, 1.5);
    c.fill_rect(x_left - 0.5, bot_y, bar_w + 1.0, 1.5);
    c.fill_rect(x_right - 0.5, top_y - 1.5, bar_w + 1.0, 1.5);
    c.fill_rect(x_right - 0.5, bot_y, bar_w + 1.0, 1.5);
  }

  let glow_r = speaker_r * 1.4;
  let back_glow = Fill::radial_gradient(
    center_x,
    center_y,
    speaker_r * 0.5,
    center_x,
    center_y,
    glow_r,
    &[
      (0.0, a.with_alpha(0.85)),
      (0.5, s.with_alpha(0.4)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_shadow(glow, 45.0 + be * 35.0);
  c.set_fill(back_glow);
  c.fill_circle(center_x, center_y, glow_r);

  let flare = |c: &mut GpuCanvas, fx: f32| {
    let fg = Fill::radial_gradient(
      fx,
      center_y,
      0.0,
      fx,
      center_y,
      speaker_r * 0.45,
      &[
        (0.0, Color::rgba(1.0, 1.0, 1.0, 0.95)),
        (0.2, p.with_alpha(0.85)),
        (0.6, s.with_alpha(0.3)),
        (1.0, Color::TRANSPARENT),
      ],
    );
    c.set_fill(fg);
    c.fill_circle(fx, center_y, speaker_r * 0.45);
  };
  flare(c, center_x - speaker_r * 0.96);
  flare(c, center_x + speaker_r * 0.96);

  let outer_rim = speaker_r;
  let inner_rim = speaker_r * 0.88;
  let metallic = Fill::linear_gradient(
    center_x - outer_rim,
    center_y - outer_rim,
    center_x + outer_rim,
    center_y + outer_rim,
    &[
      (0.0, Color::hex("#FFFFFF")),
      (0.15, Color::hex("#8E8E93")),
      (0.35, Color::hex("#2C2C2E")),
      (0.55, Color::hex("#D1D1D6")),
      (0.75, Color::hex("#48484A")),
      (1.0, Color::hex("#E5E5EA")),
    ],
  );
  c.set_shadow(Color::hex("#000000"), 18.0);
  c.set_fill(metallic);
  c.fill_ring(center_x, center_y, outer_rim, inner_rim);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.5)));
  c.set_line_width(1.5);
  c.stroke_circle(center_x, center_y, outer_rim - 1.0);
  c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, 0.6)));
  c.stroke_circle(center_x, center_y, inner_rim + 1.0);

  let bolt_radius = (outer_rim + inner_rim) / 2.0;
  for k in 0..4 {
    let angle = k as f32 * TAU / 4.0;
    let bx = center_x + angle.cos() * bolt_radius;
    let by = center_y + angle.sin() * bolt_radius;
    c.set_shadow(Color::hex("#000000"), 4.0);
    c.set_fill(Fill::Solid(Color::hex("#E5E5EA")));
    c.fill_circle(bx, by, 3.8);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(Color::hex("#1C1C1E")));
    c.set_line_width(1.2);
    c.stroke_line(bx - 2.0, by, bx + 2.0, by);
  }

  let surround_outer = inner_rim;
  let surround_inner = speaker_r * 0.74;
  let rubber = Fill::radial_gradient(
    center_x,
    center_y,
    surround_inner,
    center_x,
    center_y,
    surround_outer,
    &[
      (0.0, Color::hex("#1C1C1E")),
      (0.5, Color::hex("#3A3A3C")),
      (1.0, Color::hex("#0C0C0E")),
    ],
  );
  c.set_fill(rubber);
  c.fill_ring(center_x, center_y, surround_outer, surround_inner);

  let cone_outer = surround_inner;
  let cone_inner = speaker_r * 0.30;
  let cone = Fill::radial_gradient(
    center_x - cone_outer * 0.25,
    center_y - cone_outer * 0.25,
    cone_inner * 0.4,
    center_x,
    center_y,
    cone_outer,
    &[
      (0.0, Color::hex("#48484A")),
      (0.5, Color::hex("#2C2C2E")),
      (1.0, Color::hex("#1C1C1E")),
    ],
  );
  c.set_fill(cone);
  c.fill_circle(center_x, center_y, cone_outer);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.08)));
  let grid = 12.0f32;
  let dot_r = 1.8;
  let min_d2 = (cone_inner * 0.9) * (cone_inner * 0.9);
  let max_d2 = (cone_outer * 0.98) * (cone_outer * 0.98);
  let mut gy = center_y - cone_outer;
  let mut row = 0i32;
  while gy <= center_y + cone_outer {
    let row_offset = if row % 2 == 0 { 0.0 } else { grid * 0.5 };
    let mut gx = center_x - cone_outer;
    while gx <= center_x + cone_outer {
      let xp = gx + row_offset;
      let dxp = xp - center_x;
      let dyp = gy - center_y;
      let d2 = dxp * dxp + dyp * dyp;
      if d2 >= min_d2 && d2 <= max_d2 {
        c.fill_circle(xp, gy, dot_r);
      }
      gx += grid;
    }
    gy += grid;
    row += 1;
  }

  let dust_r = cone_inner * (1.0 + be * 0.06);
  let dust = Fill::radial_gradient(
    center_x - dust_r * 0.3,
    center_y - dust_r * 0.3,
    0.0,
    center_x,
    center_y,
    dust_r,
    &[
      (0.0, Color::hex("#636366")),
      (0.4, Color::hex("#3A3A3C")),
      (0.85, Color::hex("#1C1C1E")),
      (1.0, Color::hex("#0C0C0E")),
    ],
  );
  c.set_shadow(Color::hex("#000000"), 14.0 + be * 10.0);
  c.set_fill(dust);
  c.fill_circle(center_x, center_y, dust_r);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.35)));
  c.fill_ring_arc(
    center_x - dust_r * 0.15,
    center_y - dust_r * 0.15,
    dust_r * 0.65,
    dust_r * 0.45,
    TAU * 0.5,
    TAU * 0.925,
  );

  c.restore();
}

// ---------------------------------------------------------------------------
// Shared woofer (speakerTrio / speakerSplatter)
// ---------------------------------------------------------------------------

struct WooferStyle<'a> {
  rim_stops: &'a [(f32, Color)],
  bolt_r: f32,
  ring_alpha: f32,
  ring_step: f32,
  shadow_blur: f32,
}

fn draw_woofer(c: &mut GpuCanvas, x: f32, y: f32, r: f32, is_center: bool, style: &WooferStyle) {
  let outer_r = r;
  let inner_r = r * 0.86;
  let bolt_r = (outer_r + inner_r) / 2.0;

  let shadow = if is_center { style.shadow_blur } else { style.shadow_blur * 0.72 };
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), shadow);

  let metallic = Fill::linear_gradient(
    x - r,
    y - r,
    x + r,
    y + r,
    style.rim_stops,
  );
  c.set_fill(metallic);
  c.fill_ring(x, y, outer_r, inner_r);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::hex("#DDDDDD")));
  for k in 0..4 {
    let angle = k as f32 * TAU / 4.0;
    let bx = x + angle.cos() * bolt_r;
    let by = y + angle.sin() * bolt_r;
    c.fill_circle(bx, by, style.bolt_r);
  }

  let surround_inner = r * 0.72;
  let rubber = Fill::radial_gradient(x, y, surround_inner, x, y, inner_r, &[
    (0.0, Color::hex("#1A1A1E")),
    (0.5, Color::hex("#3A3A40")),
    (1.0, Color::hex("#101014")),
  ]);
  c.set_fill(rubber);
  c.fill_ring(x, y, inner_r, surround_inner);

  let cone_inner = r * 0.32;
  let cone = Fill::radial_gradient(x - r * 0.2, y - r * 0.2, cone_inner * 0.5, x, y, surround_inner, &[
    (0.0, Color::hex("#444855")),
    (0.6, Color::hex("#22242C")),
    (1.0, Color::hex("#111216")),
  ]);
  c.set_fill(cone);
  c.fill_circle(x, y, surround_inner);

  c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, style.ring_alpha)));
  c.set_line_width(1.2);
  let mut ring = cone_inner + 6.0;
  while ring < surround_inner - 4.0 {
    c.stroke_circle(x, y, ring);
    ring += style.ring_step;
  }

  let dust_r = cone_inner * 1.0;
  let dust = Fill::radial_gradient(x - dust_r * 0.3, y - dust_r * 0.3, 0.0, x, y, dust_r, &[
    (0.0, Color::hex("#666A78")),
    (0.4, Color::hex("#30333D")),
    (1.0, Color::hex("#0C0D10")),
  ]);
  c.set_shadow(Color::hex("#000000"), 10.0);
  c.set_fill(dust);
  c.fill_circle(x, y, dust_r);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.35)));
  c.fill_ring_arc(
    x - dust_r * 0.15,
    y - dust_r * 0.15,
    dust_r * 0.65,
    dust_r * 0.45,
    TAU * 0.5,
    TAU * 0.925,
  );
}

// ---------------------------------------------------------------------------
// speakerTrio
// ---------------------------------------------------------------------------

pub fn speaker_trio(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = super::theme_primary(theme);
  let s = super::theme_secondary(theme);
  let a = super::theme_accent(theme);
  let glow = super::theme_glow(theme);
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  const SYMBOLS: [char; 6] = ['\u{266A}', '\u{266B}', '\u{266C}', '\u{2669}', '\u{222E}', '\u{1F3BC}'];
  if st.notes.is_empty() {
    for _ in 0..18 {
      st.notes.push(FloatingNote {
        x: rng.next() * width,
        y: rng.next() * height,
        vx: (rng.next() - 0.5) * 0.8,
        vy: -0.5 - rng.next() * 1.2,
        symbol: SYMBOLS[(rng.next() * SYMBOLS.len() as f32) as usize],
        size: 14.0 + rng.next() * 18.0,
        alpha: 0.3 + rng.next() * 0.5,
        rotation: (rng.next() - 0.5) * 0.5,
        rot_speed: (rng.next() - 0.5) * 0.02,
      });
    }
  }

  let center_x = width / 2.0;
  let center_y = height / 2.0;
  let base_r = width.min(height) * 0.14;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let half_count = 40;
  let step = ((ctx.freq_data.len() as f32 * 0.5) as usize / half_count).max(1);
  let start_x = width * 0.04;
  let half_w = center_x - start_x - 4.0;
  let bar_w = ((half_w / half_count as f32) - 1.5).max(2.0);
  let max_bar_h = height * 0.32;

  c.set_shadow(glow, 18.0 + be * 15.0);
  c.set_stroke(Fill::Solid(p));
  c.set_line_width(1.5);
  c.stroke_line(start_x, center_y, width - start_x, center_y);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  for i in 0..half_count {
    let val = bin_sum(ctx.freq_data, step, i) * sensitivity;
    if val < 0.02 {
      continue;
    }
    let bar_h = 1.0f32.max(val * max_bar_h);
    let x_left = start_x + i as f32 * (bar_w + 1.5);
    let x_right = width - start_x - (i as f32 + 1.0) * (bar_w + 1.5);
    let y_top = center_y - bar_h;
    let bright_f = 0.5 + val * 0.5;
    let col = bright(mix(p, s, i as f32 / half_count as f32), bright_f);
    c.set_fill(Fill::Solid(col));
    c.set_shadow(glow, 8.0 + val * 12.0);
    c.fill_rect(x_left, y_top, bar_w, bar_h);
    c.fill_rect(x_right, y_top, bar_w, bar_h);
    if val > 0.15 {
      c.set_fill(Fill::Solid(a.with_alpha(val * 0.3)));
      c.set_shadow(Color::TRANSPARENT, 0.0);
      c.fill_rect(x_left, y_top, bar_w, 1.5);
      c.fill_rect(x_right, y_top, bar_w, 1.5);
    }
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);

  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.4), 4.0);
  let mut drawn = 0usize;
  for i in 0..st.notes.len() {
    if drawn >= 10 {
      break;
    }
    let (x, y, rotation, symbol, size, alpha) = {
      let n = &mut st.notes[i];
      n.y += n.vy - be * 1.5;
      n.x += n.vx + (n.y * 0.02).sin() * 0.5;
      n.rotation += n.rot_speed;
      if n.y < -30.0 {
        n.y = height + 20.0;
        n.x = rng.next() * width;
      }
      (n.x, n.y, n.rotation, n.symbol, n.size, n.alpha)
    };
    c.save();
    c.translate(x, y);
    c.rotate(rotation);
    c.draw_text(
      &symbol.to_string(),
      0.0,
      0.0,
      size,
      "sans-serif",
      400.0,
      TextAlign::Center,
      Fill::Solid(a.with_alpha(alpha)),
      1.0,
      &TextOpts::default(),
    );
    c.restore();
    drawn += 1;
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let left_x = center_x - base_r * 1.25;
  let right_x = center_x + base_r * 1.25;
  let left_r = base_r * 0.82 * (1.0 + be * 0.08);
  let right_r = base_r * 0.82 * (1.0 + be * 0.08);
  let center_r = base_r * 1.12 * (1.0 + be * 0.14 + ctx.beat_strength * 0.08);

  let trio_style = WooferStyle {
    rim_stops: &[
      (0.0, Color::hex("#FFFFFF")),
      (0.2, Color::hex("#999999")),
      (0.5, Color::hex("#222222")),
      (0.8, Color::hex("#CCCCCC")),
      (1.0, Color::hex("#444444")),
    ],
    bolt_r: 3.0,
    ring_alpha: 0.08,
    ring_step: 10.0,
    shadow_blur: 18.0,
  };

  draw_woofer(c, left_x, center_y, left_r, false, &trio_style);
  draw_woofer(c, right_x, center_y, right_r, false, &trio_style);
  draw_woofer(c, center_x, center_y, center_r, true, &trio_style);

  c.restore();
}

// ---------------------------------------------------------------------------
// speakerSplatter
// ---------------------------------------------------------------------------

pub fn speaker_splatter(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = super::theme_primary(theme);
  let s = super::theme_secondary(theme);
  let a = super::theme_accent(theme);
  let g = super::theme_glow(theme);
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  let center_x = width / 2.0;
  let center_y = height / 2.0;
  let max_dim = width.min(height);
  let base_r = max_dim * 0.13;

  if st.splatter.is_empty() {
    for _ in 0..45 {
      let angle = rng.next() * TAU;
      let dist = base_r * (0.4 + rng.next() * 1.3);
      st.splatter.push(SplatterDot {
        x: center_x + angle.cos() * dist,
        y: center_y + angle.sin() * dist + base_r * 0.2,
        r: 1.2 + rng.next() * 4.5,
      });
    }
  }

  let freq_avg: f32 = ctx.freq_data.iter().map(|&b| b as f32).sum::<f32>()
    / (ctx.freq_data.len() as f32 * 255.0)
    * sensitivity;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let pulse = 1.0 + be * 0.18 + bs * 0.12;
  let arc_alpha = 0.25 + be * 0.25 + freq_avg * 0.15;
  let wide = 1.0 + be * 0.15;

  st.arc_rotation += 0.02;
  let rot = st.arc_rotation;

  let soft_stroke = |c: &mut GpuCanvas,
                         cx: f32,
                         cy: f32,
                         radius: f32,
                         start: f32,
                         end: f32,
                         color: Color,
                         alpha: f32| {
    let layers: [(f32, f32); 3] = [(10.0, 0.08), (6.0, 0.15), (2.0, 0.4)];
    for (w, la) in layers {
      c.set_line_width(w);
      c.set_stroke(Fill::Solid(color.with_alpha(alpha * la)));
      c.set_line_cap(LineCap::Round);
      c.stroke_arc(cx, cy, radius, start, end);
    }
  };

  c.set_shadow(Color::TRANSPARENT, 0.0);
  for k in 1..=4 {
    let radius = base_r * (1.0 + k as f32 * 0.35) * pulse;
    let spread = (TAU * 0.30) * wide;
    let base_angle = TAU * 0.59;
    let fade = (1.0 - (k as f32 - 1.0) * 0.28).max(0.08);
    soft_stroke(
      c,
      center_x - 6.0,
      center_y - 4.0,
      radius,
      base_angle - spread * 0.5 + rot,
      base_angle + spread * 0.5 + rot,
      p,
      arc_alpha * fade,
    );
  }

  for k in 1..=4 {
    let radius = base_r * (1.0 + k as f32 * 0.35) * pulse;
    let spread = (TAU * 0.28) * wide;
    let base_angle = TAU * 0.15;
    let fade = (1.0 - (k as f32 - 1.0) * 0.28).max(0.08);
    soft_stroke(
      c,
      center_x + 6.0,
      center_y + 4.0,
      radius,
      base_angle - spread * 0.5 - rot,
      base_angle + spread * 0.5 - rot,
      s,
      arc_alpha * fade,
    );
  }

  for k in 1..=3 {
    let radius = base_r * (1.1 + k as f32 * 0.38) * pulse;
    let spread = (TAU * 0.23) * wide;
    let base_angle = -TAU * 0.125;
    let fade = (1.0 - (k as f32 - 1.0) * 0.4).max(0.08);
    soft_stroke(
      c,
      center_x + 2.0,
      center_y - 2.0,
      radius,
      base_angle - spread * 0.5 + rot,
      base_angle + spread * 0.5 + rot,
      a,
      arc_alpha * fade * 0.7,
    );
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let glow_intensity = 0.3 + be * 0.25 + bs * 0.15;
  let glow_radius = base_r * 2.8 * pulse;
  for pos in [
    (center_x, center_y),
    (center_x - base_r * 0.92, center_y + base_r * 0.06),
    (center_x + base_r * 0.92, center_y + base_r * 0.06),
  ] {
    let grad = Fill::radial_gradient(pos.0, pos.1, 0.0, pos.0, pos.1, glow_radius, &[
      (0.0, g.with_alpha(glow_intensity * 0.5)),
      (0.15, p.with_alpha(glow_intensity * 0.2)),
      (0.4, s.with_alpha(glow_intensity * 0.08)),
      (1.0, Color::TRANSPARENT),
    ]);
    c.set_fill(grad);
    c.fill_circle(pos.0, pos.1, glow_radius);
  }

  let ink_y = center_y + base_r * 0.25;
  c.set_fill(Fill::Solid(Color::hex("#14141A")));
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 10.0);
  c.fill_ellipse(center_x, ink_y, base_r * 1.4, base_r * 0.75);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let splatter_colors = [p, s, a];
  for (i, dot) in st.splatter.iter().enumerate() {
    let sx = center_x + (dot.x - center_x) * (1.0 + be * 0.15);
    let sy = center_y + (dot.y - center_y) * (1.0 + be * 0.15);
    c.set_fill(Fill::Solid(splatter_colors[i % 3]));
    c.fill_circle(sx, sy, dot.r * (1.0 + bs * 0.3));
  }

  c.set_stroke(Fill::Solid(a));
  c.set_line_cap(LineCap::Round);
  let drip_xs = [-0.85, -0.55, -0.15, 0.15, 0.5, 0.8];
  let drip_lens = [30.0, 55.0, 75.0, 45.0, 65.0, 25.0];
  for d in 0..drip_xs.len() {
    let dx = center_x + drip_xs[d] * base_r;
    let dy = ink_y + base_r * 0.2;
    let len = drip_lens[d] * (1.0 + be * 0.3);
    let thick = 2.5 + (d % 3) as f32;
    c.set_line_width(thick);
    c.set_shadow(g, 6.0);
    c.stroke_line(dx, dy, dx, dy + len);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_fill(Fill::Solid(a));
    c.fill_circle(dx, dy + len + thick * 0.5, thick * 1.2);
  }

  let center_r = base_r * 1.15 * (1.0 + be * 0.10);
  let left_r = base_r * 0.88 * (1.0 + be * 0.07);
  let right_r = base_r * 0.88 * (1.0 + be * 0.07);
  let left_x = center_x - base_r * 0.92;
  let left_y = center_y + base_r * 0.06;
  let right_x = center_x + base_r * 0.92;
  let right_y = center_y + base_r * 0.06;

  let splatter_style = WooferStyle {
    rim_stops: &[
      (0.0, Color::hex("#FFFFFF")),
      (0.2, Color::hex("#AAAAAA")),
      (0.45, Color::hex("#222226")),
      (0.75, Color::hex("#DDDDDD")),
      (1.0, Color::hex("#55555A")),
    ],
    bolt_r: 2.5,
    ring_alpha: 0.15,
    ring_step: 8.0,
    shadow_blur: 18.0,
  };

  draw_woofer(c, left_x, left_y, left_r, false, &splatter_style);
  draw_woofer(c, right_x, right_y, right_r, false, &splatter_style);
  draw_woofer(c, center_x, center_y, center_r, true, &splatter_style);

  c.restore();
}
