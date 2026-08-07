//! 3D Blocks style renderer (`threeD`) — faithful port of
//! `src/services/renderers/threeD.ts` (export path parity).
//!
//! The previous port omitted several passes the preview shows: beat sparks
//! with fading trails, decaying peak dots, the top-edge stroke line through
//! the bar tips, tip glow dots, the bright-boost glow cap on each bar, the
//! accent cap line on strong bass, the glossy floor-reflection band, and the
//! alpha-blended top/side faces. This port renders all of them in the same
//! order as the TS renderer.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{bin_sum, bright, mix, LightMote, Peak, Spark};
use crate::renderers::RenderContext;

struct Bar {
  x: f32,
  by: f32,
  bh: f32,
  bw: f32,
  dy: f32,
  val: f32,
}

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let a = crate::renderers::theme_accent(theme);
  let g = crate::renderers::theme_glow(theme);
  // TS: `Math.min(48, config.reactivity.barCount)` — no forced minimum.
  let bar_count = ctx.config.reactivity.bar_count.min(48);
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
  let step = (ctx.freq_data.len() / bar_count.max(1)).max(1);

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

  // Beat sparks (TS: `bs > 0.1 && bs > prevBeat`).
  if bs > 0.1 && bs > st.three_d_prev_beat {
    let bar_hs: Vec<f32> = (0..bar_count).map(|i| bin_sum(ctx.freq_data, step, i) * sensitivity).collect();
    let spark_count = (8.0f32 + bs * 25.0).floor() as usize;
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

  // TS updates mote positions with the PRE-increment rotationAngle
  // (threeD.ts:106-108), then increments it (threeD.ts:140) for the bars,
  // tips and mote alpha. Using the post-increment value here phase-shifts the
  // motes by one frame's rotation.
  let rot_before = st.three_d_rot;

  for m in st.motes.iter_mut() {
    m.x += m.vx + (rot_before + m.phase).sin() * 0.15;
    m.y += m.vy + (rot_before * 0.5 + m.phase).cos() * 0.05;
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

  st.three_d_rot += 0.002 + be * 0.004;
  let rot = st.three_d_rot;

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

  let mut bars: Vec<Bar> = Vec::with_capacity(bar_count);
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
    // TS: `pulse = 1 + be*0.2 + sin(rot + i*0.3) * be*0.05`
    let pulse = 1.0 + be * 0.2 + (rot + i as f32 * 0.3).sin() * be * 0.05;

    bars.push(Bar { x, by, bh, bw, dy, val });

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

    // Top face (TS: `globalAlpha = 0.6 + be * 0.15`).
    c.set_global_alpha((0.6 + be * 0.15).clamp(0.0, 1.0));
    c.set_fill(Fill::Solid(top_c));
    c.fill_polygon(&[
      (x, by),
      (x + dx, by - dy * pulse),
      (x + bw + dx, by - dy * pulse),
      (x + bw, by),
    ]);

    // Side face (TS: `globalAlpha = 0.4 + be * 0.15`).
    c.set_global_alpha((0.4 + be * 0.15).clamp(0.0, 1.0));
    c.set_fill(Fill::Solid(side_c));
    c.fill_polygon(&[
      (x + bw, by),
      (x + bw + dx, by - dy * pulse),
      (x + bw + dx, floor_y - dy * pulse),
      (x + bw, floor_y),
    ]);
    c.set_global_alpha(1.0);

    // Bright-boost glow cap (TS: shadowBlur 12+be*18 / 6+be*8 on the bar).
    c.set_shadow(g, 12.0 + be * 18.0);
    c.set_fill(Fill::Solid(bar_c));
    c.set_global_alpha((bright_boost * 0.4).clamp(0.0, 1.0));
    c.fill_rect(x, by, bw, bh.min(5.0));
    c.set_shadow(g, 6.0 + be * 8.0);
    c.fill_rect(x, by, bw, bh);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_global_alpha(1.0);

    // Accent cap line on strong bass (TS: `be > 0.25`).
    if be > 0.25 {
      c.set_fill(Fill::Solid(a.with_alpha(((be - 0.25) * 0.35).clamp(0.0, 1.0))));
      c.fill_rect(x, by, bw, bh.min(2.0));
    }
  }

  // Peaks: prune below alpha, add new on strong bars, decay.
  st.peaks.retain(|p| p.alpha > 0.01);
  for b in &bars {
    if b.val * 0.35 > 0.8 {
      let tip_x = b.x + b.bw / 2.0;
      if !st.peaks.iter().any(|p| (p.x - tip_x).abs() < 5.0) {
        st.peaks.push(Peak { x: tip_x, y: b.by, alpha: 1.0 });
      }
    }
  }
  for p in st.peaks.iter_mut() {
    p.alpha *= 0.94;
    p.y -= 0.3;
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);

  // Top-edge stroke line through the bar tips.
  let tip_y = |b: &Bar, i: usize| b.by - b.dy * (1.0 + be * 0.2 + (rot + i as f32 * 0.3).sin() * be * 0.05);
  let tips: Vec<(f32, f32)> = bars
    .iter()
    .enumerate()
    .map(|(i, b)| (b.x + b.bw / 2.0, tip_y(b, i)))
    .collect();
  c.set_stroke(Fill::Solid(p));
  c.set_global_alpha((0.2 + be * 0.2).clamp(0.0, 1.0));
  c.set_line_width(2.0);
  c.set_shadow(g, 10.0);
  c.stroke_polyline(&tips);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);

  // Tip glow dots.
  for (i, b) in bars.iter().enumerate() {
    if b.val < 0.05 {
      continue;
    }
    c.set_fill(Fill::Solid(g));
    c.set_global_alpha((b.val * 0.15 + be * 0.1).clamp(0.0, 1.0));
    c.set_shadow(g, 20.0);
    c.fill_circle(b.x + b.bw / 2.0, tip_y(b, i), 2.0 + b.val * 3.0);
    c.set_shadow(Color::TRANSPARENT, 0.0);
  }
  c.set_global_alpha(1.0);

  // Decaying peak dots.
  for p in &st.peaks {
    c.set_fill(Fill::Solid(a));
    c.set_global_alpha((p.alpha * 0.6).clamp(0.0, 1.0));
    c.set_shadow(a, 8.0);
    c.fill_circle(p.x, p.y, 2.5);
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);

  // Beat sparks with fading trails.
  for sp in &st.sparks {
    let progress = sp.life / sp.max_life;
    for (t, &(tx, ty)) in sp.trail.iter().enumerate() {
      let tp = t as f32 / sp.trail.len().max(1) as f32;
      c.set_fill(Fill::Solid(sp.color));
      c.set_global_alpha(((1.0 - progress) * tp * 0.4).clamp(0.0, 1.0));
      c.set_shadow(Color::TRANSPARENT, 0.0);
      c.fill_circle(tx, ty, sp.size * tp * 0.5);
    }
    c.set_fill(Fill::Solid(sp.color));
    c.set_global_alpha((1.0 - progress).clamp(0.0, 1.0));
    c.set_shadow(sp.color, 10.0);
    c.fill_circle(sp.x, sp.y, sp.size * (1.0 - progress * 0.4));
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(0.5);

  // Glossy floor reflection band (TS: refH = bh*0.25, alpha falls away from
  // the center column).
  for (i, b) in bars.iter().enumerate() {
    if b.val < 0.03 {
      continue;
    }
    let ref_alpha = (0.08 - (i as f32 - bar_count as f32 / 2.0).abs() * 0.003).max(0.0);
    if ref_alpha <= 0.0 {
      continue;
    }
    let col = mix(p, s, i as f32 / bar_count as f32);
    c.set_fill(Fill::Solid(col));
    c.set_global_alpha(ref_alpha.clamp(0.0, 1.0));
    c.fill_rect(b.x, floor_y + 4.0, b.bw, b.bh * 0.25);
  }
  c.set_global_alpha(1.0);

  // Light motes (TS: primary-colored, alpha `m.alpha * (0.4 + sin(rot*2 +
  // phase)*0.25)`, size `m.size * (0.8 + be*0.3)`).
  for m in &st.motes {
    c.set_global_alpha((m.alpha * (0.4 + (rot * 2.0 + m.phase).sin() * 0.25)).clamp(0.0, 1.0));
    c.set_fill(Fill::Solid(p));
    c.set_shadow(g, 4.0);
    c.fill_circle(m.x, m.y, m.size * (0.8 + be * 0.3));
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);
  c.restore();
}
