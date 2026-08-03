//! VU Meter style renderer (`vuMeter`).

use std::f32::consts::PI;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let sensitivity = ctx.config.reactivity.sensitivity;

  let cx = ctx.width / 2.0;
  let cy = ctx.height / 2.0;
  let radius = ctx.width.min(ctx.height) * 0.38;

  for ch in 0..2 {
    let start_bin = ch * 6;
    let mut sum = 0usize;
    let mut n = 0;
    for i in 0..6 {
      let k = start_bin + i;
      if k < ctx.freq_data.len() {
        sum += ctx.freq_data[k] as usize;
        n += 1;
      }
    }
    let raw = if n > 0 { (sum as f32 / (n as f32 * 255.0)) * sensitivity } else { 0.0 };

    let ch_state = &mut ctx.state.vu[ch];
    ch_state.level += (raw.min(1.0) - ch_state.level) * 0.3;
    ch_state.peak = ch_state.peak.max(ch_state.level);
    ch_state.peak *= 0.92;
    ch_state.peak_hold = ch_state.peak_hold.max(ch_state.peak);
    ch_state.peak_hold -= 0.003;
    if ch_state.peak_hold < 0.0 {
      ch_state.peak_hold = 0.0;
    }
  }

  let spacing = radius * 0.6;
  let gap = radius * 0.15;

  for ch in 0..2 {
    let x = if ch == 0 { cx - spacing - gap } else { cx + spacing + gap };
    let y = cy;
    let ch_state = &ctx.state.vu[ch];
    let level = ch_state.level;
    let peak_hold = ch_state.peak_hold;

    let green_angle = (-0.75 + level * 2.5).max(0.0);

    c.save();
    c.translate(x, y);

    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.1)));
    c.set_line_width(6.0);
    c.stroke_arc(0.0, 0.0, radius, PI * 0.8, PI * 0.2);

    let active_color = if level > 0.7 {
      if ch == 0 { theme_accent(theme) } else { Color::hex("#ff3333") }
    } else if ch == 0 {
      theme_primary(theme)
    } else {
      theme_secondary(theme)
    };
    c.save();
    c.set_stroke(Fill::Solid(active_color));
    c.set_shadow(theme_glow(theme), 12.0);
    c.stroke_arc(0.0, 0.0, radius, PI * 0.8, PI * 0.8 + green_angle);
    c.restore();

    let needle_angle = PI * 0.8 + level * 2.5;
    c.save();
    c.set_stroke(Fill::Solid(theme_accent(theme)));
    c.set_line_width(3.0);
    c.set_shadow(theme_glow(theme), 8.0);
    c.stroke_line(0.0, 0.0, needle_angle.cos() * radius * 0.7, needle_angle.sin() * radius * 0.7);
    c.restore();

    c.set_fill(Fill::Solid(Color::WHITE));
    c.fill_circle(0.0, 0.0, 6.0);

    let hold_angle = PI * 0.8 + peak_hold * 2.5;
    c.set_fill(Fill::Solid(Color::WHITE));
    c.fill_circle(hold_angle.cos() * radius, hold_angle.sin() * radius, 4.0);

    c.restore();
  }

  let label_size = (ctx.width * 0.025).min(16.0);
  c.draw_text(
    "VU METER",
    cx,
    ctx.height - 15.0,
    label_size,
    "monospace",
    400.0,
    false,
    crate::gpu2d::text::TextAlign::Center,
    Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.4)),
    1.0,
    &Default::default(),
  );
}
