//! Speaker 3D style renderer (`speaker3D`).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::bin_sum;
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let a = crate::renderers::theme_accent(theme);
  let glow = crate::renderers::theme_glow(theme);
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
  while gy <= center_y + cone_outer {
    // TS: `Math.floor((gy - centerY) / gridSpacing) % 2 === 0 ? 0 : gridSpacing * 0.5`
    // (speaker3D.ts:225). The parity comes from the absolute gy, NOT a row
    // counter — a plain counter phase-inverts the whole dot mesh whenever
    // `ceil(coneOuter / grid)` is odd.
    let row_offset = if (((gy - center_y) / grid).floor() as i32) % 2 == 0 { 0.0 } else { grid * 0.5 };
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
