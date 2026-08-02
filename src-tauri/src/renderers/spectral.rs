//! Ports of the frequency/time-domain style renderers.
//! Each mirrors its TypeScript counterpart in `src/services/renderers/`.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::{
  bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

// ---------------------------------------------------------------------------
// spectrumBars.ts
// ---------------------------------------------------------------------------

pub fn spectrum_bars(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let react = &ctx.config.reactivity;
  let bar_count = react.bar_count;
  let bar_width_cfg = react.bar_width;
  let bar_gap = react.bar_gap;
  let bar_rounding = react.bar_rounding;
  let sensitivity = react.sensitivity;
  let mirror_bars = react.mirror_bars;
  let show_peaks = react.show_peaks;
  let peak_color = Color::hex(&react.peak_color);

  let available_width = ctx.width * 0.85;
  let total_gap = bar_gap * (bar_count as f32 - 1.0);
  let bar_width = if bar_width_cfg > 0.0 {
    bar_width_cfg
  } else {
    ((available_width - total_gap) / bar_count as f32).max(3.0)
  };
  let start_x = (ctx.width - (bar_count as f32 * bar_width + total_gap)) / 2.0;
  let max_bar_height = ctx.height * 0.45;
  let center_y = ctx.height * 0.55;

  let gradient = Fill::linear_gradient(0.0, center_y, 0.0, center_y - max_bar_height, &[
    (0.0, theme_secondary(theme)),
    (0.6, theme_primary(theme)),
    (1.0, theme_accent(theme)),
  ]);

  c.set_fill(gradient.clone());
  c.set_shadow(theme_glow(theme), 15.0);

  let step = ((ctx.freq_data.len() as f32) / bar_count as f32).floor().max(1.0) as usize;

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
      c.set_global_alpha(0.4);
      c.fill_rect(x, center_y + 2.0, bar_width, bar_height * 0.5);
      c.set_global_alpha(1.0);
    }

    if show_peaks {
      let peak = &mut ctx.state.peak_data[i];
      if bar_height > *peak {
        *peak = bar_height;
      } else {
        *peak = (*peak - 2.0).max(0.0);
      }
      if *peak > 0.0 {
        c.set_fill(Fill::Solid(peak_color));
        c.fill_rect(x, center_y - *peak - 4.0, bar_width, 3.0);
        c.set_fill(gradient.clone());
      }
    }
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
}

// ---------------------------------------------------------------------------
// minimalWave.ts
// ---------------------------------------------------------------------------

pub fn minimal_wave(c: &mut GpuCanvas, ctx: &mut RenderContext) {
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

// ---------------------------------------------------------------------------
// radial.ts
// ---------------------------------------------------------------------------

pub fn radial(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let center_x = ctx.width / 2.0;
  let center_y = ctx.height * 0.48;
  let base_radius = ctx.width.min(ctx.height) * 0.18 + ctx.bass_energy * 18.0;
  let bar_count = ctx.config.reactivity.bar_count.min(96);
  let sensitivity = ctx.config.reactivity.sensitivity;

  if let Some(img) = &ctx.state.radial_center_image {
    let img_size = (base_radius - 5.0).max(0.0) * 2.0;
    let (iw, ih) = (img.w as f32, img.h as f32);
    if iw > 0.0 && ih > 0.0 {
      let s = (img_size / iw).min(img_size / ih);
      let w = iw * s;
      let h = ih * s;
      let ox = center_x - w / 2.0;
      let oy = center_y - h / 2.0;
      let layer_size = crate::gpu2d::LAYER_SIZE as f32;
      c.push_textured_quad(
        img.layer,
        ox,
        oy,
        w,
        h,
        [0.0, 0.0, iw / layer_size, ih / layer_size],
        Color::rgba(1.0, 1.0, 1.0, 1.0),
      );
    }
  } else {
    // Center disc (fallback path when no center image).
    c.save();
    let disc_grad = Fill::radial_gradient(center_x, center_y, 5.0, center_x, center_y, base_radius, &[
      (0.0, theme_primary(theme)),
      (1.0, theme_secondary(theme)),
    ]);
    c.set_fill(disc_grad);
    c.fill_circle(center_x, center_y, (base_radius - 5.0).max(0.0));
    c.restore();
  }

  // Outer ring.
  c.save();
  c.set_line_width(4.0);
  c.set_stroke(Fill::Solid(theme_accent(theme)));
  c.set_shadow(theme_glow(theme), 20.0);
  c.stroke_circle(center_x, center_y, base_radius);
  c.restore();

  let max_spike = ctx.width.min(ctx.height) * 0.25;
  let step = ((ctx.freq_data.len() as f32) / bar_count as f32).floor().max(1.0) as usize;

  c.save();
  c.set_shadow(theme_glow(theme), 12.0);

  for i in 0..bar_count {
    let val = (bin_value(ctx.freq_data, step, i) * sensitivity).min(1.0);
    let spike_h = val * max_spike;

    let angle = (i as f32 / bar_count as f32) * TAU + ctx.rotation_angle;
    let (cos, sin) = angle.sin_cos();

    let x1 = center_x + cos * base_radius;
    let y1 = center_y + sin * base_radius;
    let x2 = center_x + cos * (base_radius + spike_h);
    let y2 = center_y + sin * (base_radius + spike_h);

    let spike_grad = Fill::linear_gradient(x1, y1, x2, y2, &[
      (0.0, theme_primary(theme)),
      (1.0, theme_accent(theme)),
    ]);
    c.set_stroke(spike_grad);
    c.set_line_width(((TAU * 2.0 * base_radius) / bar_count as f32 - 3.0).max(2.0));
    c.stroke_line(x1, y1, x2, y2);
  }
  c.restore();
}

// ---------------------------------------------------------------------------
// circularBars.ts
// ---------------------------------------------------------------------------

pub fn circular_bars(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let center_x = ctx.width / 2.0;
  let center_y = ctx.height / 2.0;
  let bar_count = ctx.config.reactivity.bar_count.min(64);
  let sensitivity = ctx.config.reactivity.sensitivity;

  let step = ((ctx.freq_data.len() as f32) / bar_count as f32).floor().max(1.0) as usize;
  let max_len = ctx.width.min(ctx.height) * 0.42;
  let min_radius = 20.0;

  c.save();
  c.set_shadow(theme_glow(theme), 10.0);
  for i in 0..bar_count {
    let val = (bin_value(ctx.freq_data, step, i) * sensitivity).min(1.0);
    let bar_len = min_radius + val * max_len;
    let angle = (i as f32 / bar_count as f32) * TAU - std::f32::consts::FRAC_PI_2;
    let (cos, sin) = angle.sin_cos();

    let x1 = center_x + cos * min_radius;
    let y1 = center_y + sin * min_radius;
    let x2 = center_x + cos * bar_len;
    let y2 = center_y + sin * bar_len;

    let col = if i % 2 == 0 { theme_primary(theme) } else { theme_secondary(theme) };
    c.set_stroke(Fill::Solid(col));
    c.set_line_width(3.0);
    c.stroke_line(x1, y1, x2, y2);
  }
  c.restore();

  c.save();
  let glow_grad = Fill::radial_gradient(center_x, center_y, 0.0, center_x, center_y, min_radius, &[
    (0.0, theme_accent(theme)),
    (1.0, Color::TRANSPARENT),
  ]);
  c.set_fill(glow_grad);
  c.set_shadow(theme_glow(theme), 30.0);
  c.fill_circle(center_x, center_y, min_radius);
  c.restore();
}

// ---------------------------------------------------------------------------
// oscilloscope.ts
// ---------------------------------------------------------------------------

pub fn oscilloscope(c: &mut GpuCanvas, ctx: &mut RenderContext) {
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

// ---------------------------------------------------------------------------
// waveformFill.ts
// ---------------------------------------------------------------------------

pub fn waveform_fill(c: &mut GpuCanvas, ctx: &mut RenderContext) {
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

  // Fill (closed polygon down to the bottom edge).
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
  c.restore();

  c.save();
  c.set_stroke(Fill::Solid(theme_accent(theme)));
  c.set_line_width(2.0);
  c.set_shadow(theme_glow(theme), 10.0);
  c.stroke_polyline(&pts);
  c.restore();
}

// ---------------------------------------------------------------------------
// equalizerMatrix.ts
// ---------------------------------------------------------------------------

pub fn equalizer_matrix(c: &mut GpuCanvas, ctx: &mut RenderContext) {
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

// ---------------------------------------------------------------------------
// smoothSpectrum.ts
// ---------------------------------------------------------------------------

pub fn smooth_spectrum(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let theme = &ctx.config.theme;
  let bar_count = ctx.config.reactivity.bar_count;
  let sensitivity = ctx.config.reactivity.sensitivity;
  if bar_count < 2 {
    return;
  }

  let available_width = ctx.width * 0.92;
  let start_x = (ctx.width - available_width) / 2.0;
  let step = ((ctx.freq_data.len() as f32) / bar_count as f32).floor().max(1.0) as usize;
  let bottom_y = ctx.height * 0.85;
  let max_h = ctx.height * 0.65;
  let x_step = available_width / (bar_count as f32 - 1.0);

  let mut points: Vec<(f32, f32)> = Vec::with_capacity(bar_count);
  for i in 0..bar_count {
    let val = (bin_value(ctx.freq_data, step, i) * sensitivity).clamp(0.0, 1.0);
    let bar_h = val * max_h;
    points.push((start_x + i as f32 * x_step, bottom_y - bar_h));
  }

  // Curve = series of quadratic segments through midpoints (quadraticCurveTo).
  let mut curve: Vec<(f32, f32)> = Vec::new();
  for i in 0..points.len() - 1 {
    let (px, py) = points[i];
    let (nx, ny) = points[i + 1];
    let (cx, cy) = ((px + nx) / 2.0, (py + ny) / 2.0);
    let seg = GpuCanvas::sample_quadratic(
      if curve.is_empty() { (px, py) } else { *curve.last().unwrap() },
      (px, py),
      (cx, cy),
      6,
    );
    if curve.is_empty() {
      curve.extend(seg);
    } else {
      curve.extend(seg.into_iter().skip(1));
    }
  }
  let last = points[points.len() - 1];
  curve.push(last);

  // Fill: bottom-left -> curve -> bottom-right.
  let mut poly: Vec<(f32, f32)> = Vec::with_capacity(curve.len() + 3);
  poly.push((points[0].0, bottom_y));
  poly.extend_from_slice(&curve);
  poly.push((last.0, bottom_y));

  let fill_grad = Fill::linear_gradient(0.0, bottom_y - max_h, 0.0, bottom_y, &[
    (0.0, theme_primary(theme)),
    (0.5, theme_secondary(theme)),
    (1.0, Color::TRANSPARENT),
  ]);
  c.save();
  c.set_fill(fill_grad);
  c.set_shadow(theme_glow(theme), 20.0);
  c.fill_polygon(&poly);
  c.restore();

  c.save();
  c.set_stroke(Fill::Solid(theme_accent(theme)));
  c.set_line_width(2.0);
  c.set_shadow(theme_glow(theme), 10.0);
  c.stroke_polyline(&curve);
  c.restore();
}
