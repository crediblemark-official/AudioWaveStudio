//! Circular Bars style renderer (`circularBars`).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let center_x = ctx.width / 2.0;
  let center_y = ctx.height / 2.0;
  let bar_count = ctx.config.reactivity.bar_count.min(64);
  let sensitivity = ctx.config.reactivity.sensitivity;

  let step = ((ctx.freq_data.len() as f32) / bar_count as f32).floor().max(1.0) as usize;
  let max_len = ctx.width.min(ctx.height) * 0.42;
  let min_radius = 20.0;

  c.save();
  c.set_shadow(theme_glow(theme), 10.0);
  for i in 0..bar_count {
    let val = (bin_value(ctx.freq_data, step, i) * sensitivity).min(1.0);
    let bar_len = min_radius + val * max_len;
    let angle = (i as f32 / bar_count as f32) * TAU - std::f32::consts::FRAC_PI_2;
    let (cos, sin) = angle.sin_cos();

    let x1 = center_x + cos * min_radius;
    let y1 = center_y + sin * min_radius;
    let x2 = center_x + cos * bar_len;
    let y2 = center_y + sin * bar_len;

    let col = if i % 2 == 0 { theme_primary(theme) } else { theme_secondary(theme) };
    c.set_stroke(Fill::Solid(col));
    c.set_line_width(3.0);
    c.stroke_line(x1, y1, x2, y2);
  }
  c.restore();

  c.save();
  let glow_grad = Fill::radial_gradient(center_x, center_y, 0.0, center_x, center_y, min_radius, &[
    (0.0, theme_accent(theme)),
    (1.0, Color::TRANSPARENT),
  ]);
  c.set_fill(glow_grad);
  c.set_shadow(theme_glow(theme), 30.0);
  c.fill_circle(center_x, center_y, min_radius);
  c.restore();
}
