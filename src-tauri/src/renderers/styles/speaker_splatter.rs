//! Speaker Splatter style renderer (`speakerSplatter`).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas, LineCap};
use crate::renderers::helpers::SplatterDot;
use crate::renderers::RenderContext;

use super::woofer::{draw_woofer, WooferStyle};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let a = crate::renderers::theme_accent(theme);
  let g = crate::renderers::theme_glow(theme);
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
