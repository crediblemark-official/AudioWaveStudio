//! Smooth Spectrum style renderer (`smoothSpectrum`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let bar_count = ctx.config.reactivity.bar_count;
  let sensitivity = ctx.config.reactivity.sensitivity;
  if bar_count < 2 {
    return;
  }

  let available_width = ctx.width * 0.92;
  let start_x = (ctx.width - available_width) / 2.0;
  let step = ((ctx.freq_data.len() as f32) / bar_count as f32).floor().max(1.0) as usize;
  let bottom_y = ctx.height * 0.85;
  let max_h = ctx.height * 0.65;
  let x_step = available_width / (bar_count as f32 - 1.0);

  let mut points: Vec<(f32, f32)> = Vec::with_capacity(bar_count);
  for i in 0..bar_count {
    let val = (bin_value(ctx.freq_data, step, i) * sensitivity).clamp(0.0, 1.0);
    let bar_h = val * max_h;
    points.push((start_x + i as f32 * x_step, bottom_y - bar_h));
  }

  let mut curve: Vec<(f32, f32)> = Vec::new();
  for i in 0..points.len() - 1 {
    let (px, py) = points[i];
    let (nx, ny) = points[i + 1];
    let (cx, cy) = ((px + nx) / 2.0, (py + ny) / 2.0);
    let seg = GpuCanvas::sample_quadratic(
      if curve.is_empty() { (px, py) } else { *curve.last().unwrap() },
      (px, py),
      (cx, cy),
      6,
    );
    if curve.is_empty() {
      curve.extend(seg);
    } else {
      curve.extend(seg.into_iter().skip(1));
    }
  }
  let last = points[points.len() - 1];
  curve.push(last);

  let mut poly: Vec<(f32, f32)> = Vec::with_capacity(curve.len() + 3);
  poly.push((points[0].0, bottom_y));
  poly.extend_from_slice(&curve);
  poly.push((last.0, bottom_y));

  let fill_grad = Fill::linear_gradient(0.0, bottom_y - max_h, 0.0, bottom_y, &[
    (0.0, theme_primary(theme)),
    (0.5, theme_secondary(theme)),
    (1.0, Color::TRANSPARENT),
  ]);
  c.save();
  c.set_fill(fill_grad);
  c.set_shadow(theme_glow(theme), 20.0);
  c.fill_polygon(&poly);
  c.restore();

  c.save();
  c.set_stroke(Fill::Solid(theme_accent(theme)));
  c.set_line_width(2.0);
  c.set_shadow(theme_glow(theme), 10.0);
  c.stroke_polyline(&curve);
  c.restore();
}
