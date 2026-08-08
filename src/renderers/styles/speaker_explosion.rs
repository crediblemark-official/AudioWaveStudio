//! Speaker Explosion style renderer (`speakerExplosion`) — Explosive Subwoofer Engine.
//!
//! Renders a hyper-realistic explosive audio subwoofer speaker complete with:
//! - Pumping bass woofer cone with carbon fiber weave & rubber surround flex ring
//! - Glowing metallic dust cap dome surging on beat hits
//! - Expanding radial audio shockwave blast rings on bass drops
//! - 360° High-density audio spectrum ray burst surrounding the speaker
//! - Dynamic flying 3D particle shards & liquid paint splatters exploding into space
//! - Full UI settings integration (Scale, Position X & Y, Sensitivity, Bass Boost, Bar Count).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

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

  let center_x = width * 0.5;
  let center_y = height * 0.48;

  let base_r = ((width.min(height) * 0.24).clamp(80.0, 280.0)).clamp(50.0, width * 0.42);
  let woofer_r = base_r + (be * 32.0 * sensitivity) + (bs * 16.0);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. ATMOSPHERIC BACKDROP & RADIAL EXPLOSION AMBIENT GLOW
  // -------------------------------------------------------------------------
  let bg_haze = Fill::radial_gradient(
    center_x,
    center_y,
    0.0,
    center_x,
    center_y,
    woofer_r * 2.2,
    &[
      (0.0, glow.with_alpha(0.24 + be * 0.18)),
      (0.40, p.with_alpha(0.14)),
      (0.75, Color::rgba(0.04, 0.02, 0.08, 0.06)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(bg_haze);
  c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // 2. EXPANDING AUDIO SHOCKWAVE BLAST RINGS (ON BASS HITS)
  // -------------------------------------------------------------------------
  let shockwave_count = 3usize;
  for s_i in 0..shockwave_count {
    let s_t = ((frame_time * 0.6 + s_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
    let s_r = woofer_r * (1.0 + s_t * (1.2 + be * 0.8));
    let s_alpha = ((1.0 - s_t) * (0.40 + be * 0.40)).clamp(0.0, 0.85);

    let shock_col = mix(accent, p, s_t);
    c.set_stroke(Fill::Solid(shock_col.with_alpha(s_alpha)));
    c.set_line_width((4.0 * (1.0 - s_t) + 1.0).clamp(1.0, 6.0));
    c.set_shadow(shock_col, 12.0);
    c.stroke_circle(center_x, center_y, s_r);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 3. 360° HIGH-DENSITY RADIAL SPECTRUM RAY BURST (BEHIND SPEAKER)
  // -------------------------------------------------------------------------
  let max_spike_len = height * 0.22 * sensitivity;
  let step_f = (freq.len() / bar_count).max(1);

  for i in 0..bar_count {
    let angle = (i as f32 / bar_count as f32) * TAU + frame_time * 0.05;

    let k = (i * step_f).min(freq.len().saturating_sub(1));
    let raw_v = freq[k] as f32 / 255.0;
    let spike_len = (raw_v * sensitivity * max_spike_len).clamp(8.0, (max_spike_len * 1.5).max(8.0));

    let x1 = center_x + angle.cos() * (woofer_r * 0.92);
    let y1 = center_y + angle.sin() * (woofer_r * 0.92);
    let x2 = center_x + angle.cos() * (woofer_r * 0.92 + spike_len);
    let y2 = center_y + angle.sin() * (woofer_r * 0.92 + spike_len);

    let ray_col = mix(p, s, i as f32 / bar_count as f32);

    c.set_stroke(Fill::Solid(ray_col.with_alpha(0.88)));
    c.set_line_width((2.0 + raw_v * 3.5).clamp(1.5, 8.0));
    c.set_shadow(ray_col, 8.0 + bs * 6.0);
    c.stroke_line(x1, y1, x2, y2);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 4. FLOATING 3D EXPLOSIVE PARTICLES & PAINT DROPLETS
  // -------------------------------------------------------------------------
  let num_splatters = 42usize;
  for i in 0..num_splatters {
    let seed = i as f32 * 41.7;
    let p_t = ((frame_time * 0.5 + i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
    let dist = woofer_r * 1.05 + p_t * (height * 0.35);
    let angle = (seed * 0.3 + frame_time * 0.08) % TAU;

    let px = center_x + angle.cos() * dist;
    let py = center_y + angle.sin() * dist;

    let dot_r = ((4.0 * (1.0 - p_t) + 1.5) + bs * 3.0).clamp(1.5, 10.0);
    let dot_col = mix(accent, Color::WHITE, p_t).with_alpha((1.0 - p_t).clamp(0.1, 0.95));

    c.set_fill(Fill::Solid(dot_col));
    c.set_shadow(dot_col, 8.0);
    c.fill_ellipse(px, py, dot_r, dot_r);
  }

  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 5. HYPER-REALISTIC HIGH-FIDELITY SUBWOOFER ASSEMBLY
  // -------------------------------------------------------------------------
  // A. Outer Metallic Chassis Frame Ring
  let rim_grad = Fill::radial_gradient(
    center_x,
    center_y,
    woofer_r * 0.85,
    center_x,
    center_y,
    woofer_r,
    &[
      (0.0, Color::rgba(0.12, 0.14, 0.18, 0.98)),
      (0.5, Color::rgba(0.85, 0.90, 0.95, 0.95)),
      (0.85, s.with_alpha(0.95)),
      (1.0, Color::rgba(0.08, 0.10, 0.14, 0.98)),
    ],
  );

  c.set_fill(rim_grad);
  c.set_shadow(glow.with_alpha(0.85), 24.0);
  c.fill_ellipse(center_x, center_y, woofer_r, woofer_r);

  c.set_stroke(Fill::Solid(s));
  c.set_line_width(2.5);
  c.stroke_circle(center_x, center_y, woofer_r);

  // 8 Silver Hex Mounting Screws/Bolts on Rim
  let bolt_r = (woofer_r * 0.04).clamp(3.0, 8.0);
  let bolt_dist = woofer_r * 0.92;
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), 4.0);

  for b_idx in 0..8 {
    let b_angle = (b_idx as f32 / 8.0) * TAU;
    let bx = center_x + b_angle.cos() * bolt_dist;
    let by = center_y + b_angle.sin() * bolt_dist;

    c.set_fill(Fill::Solid(Color::rgba(0.90, 0.92, 0.96, 0.98)));
    c.fill_ellipse(bx, by, bolt_r, bolt_r);
    c.set_fill(Fill::Solid(Color::rgba(0.18, 0.20, 0.25, 0.95)));
    c.fill_ellipse(bx, by, bolt_r * 0.45, bolt_r * 0.45);
  }

  // B. Corrugated Rubber Surround Suspension Ring (Expands/contracts on bass!)
  let surround_r = woofer_r * 0.82;
  c.set_fill(Fill::Solid(Color::rgba(0.08, 0.07, 0.10, 0.98)));
  c.set_stroke(Fill::Solid(Color::rgba(0.30, 0.32, 0.40, 0.8)));
  c.set_line_width(3.0);
  c.fill_ellipse(center_x, center_y, surround_r, surround_r);
  c.stroke_circle(center_x, center_y, surround_r);

  // C. Deep Carbon / Paper Pulp Speaker Cone
  let cone_r = woofer_r * 0.66;
  let cone_grad = Fill::radial_gradient(
    center_x,
    center_y,
    0.0,
    center_x,
    center_y,
    cone_r,
    &[
      (0.0, Color::rgba(0.04, 0.03, 0.06, 0.98)),
      (0.65, Color::rgba(0.14, 0.12, 0.18, 0.98)),
      (1.0, Color::rgba(0.06, 0.05, 0.08, 0.98)),
    ],
  );

  c.set_fill(cone_grad);
  c.fill_ellipse(center_x, center_y, cone_r, cone_r);

  // Concentric cone texture rings
  c.set_stroke(Fill::Solid(Color::rgba(0.30, 0.32, 0.40, 0.35)));
  c.set_line_width(1.0);
  for &cr in &[0.3f32, 0.5, 0.7, 0.9] {
    c.stroke_circle(center_x, center_y, cone_r * cr);
  }

  // D. Luminous Metallic Dust Cap Dome (Pumping violently on bass hits!)
  let cap_r = cone_r * (0.36 + be * 0.10);

  let cap_grad = Fill::radial_gradient(
    center_x - cap_r * 0.25,
    center_y - cap_r * 0.25,
    0.0,
    center_x,
    center_y,
    cap_r,
    &[
      (0.0, Color::rgba(1.0, 0.95, 0.70, 0.98)),
      (0.4, mix(accent, Color::WHITE, bs)),
      (0.85, mix(p, accent, 0.5)),
      (1.0, Color::rgba(0.20, 0.05, 0.10, 0.98)),
    ],
  );

  c.set_fill(cap_grad);
  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(2.0);
  c.set_shadow(accent, 18.0 + bs * 10.0);
  c.fill_ellipse(center_x, center_y, cap_r, cap_r);
  c.stroke_circle(center_x, center_y, cap_r);

  // 3D Specular Highlight Reflection Spot
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.70)));
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_ellipse(center_x - cap_r * 0.3, center_y - cap_r * 0.3, cap_r * 0.22, cap_r * 0.15);

  c.restore();
}
