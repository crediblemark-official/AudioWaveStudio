//! Speaker woofer visualizer styles (`speaker3D`, `speakerTrio`, `speakerSplatter`).

use std::f32::consts::TAU;

use crate::gpu2d::text::{TextAlign, TextOpts};
use crate::gpu2d::{Color, Fill, GpuCanvas, LineCap};

use crate::renderers::RenderContext;

use super::{bin_sum, bright, mix, FloatingNote, SplatterDot};

// ---------------------------------------------------------------------------
// speaker3D
// ---------------------------------------------------------------------------

pub fn speaker_3d(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let a = crate::renderers::theme_accent(theme);
  let glow = crate::renderers::theme_glow(theme);
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;

  let center_x = width / 2.0;
  let center_y = height / 2.0;
  let base_radius = width.min(height) * 0.27;
  let bass_pulse = 1.0 + be * 0.12 + bs * 0.08;
  let speaker_r = base_radius * bass_pulse;

  let bar_grad = Fill::linear_gradient(0.0, 0.0, 0.0, height, &[
    (0.0, p.with_alpha(0.85)),
    (0.3, s.with_alpha(0.95)),
    (0.6, a.with_alpha(0.98)),
    (0.85, s.with_alpha(0.95)),
    (1.0, p.with_alpha(0.85)),
  ]);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let half_bars = 48;
  let step = ((freq.len() as f32 * 0.7) as usize / half_bars).max(1);
  let left_start = width * 0.02;
  let left_end = (left_start + 20.0).max(center_x - speaker_r * 0.85);
  let left_width = left_end - left_start;
  let right_start = (width * 0.98 - 20.0).min(center_x + speaker_r * 0.85);
  let right_end = width * 0.98;
  let right_width = right_end - right_start;
  let bar_w = ((left_width / half_bars as f32) - 2.5).max(2.5);

  c.set_shadow(glow, 20.0 + be * 20.0);
  c.set_stroke(Fill::Solid(p));
  c.set_line_width(2.2);
  c.stroke_line(0.0, center_y, width, center_y);
  c.set_shadow(glow, 15.0);

  for i in 0..half_bars {
    let val = bin_sum(freq, step, i) * sensitivity;
    if val < 0.01 {
      continue;
    }
    let bar_h = val * height * 0.36;
    let top_y = center_y - bar_h;
    let bot_y = center_y + bar_h * 0.82;
    let f = i as f32 / (half_bars - 1) as f32;
    let x_left = left_end - f * left_width - bar_w;
    let x_right = right_start + f * right_width;
    c.set_fill(bar_grad.clone());
    c.fill_rect(x_left, top_y, bar_w, bar_h * 1.82);
    c.fill_rect(x_right, top_y, bar_w, bar_h * 1.82);
    c.set_fill(Fill::Solid(a));
    c.fill_rect(x_left - 0.5, top_y - 1.5, bar_w + 1.0, 1.5);
    c.fill_rect(x_left - 0.5, bot_y, bar_w + 1.0, 1.5);
    c.fill_rect(x_right - 0.5, top_y - 1.5, bar_w + 1.0, 1.5);
    c.fill_rect(x_right - 0.5, bot_y, bar_w + 1.0, 1.5);
  }

  let glow_r = speaker_r * 1.4;
  let back_glow = Fill::radial_gradient(
    center_x,
    center_y,
    speaker_r * 0.5,
    center_x,
    center_y,
    glow_r,
    &[
      (0.0, a.with_alpha(0.85)),
      (0.5, s.with_alpha(0.4)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_shadow(glow, 45.0 + be * 35.0);
  c.set_fill(back_glow);
  c.fill_circle(center_x, center_y, glow_r);

  let flare = |c: &mut GpuCanvas, fx: f32| {
    let fg = Fill::radial_gradient(
      fx,
      center_y,
      0.0,
      fx,
      center_y,
      speaker_r * 0.45,
      &[
        (0.0, Color::rgba(1.0, 1.0, 1.0, 0.95)),
        (0.2, p.with_alpha(0.85)),
        (0.6, s.with_alpha(0.3)),
        (1.0, Color::TRANSPARENT),
      ],
    );
    c.set_fill(fg);
    c.fill_circle(fx, center_y, speaker_r * 0.45);
  };
  flare(c, center_x - speaker_r * 0.96);
  flare(c, center_x + speaker_r * 0.96);

  let outer_rim = speaker_r;
  let inner_rim = speaker_r * 0.88;
  let metallic = Fill::linear_gradient(
    center_x - outer_rim,
    center_y - outer_rim,
    center_x + outer_rim,
    center_y + outer_rim,
    &[
      (0.0, Color::hex("#FFFFFF")),
      (0.15, Color::hex("#8E8E93")),
      (0.35, Color::hex("#2C2C2E")),
      (0.55, Color::hex("#D1D1D6")),
      (0.75, Color::hex("#48484A")),
      (1.0, Color::hex("#E5E5EA")),
    ],
  );
  c.set_shadow(Color::hex("#000000"), 18.0);
  c.set_fill(metallic);
  c.fill_ring(center_x, center_y, outer_rim, inner_rim);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.5)));
  c.set_line_width(1.5);
  c.stroke_circle(center_x, center_y, outer_rim - 1.0);
  c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, 0.6)));
  c.stroke_circle(center_x, center_y, inner_rim + 1.0);

  let bolt_radius = (outer_rim + inner_rim) / 2.0;
  for k in 0..4 {
    let angle = k as f32 * TAU / 4.0;
    let bx = center_x + angle.cos() * bolt_radius;
    let by = center_y + angle.sin() * bolt_radius;
    c.set_shadow(Color::hex("#000000"), 4.0);
    c.set_fill(Fill::Solid(Color::hex("#E5E5EA")));
    c.fill_circle(bx, by, 3.8);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(Color::hex("#1C1C1E")));
    c.set_line_width(1.2);
    c.stroke_line(bx - 2.0, by, bx + 2.0, by);
  }

  let surround_outer = inner_rim;
  let surround_inner = speaker_r * 0.74;
  let rubber = Fill::radial_gradient(
    center_x,
    center_y,
    surround_inner,
    center_x,
    center_y,
    surround_outer,
    &[
      (0.0, Color::hex("#1C1C1E")),
      (0.5, Color::hex("#3A3A3C")),
      (1.0, Color::hex("#0C0C0E")),
    ],
  );
  c.set_fill(rubber);
  c.fill_ring(center_x, center_y, surround_outer, surround_inner);

  let cone_outer = surround_inner;
  let cone_inner = speaker_r * 0.30;
  let cone = Fill::radial_gradient(
    center_x - cone_outer * 0.25,
    center_y - cone_outer * 0.25,
    cone_inner * 0.4,
    center_x,
    center_y,
    cone_outer,
    &[
      (0.0, Color::hex("#48484A")),
      (0.5, Color::hex("#2C2C2E")),
      (1.0, Color::hex("#1C1C1E")),
    ],
  );
  c.set_fill(cone);
  c.fill_circle(center_x, center_y, cone_outer);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.08)));
  let grid = 12.0f32;
  let dot_r = 1.8;
  let min_d2 = (cone_inner * 0.9) * (cone_inner * 0.9);
  let max_d2 = (cone_outer * 0.98) * (cone_outer * 0.98);
  let mut gy = center_y - cone_outer;
  let mut row = 0i32;
  while gy <= center_y + cone_outer {
    let row_offset = if row % 2 == 0 { 0.0 } else { grid * 0.5 };
    let mut gx = center_x - cone_outer;
    while gx <= center_x + cone_outer {
      let xp = gx + row_offset;
      let dxp = xp - center_x;
      let dyp = gy - center_y;
      let d2 = dxp * dxp + dyp * dyp;
      if d2 >= min_d2 && d2 <= max_d2 {
        c.fill_circle(xp, gy, dot_r);
      }
      gx += grid;
    }
    gy += grid;
    row += 1;
  }

  let dust_r = cone_inner * (1.0 + be * 0.06);
  let dust = Fill::radial_gradient(
    center_x - dust_r * 0.3,
    center_y - dust_r * 0.3,
    0.0,
    center_x,
    center_y,
    dust_r,
    &[
      (0.0, Color::hex("#636366")),
      (0.4, Color::hex("#3A3A3C")),
      (0.85, Color::hex("#1C1C1E")),
      (1.0, Color::hex("#0C0C0E")),
    ],
  );
  c.set_shadow(Color::hex("#000000"), 14.0 + be * 10.0);
  c.set_fill(dust);
  c.fill_circle(center_x, center_y, dust_r);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.35)));
  c.fill_ring_arc(
    center_x - dust_r * 0.15,
    center_y - dust_r * 0.15,
    dust_r * 0.65,
    dust_r * 0.45,
    TAU * 0.5,
    TAU * 0.925,
  );

  c.restore();
}

// ---------------------------------------------------------------------------
// Shared woofer (speakerTrio / speakerSplatter)
// ---------------------------------------------------------------------------

struct WooferStyle<'a> {
  rim_stops: &'a [(f32, Color)],
  bolt_r: f32,
  ring_alpha: f32,
  ring_step: f32,
  shadow_blur: f32,
}

fn draw_woofer(c: &mut GpuCanvas, x: f32, y: f32, r: f32, is_center: bool, style: &WooferStyle) {
  let outer_r = r;
  let inner_r = r * 0.86;
  let bolt_r = (outer_r + inner_r) / 2.0;

  let shadow = if is_center { style.shadow_blur } else { style.shadow_blur * 0.72 };
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), shadow);

  let metallic = Fill::linear_gradient(
    x - r,
    y - r,
    x + r,
    y + r,
    style.rim_stops,
  );
  c.set_fill(metallic);
  c.fill_ring(x, y, outer_r, inner_r);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::hex("#DDDDDD")));
  for k in 0..4 {
    let angle = k as f32 * TAU / 4.0;
    let bx = x + angle.cos() * bolt_r;
    let by = y + angle.sin() * bolt_r;
    c.fill_circle(bx, by, style.bolt_r);
  }

  let surround_inner = r * 0.72;
  let rubber = Fill::radial_gradient(x, y, surround_inner, x, y, inner_r, &[
    (0.0, Color::hex("#1A1A1E")),
    (0.5, Color::hex("#3A3A40")),
    (1.0, Color::hex("#101014")),
  ]);
  c.set_fill(rubber);
  c.fill_ring(x, y, inner_r, surround_inner);

  let cone_inner = r * 0.32;
  let cone = Fill::radial_gradient(x - r * 0.2, y - r * 0.2, cone_inner * 0.5, x, y, surround_inner, &[
    (0.0, Color::hex("#444855")),
    (0.6, Color::hex("#22242C")),
    (1.0, Color::hex("#111216")),
  ]);
  c.set_fill(cone);
  c.fill_circle(x, y, surround_inner);

  c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, style.ring_alpha)));
  c.set_line_width(1.2);
  let mut ring = cone_inner + 6.0;
  while ring < surround_inner - 4.0 {
    c.stroke_circle(x, y, ring);
    ring += style.ring_step;
  }

  let dust_r = cone_inner * 1.0;
  let dust = Fill::radial_gradient(x - dust_r * 0.3, y - dust_r * 0.3, 0.0, x, y, dust_r, &[
    (0.0, Color::hex("#666A78")),
    (0.4, Color::hex("#30333D")),
    (1.0, Color::hex("#0C0D10")),
  ]);
  c.set_shadow(Color::hex("#000000"), 10.0);
  c.set_fill(dust);
  c.fill_circle(x, y, dust_r);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.35)));
  c.fill_ring_arc(
    x - dust_r * 0.15,
    y - dust_r * 0.15,
    dust_r * 0.65,
    dust_r * 0.45,
    TAU * 0.5,
    TAU * 0.925,
  );
}

// ---------------------------------------------------------------------------
// speakerTrio
// ---------------------------------------------------------------------------

pub fn speaker_trio(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let a = crate::renderers::theme_accent(theme);
  let glow = crate::renderers::theme_glow(theme);
  let sensitivity = ctx.config.reactivity.sensitivity;
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

  let center_x = width / 2.0;
  let center_y = height / 2.0;
  let base_r = width.min(height) * 0.14;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let half_count = 40;
  let step = ((ctx.freq_data.len() as f32 * 0.5) as usize / half_count).max(1);
  let start_x = width * 0.04;
  let half_w = center_x - start_x - 4.0;
  let bar_w = ((half_w / half_count as f32) - 1.5).max(2.0);
  let max_bar_h = height * 0.32;

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
  let mut drawn = 0usize;
  for i in 0..st.notes.len() {
    if drawn >= 10 {
      break;
    }
    let (x, y, rotation, symbol, size, alpha) = {
      let n = &mut st.notes[i];
      n.y += n.vy - be * 1.5;
      n.x += n.vx + (n.y * 0.02).sin() * 0.5;
      n.rotation += n.rot_speed;
      if n.y < -30.0 {
        n.y = height + 20.0;
        n.x = rng.next() * width;
      }
      (n.x, n.y, n.rotation, n.symbol, n.size, n.alpha)
    };
    c.save();
    c.translate(x, y);
    c.rotate(rotation);
    c.draw_text(
      &symbol.to_string(),
      0.0,
      0.0,
      size,
      "sans-serif",
      400.0,
      TextAlign::Center,
      Fill::Solid(a.with_alpha(alpha)),
      1.0,
      &TextOpts::default(),
    );
    c.restore();
    drawn += 1;
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
    ring_alpha: 0.08,
    ring_step: 10.0,
    shadow_blur: 18.0,
  };

  draw_woofer(c, left_x, center_y, left_r, false, &trio_style);
  draw_woofer(c, right_x, center_y, right_r, false, &trio_style);
  draw_woofer(c, center_x, center_y, center_r, true, &trio_style);

  c.restore();
}

// ---------------------------------------------------------------------------
// speakerSplatter
// ---------------------------------------------------------------------------

pub fn speaker_splatter(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);
  let a = crate::renderers::theme_accent(theme);
  let g = crate::renderers::theme_glow(theme);
  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  let center_x = width / 2.0;
  let center_y = height / 2.0;
  let max_dim = width.min(height);
  let base_r = max_dim * 0.13;

  if st.splatter.is_empty() {
    for _ in 0..45 {
      let angle = rng.next() * TAU;
      let dist = base_r * (0.4 + rng.next() * 1.3);
      st.splatter.push(SplatterDot {
        x: center_x + angle.cos() * dist,
        y: center_y + angle.sin() * dist + base_r * 0.2,
        r: 1.2 + rng.next() * 4.5,
      });
    }
  }

  let freq_avg: f32 = ctx.freq_data.iter().map(|&b| b as f32).sum::<f32>()
    / (ctx.freq_data.len() as f32 * 255.0)
    * sensitivity;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let pulse = 1.0 + be * 0.18 + bs * 0.12;
  let arc_alpha = 0.25 + be * 0.25 + freq_avg * 0.15;
  let wide = 1.0 + be * 0.15;

  st.arc_rotation += 0.02;
  let rot = st.arc_rotation;

  let soft_stroke = |c: &mut GpuCanvas,
                         cx: f32,
                         cy: f32,
                         radius: f32,
                         start: f32,
                         end: f32,
                         color: Color,
                         alpha: f32| {
    let layers: [(f32, f32); 3] = [(10.0, 0.08), (6.0, 0.15), (2.0, 0.4)];
    for (w, la) in layers {
      c.set_line_width(w);
      c.set_stroke(Fill::Solid(color.with_alpha(alpha * la)));
      c.set_line_cap(LineCap::Round);
      c.stroke_arc(cx, cy, radius, start, end);
    }
  };

  c.set_shadow(Color::TRANSPARENT, 0.0);
  for k in 1..=4 {
    let radius = base_r * (1.0 + k as f32 * 0.35) * pulse;
    let spread = (TAU * 0.30) * wide;
    let base_angle = TAU * 0.59;
    let fade = (1.0 - (k as f32 - 1.0) * 0.28).max(0.08);
    soft_stroke(
      c,
      center_x - 6.0,
      center_y - 4.0,
      radius,
      base_angle - spread * 0.5 + rot,
      base_angle + spread * 0.5 + rot,
      p,
      arc_alpha * fade,
    );
  }

  for k in 1..=4 {
    let radius = base_r * (1.0 + k as f32 * 0.35) * pulse;
    let spread = (TAU * 0.28) * wide;
    let base_angle = TAU * 0.15;
    let fade = (1.0 - (k as f32 - 1.0) * 0.28).max(0.08);
    soft_stroke(
      c,
      center_x + 6.0,
      center_y + 4.0,
      radius,
      base_angle - spread * 0.5 - rot,
      base_angle + spread * 0.5 - rot,
      s,
      arc_alpha * fade,
    );
  }

  for k in 1..=3 {
    let radius = base_r * (1.1 + k as f32 * 0.38) * pulse;
    let spread = (TAU * 0.23) * wide;
    let base_angle = -TAU * 0.125;
    let fade = (1.0 - (k as f32 - 1.0) * 0.4).max(0.08);
    soft_stroke(
      c,
      center_x + 2.0,
      center_y - 2.0,
      radius,
      base_angle - spread * 0.5 + rot,
      base_angle + spread * 0.5 + rot,
      a,
      arc_alpha * fade * 0.7,
    );
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let glow_intensity = 0.3 + be * 0.25 + bs * 0.15;
  let glow_radius = base_r * 2.8 * pulse;
  for pos in [
    (center_x, center_y),
    (center_x - base_r * 0.92, center_y + base_r * 0.06),
    (center_x + base_r * 0.92, center_y + base_r * 0.06),
  ] {
    let grad = Fill::radial_gradient(pos.0, pos.1, 0.0, pos.0, pos.1, glow_radius, &[
      (0.0, g.with_alpha(glow_intensity * 0.5)),
      (0.15, p.with_alpha(glow_intensity * 0.2)),
      (0.4, s.with_alpha(glow_intensity * 0.08)),
      (1.0, Color::TRANSPARENT),
    ]);
    c.set_fill(grad);
    c.fill_circle(pos.0, pos.1, glow_radius);
  }

  let ink_y = center_y + base_r * 0.25;
  c.set_fill(Fill::Solid(Color::hex("#14141A")));
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 10.0);
  c.fill_ellipse(center_x, ink_y, base_r * 1.4, base_r * 0.75);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let splatter_colors = [p, s, a];
  for (i, dot) in st.splatter.iter().enumerate() {
    let sx = center_x + (dot.x - center_x) * (1.0 + be * 0.15);
    let sy = center_y + (dot.y - center_y) * (1.0 + be * 0.15);
    c.set_fill(Fill::Solid(splatter_colors[i % 3]));
    c.fill_circle(sx, sy, dot.r * (1.0 + bs * 0.3));
  }

  c.set_stroke(Fill::Solid(a));
  c.set_line_cap(LineCap::Round);
  let drip_xs = [-0.85, -0.55, -0.15, 0.15, 0.5, 0.8];
  let drip_lens = [30.0, 55.0, 75.0, 45.0, 65.0, 25.0];
  for d in 0..drip_xs.len() {
    let dx = center_x + drip_xs[d] * base_r;
    let dy = ink_y + base_r * 0.2;
    let len = drip_lens[d] * (1.0 + be * 0.3);
    let thick = 2.5 + (d % 3) as f32;
    c.set_line_width(thick);
    c.set_shadow(g, 6.0);
    c.stroke_line(dx, dy, dx, dy + len);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_fill(Fill::Solid(a));
    c.fill_circle(dx, dy + len + thick * 0.5, thick * 1.2);
  }

  let center_r = base_r * 1.15 * (1.0 + be * 0.10);
  let left_r = base_r * 0.88 * (1.0 + be * 0.07);
  let right_r = base_r * 0.88 * (1.0 + be * 0.07);
  let left_x = center_x - base_r * 0.92;
  let left_y = center_y + base_r * 0.06;
  let right_x = center_x + base_r * 0.92;
  let right_y = center_y + base_r * 0.06;

  let splatter_style = WooferStyle {
    rim_stops: &[
      (0.0, Color::hex("#FFFFFF")),
      (0.2, Color::hex("#AAAAAA")),
      (0.45, Color::hex("#222226")),
      (0.75, Color::hex("#DDDDDD")),
      (1.0, Color::hex("#55555A")),
    ],
    bolt_r: 2.5,
    ring_alpha: 0.15,
    ring_step: 8.0,
    shadow_blur: 18.0,
  };

  draw_woofer(c, left_x, left_y, left_r, false, &splatter_style);
  draw_woofer(c, right_x, right_y, right_r, false, &splatter_style);
  draw_woofer(c, center_x, center_y, center_r, true, &splatter_style);

  c.restore();
}
