//! Minimal Wave style renderer (`minimal`).

use crate::gpu2d::{Fill, GpuCanvas};
use crate::renderers::{bin_value, theme_primary, RenderContext};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let react = &ctx.config.reactivity;
  let sensitivity = react.sensitivity;
  let bar_count = react.bar_count.max(64);
  let available_width = ctx.width * 0.9;
  let step = ((ctx.freq_data.len() as f32) / bar_count as f32).floor().max(1.0) as usize;
  let bar_width = available_width / bar_count as f32 - 3.0;
  let gap = 3.0;
  let start_x = (ctx.width - (bar_count as f32 * bar_width + (bar_count as f32 - 1.0) * gap)) / 2.0;
  let max_bar_height = ctx.height * 0.7;
  let center_y = ctx.height / 2.0;

  c.set_fill(Fill::Solid(theme_primary(theme)));

  for i in 0..bar_count {
    let value = bin_value(ctx.freq_data, step, i).min(1.0) * sensitivity;
    let bar_height = (value * max_bar_height).max(4.0);
    let x = start_x + i as f32 * (bar_width + gap);
    c.fill_rounded_rect(x, center_y - bar_height, bar_width, bar_height * 2.0, bar_width / 2.0);
  }
}
