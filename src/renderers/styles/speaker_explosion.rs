//! Speaker Explosion style renderer (`speakerExplosion`).
//!
//! Renders an explosive audio subwoofer speaker complete with pumping bass cone,
//! chrome mounting chassis with 6 hex bolts, rubber surround roll, golden dust cap,
//! 360-degree needle-sharp radial spectrum ray burst, paint drips, and floating splatter droplets.

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
  // 4. HYPER-REALISTIC HIGH-FIDELITY SUBWOOFER ASSEMBLY
  // -------------------------------------------------------------------------
  // A. Outer Chrome / Cyan Metallic Chassis Frame Ring
  let rim_grad = Fill::radial_gradient(
    center_x,
    center_y,
    woofer_r * 0.85,
    center_x,
    center_y,
    woofer_r,
    &[
      (0.0, Color::rgba(0.1, 0.12, 0.16, 0.98)),
      (0.5, Color::rgba(0.85, 0.9, 0.95, 0.95)),
      (0.85, Color::rgba(0.2, 0.85, 1.0, 0.95)),
      (1.0, Color::rgba(0.08, 0.1, 0.14, 0.98)),
    ],
  );

  c.set_fill(rim_grad);
  c.set_shadow(electric_cyan.with_alpha(0.85), 22.0);
  c.fill_ellipse(center_x, center_y, woofer_r, woofer_r);

  c.set_stroke(Fill::Solid(electric_cyan));
  c.set_line_width(2.5);
  c.stroke_circle(center_x, center_y, woofer_r);

  // 6 Silver Hex Mounting Screws/Bolts on Rim
  let bolt_r = (woofer_r * 0.04).clamp(3.0, 8.0);
  let bolt_dist = woofer_r * 0.91;
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 4.0);

  for b_idx in 0..6 {
    let b_angle = (b_idx as f32 / 6.0) * TAU;
    let bx = center_x + b_angle.cos() * bolt_dist;
    let by = center_y + b_angle.sin() * bolt_dist;

    c.set_fill(Fill::Solid(Color::rgba(0.88, 0.88, 0.92, 0.98)));
    c.fill_ellipse(bx, by, bolt_r, bolt_r);
    c.set_fill(Fill::Solid(Color::rgba(0.2, 0.2, 0.25, 0.95)));
    c.fill_ellipse(bx, by, bolt_r * 0.45, bolt_r * 0.45);
  }

  // B. Corrugated Rubber Surround Suspension Ring
  let surround_r = woofer_r * 0.82;
  c.set_fill(Fill::Solid(Color::rgba(0.08, 0.07, 0.1, 0.98)));
  c.set_stroke(Fill::Solid(Color::rgba(0.25, 0.22, 0.32, 0.8)));
  c.set_line_width(3.0);
  c.fill_ellipse(center_x, center_y, surround_r, surround_r);
  c.stroke_circle(center_x, center_y, surround_r);

  // C. Deep Carbon / Paper Pulp Speaker Cone
  let cone_r = woofer_r * 0.66;
  let cone_grad = Fill::radial_gradient(
    center_x,
    center_y,
    0.0,
    center_x,
    center_y,
    cone_r,
    &[
      (0.0, Color::rgba(0.04, 0.03, 0.06, 0.98)),
      (0.65, Color::rgba(0.12, 0.1, 0.16, 0.98)),
      (1.0, Color::rgba(0.06, 0.05, 0.08, 0.98)),
    ],
  );

  c.set_fill(cone_grad);
  c.fill_ellipse(center_x, center_y, cone_r, cone_r);

  // Concentric cone texture rings
  c.set_stroke(Fill::Solid(Color::rgba(0.25, 0.22, 0.32, 0.3)));
  c.set_line_width(1.0);
  for &cr in &[0.3f32, 0.5, 0.7, 0.9] {
    c.stroke_circle(center_x, center_y, cone_r * cr);
  }

  // D. Golden Copper Metallic Dust Cap Dome (Pumping with Bass!)
  let cap_r = cone_r * (0.36 + be * 0.08);

  let cap_grad = Fill::radial_gradient(
    center_x - cap_r * 0.25,
    center_y - cap_r * 0.25,
    0.0,
    center_x,
    center_y,
    cap_r,
    &[
      (0.0, Color::rgba(1.0, 0.92, 0.55, 0.98)),
      (0.4, Color::rgba(1.0, 0.7, 0.15, 0.98)),
      (0.85, Color::rgba(0.75, 0.42, 0.08, 0.98)),
      (1.0, Color::rgba(0.45, 0.2, 0.04, 0.98)),
    ],
  );

  c.set_fill(cap_grad);
  c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.85, 0.4, 0.9)));
  c.set_line_width(2.0);
  c.set_shadow(Color::rgba(1.0, 0.65, 0.1, 0.9), 18.0);
  c.fill_ellipse(center_x, center_y, cap_r, cap_r);
  c.stroke_circle(center_x, center_y, cap_r);

  // 3D Specular Highlight Reflection Spot
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.65)));
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_ellipse(center_x - cap_r * 0.3, center_y - cap_r * 0.3, cap_r * 0.22, cap_r * 0.15);

  c.restore();
}
