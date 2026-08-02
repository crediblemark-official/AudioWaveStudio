//! Ports of stateful / noise-based renderers (vuMeter, pulseRings, auroraWave).

use std::f32::consts::PI;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, PulseRing, RenderContext,
};

// ---------------------------------------------------------------------------
// vuMeter.ts
// ---------------------------------------------------------------------------

pub fn vu_meter(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let sensitivity = ctx.config.reactivity.sensitivity;

  let cx = ctx.width / 2.0;
  let cy = ctx.height / 2.0;
  let radius = ctx.width.min(ctx.height) * 0.38;

  for ch in 0..2 {
    let start_bin = ch * 6;
    let mut sum = 0usize;
    let mut n = 0;
    for i in 0..6 {
      let k = start_bin + i;
      if k < ctx.freq_data.len() {
        sum += ctx.freq_data[k] as usize;
        n += 1;
      }
    }
    let raw = if n > 0 { (sum as f32 / (n as f32 * 255.0)) * sensitivity } else { 0.0 };

    let ch_state = &mut ctx.state.vu[ch];
    ch_state.level += (raw.min(1.0) - ch_state.level) * 0.3;
    ch_state.peak = ch_state.peak.max(ch_state.level);
    ch_state.peak *= 0.92;
    ch_state.peak_hold = ch_state.peak_hold.max(ch_state.peak);
    ch_state.peak_hold -= 0.003;
    if ch_state.peak_hold < 0.0 {
      ch_state.peak_hold = 0.0;
    }
  }

  let spacing = radius * 0.6;
  let gap = radius * 0.15;

  for ch in 0..2 {
    let x = if ch == 0 { cx - spacing - gap } else { cx + spacing + gap };
    let y = cy;
    let ch_state = &ctx.state.vu[ch];
    let level = ch_state.level;
    let peak_hold = ch_state.peak_hold;

    let green_angle = (-0.75 + level * 2.5).max(0.0);

    c.save();
    c.translate(x, y);

    // Background arc.
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.1)));
    c.set_line_width(6.0);
    c.stroke_arc(0.0, 0.0, radius, PI * 0.8, PI * 0.2);

    // Active arc.
    let active_color = if level > 0.7 {
      if ch == 0 { theme_accent(theme) } else { Color::hex("#ff3333") }
    } else if ch == 0 {
      theme_primary(theme)
    } else {
      theme_secondary(theme)
    };
    c.save();
    c.set_stroke(Fill::Solid(active_color));
    c.set_shadow(theme_glow(theme), 12.0);
    c.stroke_arc(0.0, 0.0, radius, PI * 0.8, PI * 0.8 + green_angle);
    c.restore();

    // Needle.
    let needle_angle = PI * 0.8 + level * 2.5;
    c.save();
    c.set_stroke(Fill::Solid(theme_accent(theme)));
    c.set_line_width(3.0);
    c.set_shadow(theme_glow(theme), 8.0);
    c.stroke_line(0.0, 0.0, needle_angle.cos() * radius * 0.7, needle_angle.sin() * radius * 0.7);
    c.restore();

    // Center cap.
    c.set_fill(Fill::Solid(Color::WHITE));
    c.fill_circle(0.0, 0.0, 6.0);

    // Peak hold dot.
    let hold_angle = PI * 0.8 + peak_hold * 2.5;
    c.set_fill(Fill::Solid(Color::WHITE));
    c.fill_circle(hold_angle.cos() * radius, hold_angle.sin() * radius, 4.0);

    c.restore();
  }

  // "VU METER" label (baseline at `height - 15`, monospace, centered).
  let label_size = (ctx.width * 0.025).min(16.0);
  c.draw_text(
    "VU METER",
    cx,
    ctx.height - 15.0,
    label_size,
    "monospace",
    400.0,
    crate::gpu2d::text::TextAlign::Center,
    Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.4)),
    1.0,
    &Default::default(),
  );
}

// ---------------------------------------------------------------------------
// pulseRings.ts
// ---------------------------------------------------------------------------

pub fn pulse_rings(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let center_x = ctx.width / 2.0;
  let center_y = ctx.height / 2.0;
  let max_dim = ctx.width.max(ctx.height) * 0.8;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;

  let state = &mut ctx.state;
  if bs > 0.15 && bs > state.prev_beat {
    let count = 1 + (bs * 2.0).floor() as u32;
    for i in 0..count {
      let color = if i % 2 == 0 { theme_primary(theme) } else { theme_secondary(theme) };
      state.rings.push(PulseRing {
        radius: 10.0 + i as f32 * 20.0,
        max_radius: max_dim * (0.5 + state.rng.next() * 0.5),
        alpha: 0.4 + be * 0.3,
        speed: 2.0 + bs * 3.0 + state.rng.next() * 2.0,
        thickness: 2.0 + be * 4.0 + bs * 3.0,
        color,
      });
    }
  }
  state.prev_beat = bs;

  for i in (0..state.rings.len()).rev() {
    let r = &mut state.rings[i];
    r.radius += r.speed;
    r.alpha *= 0.985;

    if r.radius > r.max_radius || r.alpha < 0.01 {
      state.rings.remove(i);
      continue;
    }

    let denom = 0.4 + be * 0.3;
    let lw = if denom > 0.0 { r.thickness * (r.alpha / denom) } else { r.thickness };
    c.save();
    c.set_global_alpha(r.alpha);
    c.set_stroke(Fill::Solid(r.color));
    c.set_line_width(lw);
    c.set_shadow(theme_glow(theme), 15.0);
    c.stroke_circle(center_x, center_y, r.radius);
    c.restore();
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
}

// ---------------------------------------------------------------------------
// auroraWave.ts
// ---------------------------------------------------------------------------

struct AuroraLayer {
  offset: f32,
  speed: f32,
  freq: f32,
  amp: f32,
  color: Color,
  alpha: f32,
  y_base: f32,
}

pub fn aurora_wave(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let be = ctx.bass_energy;

  let t = ctx.state.aurora_t;
  let bass_amp = 0.1 + be * 0.4;

  let primary = theme_primary(theme);
  let secondary = theme_secondary(theme);
  let accent = Color::hex(&theme.accent_color);

  let layers = [
    AuroraLayer { offset: 0.0, speed: 0.5, freq: 0.003, amp: 0.06, color: primary, alpha: 0.25, y_base: ctx.height * 0.3 },
    AuroraLayer { offset: 2.0, speed: 0.7, freq: 0.005, amp: 0.04, color: secondary, alpha: 0.2, y_base: ctx.height * 0.45 },
    AuroraLayer { offset: 4.0, speed: 0.3, freq: 0.002, amp: 0.08, color: accent, alpha: 0.15, y_base: ctx.height * 0.35 },
  ];

  // Gradient fills (transparent edges).
  for layer in &layers {
    let mut points: Vec<(f32, f32)> = Vec::new();
    let mut x = 0.0f32;
    while x <= ctx.width {
      let freq_idx = ((x / ctx.width) * ctx.freq_data.len() as f32).floor() as usize;
      let f_val = *ctx.freq_data.get(freq_idx).unwrap_or(&0) as f32;
      let wave = (x * layer.freq + t * layer.speed + layer.offset).sin();
      let wave2 = (x * layer.freq * 2.3 + t * layer.speed * 1.7 + layer.offset + 1.5).sin();
      let amp = layer.amp * (1.0 + bass_amp * 2.0) * (1.0 + (f_val / 255.0) * 0.5);
      let y = layer.y_base + wave * amp * ctx.height + wave2 * amp * ctx.height * 0.5;
      points.push((x, y));
      x += 4.0;
    }

    let mut poly: Vec<(f32, f32)> = Vec::with_capacity(points.len() + 2);
    poly.push((0.0, ctx.height));
    poly.extend_from_slice(&points);
    poly.push((ctx.width, ctx.height));

    let alpha_hex = (layer.alpha * 255.0).round() as u32;
    let grad = Fill::linear_gradient(0.0, 0.0, ctx.width, 0.0, &[
      (0.0, with_alpha(layer.color, 0.0)),
      (0.3, with_alpha(layer.color, alpha_hex as f32 / 255.0)),
      (0.7, with_alpha(layer.color, alpha_hex as f32 / 255.0)),
      (1.0, with_alpha(layer.color, 0.0)),
    ]);
    c.save();
    c.set_global_alpha(layer.alpha * (0.5 + be * 0.5));
    c.set_fill(grad);
    c.fill_polygon(&poly);
    c.restore();
  }

  // Strokes.
  for layer in &layers {
    let mut pts: Vec<(f32, f32)> = Vec::new();
    let mut x = 0.0f32;
    while x <= ctx.width {
      let freq_idx = ((x / ctx.width) * ctx.freq_data.len() as f32).floor() as usize;
      let f_val = *ctx.freq_data.get(freq_idx).unwrap_or(&0) as f32;
      let wave = (x * layer.freq + t * layer.speed + layer.offset).sin();
      let wave2 = (x * layer.freq * 2.3 + t * layer.speed * 1.7 + layer.offset + 1.5).sin();
      let amp = layer.amp * (1.0 + bass_amp * 2.0) * (1.0 + (f_val / 255.0) * 0.5);
      let y = layer.y_base + wave * amp * ctx.height + wave2 * amp * ctx.height * 0.5;
      pts.push((x, y));
      x += 4.0;
    }

    c.save();
    c.set_stroke(Fill::Solid(layer.color));
    c.set_line_width(2.0);
    c.set_global_alpha(layer.alpha * (0.8 + be * 0.2));
    c.set_shadow(layer.color, 20.0);
    c.stroke_polyline(&pts);
    c.restore();
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let next_t = t + 0.008;
  ctx.state.aurora_t = if next_t > 10000.0 { next_t - 10000.0 } else { next_t };
}

fn with_alpha(color: Color, a: f32) -> Color {
  Color { r: color.r, g: color.g, b: color.b, a }
}
