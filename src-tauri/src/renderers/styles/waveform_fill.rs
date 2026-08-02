//! Waveform Fill style renderer (`waveformFill`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let center_y = ctx.height * 0.55;
  let len = ctx.time_data.len();
  if len < 2 {
    return;
  }
  let slice_width = ctx.width / (len as f32 - 1.0);
  let sensitivity = ctx.config.reactivity.sensitivity;

  let mut pts: Vec<(f32, f32)> = Vec::with_capacity(len);
  for i in 0..len {
    let v = ctx.time_data[i] as f32 / 128.0 - 1.0;
    let y = center_y + v * (ctx.height * 0.28) * sensitivity;
    pts.push((i as f32 * slice_width, y));
  }

  let mirror = ctx.config.reactivity.mirror_bars;

  let mut poly: Vec<(f32, f32)> = pts.clone();
  poly.push((ctx.width, ctx.height));
  poly.push((0.0, ctx.height));

  let fill_grad = Fill::linear_gradient(0.0, 0.0, 0.0, ctx.height, &[
    (0.0, theme_primary(theme)),
    (0.5, theme_secondary(theme)),
    (1.0, Color::TRANSPARENT),
  ]);
  c.save();
  c.set_fill(fill_grad);
  c.set_shadow(theme_glow(theme), 20.0);
  c.fill_polygon(&poly);

  if mirror {
    let mirror_pts: Vec<(f32, f32)> = pts
      .iter()
      .map(|&(x, y)| (x, center_y - (y - center_y)))
      .collect();
    let mut mirror_poly: Vec<(f32, f32)> = mirror_pts.clone();
    mirror_poly.push((ctx.width, 0.0));
    mirror_poly.push((0.0, 0.0));

    c.set_global_alpha(0.5);
    c.fill_polygon(&mirror_poly);
    c.set_global_alpha(1.0);
  }

  c.restore();

  c.save();
  c.set_stroke(Fill::Solid(theme_accent(theme)));
  c.set_line_width(2.0);
  c.set_shadow(theme_glow(theme), 10.0);
  c.stroke_polyline(&pts);

  if mirror {
    let mirror_pts: Vec<(f32, f32)> = pts
      .iter()
      .map(|&(x, y)| (x, center_y - (y - center_y)))
      .collect();
    c.set_global_alpha(0.6);
    c.stroke_polyline(&mirror_pts);
    c.set_global_alpha(1.0);
  }

  c.restore();
}
