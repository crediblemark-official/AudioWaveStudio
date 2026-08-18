//! Speaker Trio style renderer (`speakerTrio`).

use crate::gpu2d::text::{TextAlign, TextOpts};
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{bin_sum, bright, mix, FloatingNote};
use crate::renderers::RenderContext;

use super::woofer::{draw_woofer, WooferStyle};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let a = crate::renderers::theme_accent(theme);
  let glow = crate::renderers::theme_glow(theme);
  let sensitivity = ctx.config.reactivity.sensitivity;
  let user_scale = ctx.config.scale.clamp(0.1, 5.0);
  let pos_offset_x = ctx.config.position_x * width * 0.5;
  let pos_offset_y = -ctx.config.position_y * height * 0.5;
  let bar_count = ctx.config.reactivity.bar_count.clamp(8, 128);
  let be = ctx.bass_energy;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  const SYMBOLS: [char; 6] = ['\u{266A}', '\u{266B}', '\u{266C}', '\u{2669}', '\u{222E}', '\u{1F3BC}'];
  if st.notes.is_empty() {
    for _ in 0..18 {
      st.notes.push(FloatingNote {
        x: rng.next() * width,
        y: rng.next() * height,
        vx: (rng.next() - 0.5) * 0.8,
        vy: -0.5 - rng.next() * 1.2,
        symbol: SYMBOLS[(rng.next() * SYMBOLS.len() as f32) as usize],
        size: 14.0 + rng.next() * 18.0,
        alpha: 0.3 + rng.next() * 0.5,
        rotation: (rng.next() - 0.5) * 0.5,
        rot_speed: (rng.next() - 0.5) * 0.02,
      });
    }
  }

  let center_x = width * 0.5 + pos_offset_x;
  let center_y = height * 0.5 + pos_offset_y;
  let base_r = width.min(height) * 0.14 * user_scale;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let half_count = (bar_count / 2).max(4);
  let step = ((ctx.freq_data.len() as f32 * 0.5) as usize / half_count).max(1);
  let start_x = (center_x - width * 0.46 * user_scale).max(width * 0.02);
  let half_w = (center_x - start_x - base_r * 2.2).max(20.0);
  let bar_w = ((half_w / half_count as f32) - 1.5).max(2.0);
  let max_bar_h = height * 0.32 * user_scale;

  c.set_shadow(glow, 18.0 + be * 15.0);
  c.set_stroke(Fill::Solid(p));
  c.set_line_width(1.5);
  c.stroke_line(start_x, center_y, width - start_x, center_y);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  for i in 0..half_count {
    let val = bin_sum(ctx.freq_data, step, i) * sensitivity;
    if val < 0.02 {
      continue;
    }
    let bar_h = 1.0f32.max(val * max_bar_h);
    let x_left = start_x + i as f32 * (bar_w + 1.5);
    let x_right = width - start_x - (i as f32 + 1.0) * (bar_w + 1.5);
    let y_top = center_y - bar_h;
    let bright_f = 0.5 + val * 0.5;
    let col = bright(mix(p, s, i as f32 / half_count as f32), bright_f);
    c.set_fill(Fill::Solid(col));
    c.set_shadow(glow, 8.0 + val * 12.0);
    c.fill_rect(x_left, y_top, bar_w, bar_h);
    c.fill_rect(x_right, y_top, bar_w, bar_h);
    if val > 0.15 {
      c.set_fill(Fill::Solid(a.with_alpha(val * 0.3)));
      c.set_shadow(Color::TRANSPARENT, 0.0);
      c.fill_rect(x_left, y_top, bar_w, 1.5);
      c.fill_rect(x_right, y_top, bar_w, 1.5);
    }
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);

  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.4), 4.0);
  // TS draws EVERY floating note (initNotes creates 18) with a FIXED 24px
  // font (`c.font = '24px sans-serif'` — the per-note `size` field is never
  // used for sizing in the preview). The old `drawn >= 10` cap dropped 8
  // notes and the per-note size made exports look busier/denser than preview.
  for i in 0..st.notes.len() {
    let (x, y, rotation, symbol, alpha) = {
      let n = &mut st.notes[i];
      n.y += n.vy - be * 1.5;
      n.x += n.vx + (n.y * 0.02).sin() * 0.5;
      n.rotation += n.rot_speed;
      if n.y < -30.0 {
        n.y = height + 20.0;
        n.x = rng.next() * width;
      }
      (n.x, n.y, n.rotation, n.symbol, n.alpha)
    };
    c.save();
    c.translate(x, y);
    c.rotate(rotation);
    c.draw_text(
      &symbol.to_string(),
      0.0,
      0.0,
      24.0,
      "sans-serif",
      400.0,
      false,
      TextAlign::Center,
      Fill::Solid(a.with_alpha(alpha)),
      1.0,
      &TextOpts::default(),
    );
    c.restore();
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let left_x = center_x - base_r * 1.25;
  let right_x = center_x + base_r * 1.25;
  let left_r = base_r * 0.82 * (1.0 + be * 0.08);
  let right_r = base_r * 0.82 * (1.0 + be * 0.08);
  let center_r = base_r * 1.12 * (1.0 + be * 0.14 + ctx.beat_strength * 0.08);

  let trio_style = WooferStyle {
    rim_stops: &[
      (0.0, Color::hex("#FFFFFF")),
      (0.2, Color::hex("#999999")),
      (0.5, Color::hex("#222222")),
      (0.8, Color::hex("#CCCCCC")),
      (1.0, Color::hex("#444444")),
    ],
    bolt_r: 3.0,
    bolt_color: Color::hex("#DDDDDD"),
    ring_alpha: 0.08,
    ring_step: 10.0,
    ring_start: 6.0,
    ring_end_margin: 4.0,
    ring_width: 1.2,
    cone_inner_ratio: 0.32,
    rubber_stops: &[
      (0.0, Color::hex("#1A1A1E")),
      (0.5, Color::hex("#3A3A40")),
      (1.0, Color::hex("#101014")),
    ],
    cone_stops: &[
      (0.0, Color::hex("#444855")),
      (0.6, Color::hex("#22242C")),
      (1.0, Color::hex("#111216")),
    ],
    dust_mid: Color::hex("#30333D"),
    shadow_blur: 18.0,
    shadow_color: Color::rgba(0.0, 0.0, 0.0, 0.6),
    dust_scale: 0.06,
    dust_shadow: 10.0,
    crescent_alpha: 0.35,
  };

  draw_woofer(c, left_x, center_y, left_r, false, be, &trio_style);
  draw_woofer(c, right_x, center_y, right_r, false, be, &trio_style);
  draw_woofer(c, center_x, center_y, center_r, true, be, &trio_style);

  c.restore();
}
