//! 3D Audio Waterfall style renderer (`waterfall3D`) — Cascading Spectrum Waterfall Engine.
//!
//! Renders a genuine audio FFT waterfall: the current spectrum frame pours in at
//! the front and cascades upward in a receding column of history rows that fade
//! and compress toward the horizon like flowing water.
//! Features:
//! - Rolling spectrum history maintained in `frame_history`
//! - Perspective-compressed cascade rows (front bright & tall, rear dim & short)
//! - Bass-reactive surge front wall & oscillating "surface" glint lines
//! - Falling droplet motes shimmering along the cascade face
//! - Full UI settings integration (Scale, Position X & Y, Sensitivity, Bass Boost, Bar Count).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const WATERFALL_ROWS: usize = 40;
const WATERFALL_COLS: usize = 64;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;

  let p = theme_primary(theme);
  let s = theme_secondary(theme);
  let accent = theme_accent(theme);
  let glow = theme_glow(theme);

  // Settings integration
  let sensitivity = ctx.config.reactivity.sensitivity;
  let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let frame_time = ctx.frame_time;

  let st = &mut ctx.state.advanced;

  // Rolling spectrum history (newest frame first).
  if st.frame_history.first().map(|f| f.len()) != Some(freq.len()) {
    st.frame_history.clear();
  }
  st.frame_history.insert(0, freq.to_vec());
  if st.frame_history.len() > WATERFALL_ROWS {
    st.frame_history.pop();
  }

  let cx = width * 0.5;
  let front_y = height * 0.82;
  let horizon_y = height * 0.18;
  let rows_avail = st.frame_history.len().min(WATERFALL_ROWS);

  let surf_r = (width * 0.28).clamp(140.0, 520.0);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Deep watery backdrop
  let bg = Fill::linear_gradient(
    0.0,
    0.0,
    0.0,
    height,
    &[
      (0.0, Color::rgba(0.01, 0.03, 0.06, 1.0)),
      (0.55, Color::rgba(0.02, 0.05, 0.10, 1.0)),
      (1.0, Color::rgba(0.01, 0.03, 0.05, 1.0)),
    ],
  );
  c.set_fill(bg);
  c.fill_rect(0.0, 0.0, width, height);

  // Ambient glow rising from the cascade mouth
  let glow_fill = Fill::radial_gradient(
    cx,
    front_y,
    0.0,
    cx,
    front_y,
    surf_r,
    &[
      (0.0, glow.with_alpha(0.22 + be * 0.18)),
      (0.45, accent.with_alpha(0.10)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(glow_fill);
  c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // 1. CASCADING SPECTRUM WATERFALL ROWS
  // -------------------------------------------------------------------------
  let step_f = (freq.len() / bar_count).max(1);
  let row_max_h = height * 0.36 * sensitivity;
  let row_gap = (front_y - horizon_y) / (WATERFALL_ROWS as f32 * 1.6);

  let band_w = width / (WATERFALL_COLS as f32 + 2.0);
  let burst = be * (0.18 + bs * 0.22);

  for r in 0..rows_avail {
    let f = r as f32;
    let t = f / (WATERFALL_ROWS as f32).max(1.0); // 0 front … 1 back
    let fade = 1.0 - t * 0.86;
    let persp = 1.0 - t * 0.55; // rows compress toward horizon
    let row_center_y = front_y - row_gap * f * (1.0 - t * 0.3);

    let row_col = mix(mix(s, p, fade), glow, fade * 0.35);

    // "Surface" glint line oscillating above each row
    let glint_y = row_center_y - (frame_time * 1.5 + f * 0.7).sin() * (2.5 + f * 0.35);
    c.set_stroke(Fill::Solid(row_col.with_alpha(0.14 * fade)));
    c.set_line_width(1.0);
    c.stroke_line(cx - surf_r * persp, glint_y, cx + surf_r * persp, glint_y);

    // Row bars: mirror-mapped spectrum
    for j in 0..WATERFALL_COLS {
      let mirrored = if j < WATERFALL_COLS / 2 {
        WATERFALL_COLS / 2 - j - 1
      } else {
        j - WATERFALL_COLS / 2
      };
      let k = (mirrored * step_f * (bar_count as usize / (WATERFALL_COLS / 2)).max(1))
        .min(freq.len().saturating_sub(1));
      let row_fv = (st.frame_history[r].get(k).copied().unwrap_or(0) as f32) / 255.0;

      let h_cur = (row_fv * row_max_h * persp + 4.0 * persp) * (1.0 + burst * (1.0 - t));
      let x0 = cx - surf_r * persp + j as f32 * band_w * persp;
      let y0 = row_center_y - h_cur;

      let col = mix(row_col, Color::WHITE, row_fv * 0.5 * fade);
      c.set_fill(Fill::Solid(col.with_alpha((0.42 + fade * 0.5).min(0.95))));
      c.fill_rect(x0, y0, band_w * persp * 0.72, h_cur);
    }
  }

  // -------------------------------------------------------------------------
  // 2. FRONT SURGE WALL (current frame, loudest & nearest)
  // -------------------------------------------------------------------------
  for j in 0..WATERFALL_COLS {
    let mirrored = if j < WATERFALL_COLS / 2 {
      WATERFALL_COLS / 2 - j - 1
    } else {
      j - WATERFALL_COLS / 2
    };
    let k = (mirrored * step_f * (bar_count as usize / (WATERFALL_COLS / 2)).max(1))
      .min(freq.len().saturating_sub(1));
    let fv = freq[k] as f32 / 255.0;
    let h_cur = fv * row_max_h * (1.0 + burst) + 5.0;

    let x0 = cx - surf_r + j as f32 * band_w;
    let y0 = front_y - h_cur;
    let col = mix(mix(p, s, fv), Color::WHITE, fv * 0.45);
    c.set_fill(Fill::Solid(col.with_alpha(0.95)));
    c.set_shadow(col.with_alpha(0.5), 14.0);
    c.fill_rect(x0, y0, band_w * 0.78, h_cur);
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 3. FALLING DROPLET MOTES
  // -------------------------------------------------------------------------
  for m_i in 0..18usize {
    let m_t = ((frame_time * 0.5 + m_i as f32 * 0.13) % 1.0).clamp(0.0, 1.0);
    let mx = cx + (m_i as f32 * 37.0).sin() * surf_r * 0.85;
    let my = horizon_y + (front_y - horizon_y) * m_t;
    let m_sz = 1.5 + (1.0 - m_t) * 2.2;
    let m_col = mix(glow, Color::WHITE, m_t * 0.6).with_alpha(0.25 + (1.0 - m_t) * 0.6);
    c.set_fill(Fill::Solid(m_col));
    c.fill_circle(mx, my, m_sz);
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}
