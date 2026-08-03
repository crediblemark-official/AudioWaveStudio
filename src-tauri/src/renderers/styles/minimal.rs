//! Minimal Wave style renderer (`minimal`) — Rust port of
//! `src/services/renderers/minimalWave.ts`.
//!
//! The TS renderer caps the bar count at 64 (`Math.min(64, barCount)`); it
//! never raises it. Mirroring that exactly keeps slider parity: a user who
//! sets Bar Count = 16 must see 16 bars in the Rust export, not a forced 64.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{bin_value, theme_glow, theme_primary, RenderContext};

/// Bar count clamp identical to TS `Math.min(64, config.reactivity.barCount)`.
pub fn effective_bar_count(bar_count: usize) -> usize {
  bar_count.min(64)
}

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let react = &ctx.config.reactivity;
  let sensitivity = react.sensitivity;
  let bar_count = effective_bar_count(react.bar_count);
  // TS: `const availableWidth = width * 0.7;`
  let available_width = ctx.width * 0.7;
  let bar_width = available_width / bar_count as f32 - 3.0;
  // TS: `const startX = (width - availableWidth) / 2;`
  let start_x = (ctx.width - available_width) / 2.0;
  // TS: `const centerY = height * 0.55;`
  let center_y = ctx.height * 0.55;
  let step = ((ctx.freq_data.len() as f32) / bar_count as f32).floor().max(1.0) as usize;

  c.set_fill(Fill::Solid(theme_primary(theme)));
  c.set_shadow(theme_glow(theme), 10.0);

  for i in 0..bar_count {
    // TS: `val = (val / step / 255) * sensitivity;` — NO clamp, so hot bins
    // with sensitivity > 1 can exceed the nominal max (parity must match).
    let value = bin_value(ctx.freq_data, step, i) * sensitivity;
    // TS: `const barH = Math.max(4, val * height * 0.35);` — bar is centered
    // on centerY (roundRect from centerY - barH/2 with height barH).
    let bar_height = (value * ctx.height * 0.35).max(4.0);
    let x = start_x + i as f32 * (bar_width + 3.0);
    c.fill_rounded_rect(x, center_y - bar_height / 2.0, bar_width, bar_height, bar_width / 2.0);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
}
