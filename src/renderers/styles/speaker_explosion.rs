//! Speaker Explosion style renderer (`speakerExplosion`).
//!
//! Renders an explosive audio subwoofer speaker complete with pumping bass cone,
//! golden dust cap, 360-degree needle-sharp radial spectrum ray burst, paint drips,
//! and floating paint splatter droplets.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const SPIKE_COUNT: usize = 120;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let t = ctx.frame_time;

  let center_x = width * 0.5;
  let center_y = height * 0.48;

  let base_r = (width.min(height) * 0.22).clamp(80.0, 260.0);
  let woofer_r = base_r + (be * 30.0) + (bs * 15.0);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.95);
  let electric_cyan = Color::rgba(0.0, 0.85, 1.0, 0.95);
  let purple_magenta = Color::rgba(0.7, 0.1, 0.95, 0.95);

  // -------------------------------------------------------------------------
  // 1. HIGH-DENSITY RADIAL SPIKY SPECTRUM RAY BURST (BEHIND SPEAKER)
  // -------------------------------------------------------------------------
  let max_spike_len = height * 0.22 * sensitivity;
  let step = (freq.len() / (SPIKE_COUNT / 2)).max(1);

  for i in 0..SPIKE_COUNT {
    let angle = (i as f32 / SPIKE_COUNT as f32) * TAU;

    let bin_i = if i <= SPIKE_COUNT / 2 {
      (i * step).min(freq.len().saturating_sub(1))
    } else {
      ((SPIKE_COUNT - i) * step).min(freq.len().saturating_sub(1))
    };

    let raw_v = *freq.get(bin_i).unwrap_or(&0) as f32 / 255.0;
    let spike_len = (raw_v * sensitivity * max_spike_len).clamp(10.0, max_spike_len * 1.4);

    let x1 = center_x + angle.cos() * (woofer_r * 0.9);
    let y1 = center_y + angle.sin() * (woofer_r * 0.9);
    let x2 = center_x + angle.cos() * (woofer_r * 0.9 + spike_len);
    let y2 = center_y + angle.sin() * (woofer_r * 0.9 + spike_len);

    let ray_col = if i % 3 == 0 {
      purple_magenta
    } else if i % 3 == 1 {
      hot_pink
    } else {
      p
    };

    c.set_stroke(Fill::Solid(ray_col.with_alpha(0.85)));
    c.set_line_width(2.0 + (raw_v * 3.0));
    c.set_shadow(ray_col.with_alpha(0.6), 8.0 + bs * 6.0);
    c.stroke_line(x1, y1, x2, y2);
  }

  // -------------------------------------------------------------------------
  // 2. FLOATING PAINT SPLATTER BLOBS & SPARK TENDRILS
  // -------------------------------------------------------------------------
  let num_splatters = 36usize;
  for i in 0..num_splatters {
    let seed = i as f32 * 41.7;
    let dist = woofer_r * 1.1 + (seed % (height * 0.25)) + (be * 40.0);
    let angle = (seed * 0.3 + t * 0.1) % TAU;

    let px = center_x + angle.cos() * dist;
    let py = center_y + angle.sin() * dist;

    let dot_r = (3.0 + (seed % 5.0) + bs * 4.0).clamp(2.0, 12.0);
    let dot_col = if i % 4 == 0 {
      Color::WHITE
    } else if i % 4 == 1 {
      electric_cyan
    } else if i % 4 == 2 {
      hot_pink
    } else {
      purple_magenta
    };

    c.set_fill(Fill::Solid(dot_col.with_alpha(0.9)));
    c.set_shadow(dot_col, 10.0);
    c.fill_ellipse(px, py, dot_r, dot_r);
  }

  // -------------------------------------------------------------------------
  // 3. PAINT DRIPPING EFFECT (HANGING DOWN FROM BOTTOM OF SPEAKER)
  // -------------------------------------------------------------------------
  let drip_count = 9usize;
  let drip_width = (woofer_r * 1.8) / drip_count as f32;
  let drip_start_x = center_x - woofer_r * 0.9;
  let drip_base_y = center_y + woofer_r * 0.6;

  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 10.0);

  for d in 0..drip_count {
    let seed = d as f32 * 73.1;
    let dx = drip_start_x + d as f32 * drip_width + drip_width * 0.5;
    let drip_len = 30.0 + (seed % 60.0) + (be * 35.0);

    let drip_col = if d % 2 == 0 {
      Color::rgba(0.95, 0.95, 1.0, 0.95) // White paint drip
    } else {
      purple_magenta.with_alpha(0.9) // Purple paint drip
    };

    // Main drip stalk line
    c.set_stroke(Fill::Solid(drip_col));
    c.set_line_width((drip_width * 0.5).clamp(4.0, 12.0));
    c.stroke_line(dx, drip_base_y, dx, drip_base_y + drip_len);

    // Drip bulb droplet tip
    c.set_fill(Fill::Solid(drip_col));
    c.fill_ellipse(dx, drip_base_y + drip_len, drip_width * 0.4, drip_width * 0.4);
  }

  // -------------------------------------------------------------------------
  // 4. SUBWOOFER SPEAKER CONE & METALLIC RIM
  // -------------------------------------------------------------------------
  // Outer Cyan/Chrome Metallic Frame Ring
  c.set_fill(Fill::Solid(Color::rgba(0.12, 0.14, 0.18, 0.98)));
  c.set_stroke(Fill::Solid(electric_cyan));
  c.set_line_width(4.0);
  c.set_shadow(electric_cyan.with_alpha(0.85), 20.0);
  c.fill_ellipse(center_x, center_y, woofer_r, woofer_r);
  c.stroke_circle(center_x, center_y, woofer_r);

  // Inner Dark Woofer Cone
  let cone_r = woofer_r * 0.76;
  c.set_fill(Fill::Solid(Color::rgba(0.04, 0.03, 0.06, 0.98)));
  c.set_stroke(Fill::Solid(Color::rgba(0.3, 0.28, 0.38, 0.7)));
  c.set_line_width(2.0);
  c.fill_ellipse(center_x, center_y, cone_r, cone_r);
  c.stroke_circle(center_x, center_y, cone_r);

  // Golden / Copper Center Dust Cap (Pumping with Bass!)
  let cap_r = cone_r * (0.34 + be * 0.08);
  let gold_col = Color::rgba(1.0, 0.72, 0.2, 0.98);

  c.set_fill(Fill::Solid(gold_col));
  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(2.0);
  c.set_shadow(gold_col.with_alpha(0.9), 16.0);
  c.fill_ellipse(center_x, center_y, cap_r, cap_r);
  c.stroke_circle(center_x, center_y, cap_r);

  c.restore();
}
