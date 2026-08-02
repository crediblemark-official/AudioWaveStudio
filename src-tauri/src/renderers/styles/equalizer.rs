//! Matrix Equalizer style renderer (`equalizer`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let cols = ctx.config.reactivity.bar_count.min(48);
  let rows = 18;
  let available_w = ctx.width * 0.8;
  let block_w = available_w / cols as f32 - 4.0;
  let block_h = (ctx.height * 0.35) / rows as f32 - 3.0;
  let start_x = (ctx.width - available_w) / 2.0;
  let start_y = ctx.height * 0.6;

  let step = ((ctx.freq_data.len() as f32) / cols as f32).floor().max(1.0) as usize;

  for col in 0..cols {
    let val = (bin_value(ctx.freq_data, step, col) * ctx.config.reactivity.sensitivity).min(1.0);
    let active_rows = (val * rows as f32).floor() as usize;

    for r in 0..rows {
      let bx = start_x + col as f32 * (block_w + 4.0);
      let by = start_y - r as f32 * (block_h + 3.0);

      if r < active_rows {
        let col_r = if r > (rows as f32 * 0.8) as usize {
          theme_accent(theme)
        } else if r > (rows as f32 * 0.5) as usize {
          theme_primary(theme)
        } else {
          theme_secondary(theme)
        };
        c.set_fill(Fill::Solid(col_r));
        c.set_shadow(theme_glow(theme), 8.0);
        c.fill_rect(bx, by, block_w, block_h);
      } else {
        c.set_global_alpha(0.12);
        c.set_fill(Fill::Solid(Color::WHITE));
        c.fill_rect(bx, by, block_w, block_h);
        c.set_global_alpha(1.0);
      }
    }
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);
}
