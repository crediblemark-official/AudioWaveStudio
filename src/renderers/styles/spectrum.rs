//! Spectrum Bars style renderer (`spectrum`).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let react = &ctx.config.reactivity;
  let bar_count = react.bar_count;
  let bar_width_cfg = react.bar_width;
  let bar_gap = react.bar_gap;
  let bar_rounding = react.bar_rounding;
  let sensitivity = react.sensitivity;
  let mirror_bars = react.mirror_bars;
  let show_peaks = react.show_peaks;
  // TS: `config.reactivity.peakColor || theme.accentColor` — an empty peak
  // color falls back to the accent color, never to the dark #0b0c10 default.
  let peak_color = if react.peak_color.trim().is_empty() {
    theme_accent(theme)
  } else {
    Color::hex(&react.peak_color)
  };

  let available_width = ctx.width * 0.85;
  let user_scale = ctx.config.scale.clamp(0.1, 5.0);
  let pos_offset_x = ctx.config.position_x * ctx.width * 0.5;
  let pos_offset_y = -ctx.config.position_y * ctx.height * 0.5;
  let total_gap = bar_gap * (bar_count as f32 - 1.0);
  let bar_width = if bar_width_cfg > 0.0 {
    bar_width_cfg * user_scale
  } else {
    ((available_width * user_scale - total_gap) / bar_count as f32).max(3.0)
  };
  let start_x = (ctx.width * 0.5 + pos_offset_x) - (bar_count as f32 * bar_width + total_gap) / 2.0;
  let max_bar_height = ctx.height * 0.45 * user_scale;
  let center_y = ctx.height * 0.5 + pos_offset_y + ctx.height * 0.05 * user_scale;

  let gradient = Fill::linear_gradient(0.0, center_y, 0.0, center_y - max_bar_height, &[
    (0.0, theme_secondary(theme)),
    (0.6, theme_primary(theme)),
    (1.0, theme_accent(theme)),
  ]);

  c.set_fill(gradient.clone());
  c.set_shadow(theme_glow(theme), 15.0);

  let step = ((ctx.freq_data.len() as f32) / bar_count as f32).floor().max(1.0) as usize;

  if ctx.state.peak_data.len() < bar_count {
    ctx.state.peak_data.resize(bar_count, 0.0);
  }

  for i in 0..bar_count {
    let val = bin_value(ctx.freq_data, step, i).min(1.0) * sensitivity;
    let bar_height = val * max_bar_height;
    let x = start_x + i as f32 * (bar_width + bar_gap);

    if bar_rounding > 0.0 {
      c.fill_rounded_rect_top(x, center_y - bar_height, bar_width, bar_height, bar_rounding);
    } else {
      c.fill_rect(x, center_y - bar_height, bar_width, bar_height);
    }

    if mirror_bars {
      c.set_global_alpha(0.6);
      if bar_rounding > 0.0 {
        c.fill_rounded_rect_bottom(x, center_y + 2.0, bar_width, bar_height, bar_rounding);
      } else {
        c.fill_rect(x, center_y + 2.0, bar_width, bar_height);
      }
      c.set_global_alpha(1.0);
    }

    if show_peaks {
      let peak = &mut ctx.state.peak_data[i];
      if bar_height > *peak {
        *peak = bar_height;
      } else {
        *peak = (*peak - 2.0).max(0.0);
      }
      // TS: `if (isPeaks && peakData[i] > 2)` — peaks decay by 2/frame, so
      // values in (0, 2] are below the draw threshold in the preview.
      if *peak > 2.0 {
        c.set_fill(Fill::Solid(peak_color));
        c.fill_rect(x, center_y - *peak - 4.0, bar_width, 3.0);
        if mirror_bars {
          c.fill_rect(x, center_y + *peak + 2.0, bar_width, 3.0);
        }
        c.set_fill(gradient.clone());
      }
    }
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
}
