//! Oscilloscope style renderer (`oscilloscope`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let center_y = ctx.height * 0.52;
  let len = ctx.time_data.len();
  if len < 2 {
    return;
  }
  let slice_width = ctx.width / (len as f32 - 1.0);
  let sensitivity = ctx.config.reactivity.sensitivity;

  let passes: [(f32, f32, f32, Color); 3] = [
    (0.2, 25.0, 8.0, theme_glow(theme)),
    (0.6, 15.0, 4.0, theme_secondary(theme)),
    (1.0, 6.0, 2.0, theme_primary(theme)),
  ];

  for (alpha, blur, width, color) in passes {
    c.save();
    c.set_global_alpha(alpha);
    c.set_shadow(color, blur);
    c.set_stroke(Fill::Solid(color));
    c.set_line_width(width);
    let mut pts: Vec<(f32, f32)> = Vec::with_capacity(len);
    for i in 0..len {
      let v = ctx.time_data[i] as f32 / 128.0 - 1.0;
      let y = center_y + v * (ctx.height * 0.3) * sensitivity;
      pts.push((i as f32 * slice_width, y));
    }
    c.stroke_polyline(&pts);
    c.restore();
  }
}
