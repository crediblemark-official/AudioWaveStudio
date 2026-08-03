//! Fire 3D style renderer (`api3D`) — faithful port of
//! `src/services/renderers/api3D.ts` (export path parity).
//!
//! Mirrors the TS model exactly: 220-point Gaussian bell-curve displacement
//! per frequency bin (with downward peaks on selected bins), the additive
//! plasma smoke glow cloud, the straight center laser baseline, the two
//! woven sub-thread lines, vertical light needles on tall peaks, the
//! three-pass hero neon waveform (7.5 → 3.6 → 1.8 px), the glossy floor
//! reflection, and floating micro embers.
//!
//! `fireWidthRatio` and `fireHeightScale` sliders drive the usable width and
//! the peak height scale, exactly like the preview.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{quadratic_wave, Ember};
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
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
  let freq = ctx.freq_data;

  let rng = &mut ctx.state.rng;
  let st = &mut ctx.state.advanced;

  let fire_width_ratio = ctx.config.reactivity.fire_width_ratio.unwrap_or(0.94);
  let fire_height_scale = ctx.config.reactivity.fire_height_scale.unwrap_or(1.0);

  // TS: `time += 0.03 + be * 0.02;`
  st.api_time += 0.03 + be * 0.02;
  let time = st.api_time;
  let center_y = height * 0.5;
  let half_margin_ratio = (1.0 - fire_width_ratio) / 2.0;
  let start_x = width * half_margin_ratio.max(0.01);
  let end_x = width * (1.0 - half_margin_ratio).min(0.99);
  let wave_width = end_x - start_x;
  let point_count = 220usize;

  let mut displacements = vec![0.0f32; point_count];
  let mut sub1 = vec![0.0f32; point_count];
  let mut sub2 = vec![0.0f32; point_count];

  // TS: `binCount = max(1, min(48, floor(freqData.length / 4)))`.
  let bin_count = 48.min(freq.len() / 4).max(1);
  let step = (freq.len() / bin_count).max(1);

  for b in 0..bin_count {
    let mut sum = 0usize;
    for k in 0..step {
      sum += *freq.get(b * step + k).unwrap_or(&0) as usize;
    }
    let val = (sum as f32 / (step as f32 * 255.0)) * sensitivity;
    if val < 0.04 {
      continue;
    }

    let bin_ratio = b as f32 / bin_count as f32;
    let peak_x = start_x + bin_ratio * wave_width;

    // TS: downward peaks on selected bins.
    let is_downward = b % 5 == 2 || b % 7 == 4;
    let sign = if is_downward { 1.0 } else { -1.0 };
    let peak_h = val * height * 0.32 * fire_height_scale * sign;
    let sigma = 16.0 + val * 10.0;

    for i in 0..point_count {
      let px = start_x + (i as f32 / (point_count as f32 - 1.0)) * wave_width;
      let dist = px - peak_x;
      let gauss = (-(dist * dist) / (2.0 * sigma * sigma)).exp();
      displacements[i] += peak_h * gauss;
      let d1 = dist - 12.0;
      sub1[i] += peak_h * 0.65 * (-(d1 * d1) / (2.0 * sigma * sigma)).exp();
      let d2 = dist + 15.0;
      sub2[i] += peak_h * 0.45 * (-(d2 * d2) / (2.0 * sigma * sigma)).exp();
    }

    // TS micro embers near peak tops.
    if val > 0.25 && rng.next() < 0.25 + bs * 0.3 {
      st.embers.push(Ember {
        x: peak_x + (rng.next() - 0.5) * 12.0,
        y: center_y + peak_h * 0.8 + (rng.next() - 0.5) * 10.0,
        vx: (rng.next() - 0.5) * 1.5,
        vy: (rng.next() - 0.5) * 1.5,
        size: 0.6 + rng.next() * 2.0,
        life: 0.0,
        max_life: 25.0 + rng.next() * 35.0,
      });
    }
  }

  c.save();

  // TS sets `globalCompositeOperation = 'lighter'` once (api3D.ts:99) and
  // never resets it — the restore at the end of the style is what reverts to
  // `source-over`. So EVERY pass (laser baseline, threads, needles, hero wave,
  // reflection, embers) composites additively, not just the cloud. Keep the
  // additive blend active for the whole style.
  c.set_blend_additive();
  // --- PASS 1: PLASMA SMOKE GLOW CLOUD (additive) ---
  c.set_shadow(g, 40.0 + be * 20.0);
  let cloud = Fill::linear_gradient(0.0, center_y - height * 0.2, 0.0, center_y + height * 0.2, &[
    (0.0, Color::rgba(0.0, 0.0, 0.0, 0.0)),
    (0.5, s.with_alpha(0.2)), // TS `secondaryColor + '33'` = alpha 51/255
    (1.0, Color::rgba(0.0, 0.0, 0.0, 0.0)),
  ]);
  c.set_fill(cloud);
  c.fill_rect(start_x, center_y - height * 0.25, wave_width, height * 0.5);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // --- PASS 2: STRAIGHT CENTER LASER BASELINE ---
  c.set_shadow(g, 14.0);
  c.set_stroke(Fill::Solid(p));
  c.set_global_alpha(0.75);
  c.set_line_width(1.6);
  c.stroke_line(start_x, center_y, end_x, center_y);
  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let hero_raw: Vec<(f32, f32)> = (0..point_count)
    .map(|i| {
      let px = start_x + (i as f32 / (point_count as f32 - 1.0)) * wave_width;
      (px, center_y + displacements[i])
    })
    .collect();
  let hero = quadratic_wave(&hero_raw, 5);

  // --- PASS 3: SECONDARY WOVEN SUB-THREAD WAVE LINES ---
  let thread1_raw: Vec<(f32, f32)> = (0..point_count)
    .map(|i| {
      let px = start_x + (i as f32 / (point_count as f32 - 1.0)) * wave_width;
      (px, center_y + sub1[i] + (i as f32 * 0.1 + time * 4.0).sin() * 4.0)
    })
    .collect();
  c.set_shadow(g, 10.0);
  c.set_stroke(Fill::Solid(s));
  c.set_global_alpha(0.45);
  c.set_line_width(1.2);
  c.stroke_polyline(&quadratic_wave(&thread1_raw, 5));

  let thread2_raw: Vec<(f32, f32)> = (0..point_count)
    .map(|i| {
      let px = start_x + (i as f32 / (point_count as f32 - 1.0)) * wave_width;
      (px, center_y + sub2[i] + (i as f32 * 0.12 - time * 3.5).cos() * 5.0)
    })
    .collect();
  c.set_stroke(Fill::Solid(a));
  c.set_global_alpha(0.35);
  c.set_line_width(1.0);
  c.stroke_polyline(&quadratic_wave(&thread2_raw, 5));
  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // --- PASS 4: VERTICAL LIGHT NEEDLES ON HIGH PEAKS ---
  c.set_shadow(g, 15.0);
  for i in (0..point_count).step_by(3) {
    let disp = displacements[i];
    let abs_disp = disp.abs();
    if abs_disp <= 35.0 {
      continue;
    }
    let px = start_x + (i as f32 / (point_count as f32 - 1.0)) * wave_width;
    let needle_h = abs_disp * 1.4;
    let py = center_y + disp;
    let needle = Fill::linear_gradient(0.0, py - needle_h * 0.5, 0.0, py + needle_h * 0.5, &[
      (0.0, Color::rgba(1.0, 1.0, 1.0, 0.0)),
      (0.5, a.with_alpha(0.8)),
      (1.0, Color::rgba(1.0, 1.0, 1.0, 0.0)),
    ]);
    c.set_stroke(needle);
    c.set_line_width(1.2);
    let y0 = center_y - if disp < 0.0 { needle_h } else { 10.0 };
    let y1 = center_y + if disp > 0.0 { needle_h } else { 10.0 };
    c.stroke_line(px, y0, px, y1);
  }
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // --- PASS 5: MAIN GLOWING NEON WAVEFORM (3 passes over the same path) ---
  c.set_shadow(g, 28.0 + be * 18.0);
  c.set_stroke(Fill::Solid(s));
  c.set_global_alpha(0.65);
  c.set_line_width(7.5);
  c.stroke_polyline(&hero);

  c.set_shadow(g, 16.0);
  c.set_stroke(Fill::Solid(p));
  c.set_line_width(3.6);
  c.stroke_polyline(&hero);

  c.set_shadow(a, 8.0);
  c.set_stroke(Fill::Solid(a));
  c.set_line_width(1.8);
  c.stroke_polyline(&hero);
  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // --- PASS 6: SUBTLE GLOSSY FLOOR REFLECTION ---
  let refl_raw: Vec<(f32, f32)> = (0..point_count)
    .map(|i| {
      let px = start_x + (i as f32 / (point_count as f32 - 1.0)) * wave_width;
      (px, center_y - displacements[i] * 0.45)
    })
    .collect();
  c.set_shadow(g, 12.0);
  c.set_stroke(Fill::Solid(s));
  c.set_global_alpha(0.2);
  c.set_line_width(2.2);
  c.stroke_polyline(&quadratic_wave(&refl_raw, 5));
  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // --- PASS 7: FLOATING MICRO EMBERS & DUST ---
  let mut i = st.embers.len();
  while i > 0 {
    i -= 1;
    let remove = {
      let e = &mut st.embers[i];
      e.life += 1.0;
      e.x += e.vx;
      e.y += e.vy;
      e.life / e.max_life >= 1.0
    };
    if remove {
      st.embers.remove(i);
      continue;
    }
    let (ex, ey, size, alpha) = {
      let e = &st.embers[i];
      let progress = e.life / e.max_life;
      (e.x, e.y, e.size * (1.0 - progress * 0.3), (1.0 - progress) * 0.85)
    };
    c.set_shadow(g, 6.0);
    c.set_fill(Fill::Solid(p));
    c.set_global_alpha(alpha.clamp(0.0, 1.0));
    c.fill_circle(ex, ey, size);
    c.set_global_alpha(1.0);
  }
  if st.embers.len() > 180 {
    let n = st.embers.len() - 180;
    st.embers.drain(0..n);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_global_alpha(1.0);
  c.restore();
}
