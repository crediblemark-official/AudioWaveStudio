//! Orbit Spike style renderer (`orbitSpike`).
//!
//! A white circular ring with two smooth teardrop/crescent spikes that continuously
//! ORBIT (rotate) around the ring. Similar to a radial visualizer but with two
//! organic crescent shapes spinning instead of bars.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const RING_PTS: usize = 120;
const SPIKE_CURVE_PTS: usize = 30;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;

  // The rotation angle drives the teardrop orbit motion
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let base_r = (width.min(height) * 0.25).clamp(85.0, 290.0);
  let r = base_r + (be * 12.0);
  let stroke_w = (base_r * 0.04).clamp(3.5, 10.0);

  // -------------------------------------------------------------------------
  // 1. STATIC WHITE CIRCULAR RING (stays in place)
  // -------------------------------------------------------------------------
  let mut ring_pts = Vec::with_capacity(RING_PTS + 1);
  for i in 0..=RING_PTS {
    let angle = (i as f32 / RING_PTS as f32) * std::f32::consts::TAU;
    let bin = (i * freq.len() / RING_PTS).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let wave = fv * sensitivity * 6.0;
    let radius = r + wave;
    ring_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
  }

  let theme = &ctx.config.theme;
  let _p = crate::renderers::theme_primary(theme);

  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(stroke_w);
  c.stroke_polyline(&ring_pts);

  // -------------------------------------------------------------------------
  // 2. TWO ORBITING SMOOTH TEARDROP SPIKES (rotate around the ring!)
  // -------------------------------------------------------------------------
  // The teardrops orbit at angle `rot` and `rot + PI`, spinning continuously.
  let spike_reach = (base_r * 0.48 + be * 22.0 + bs * 10.0).clamp(35.0, 170.0);
  let angular_spread = 0.35f32; // Width of spike base on ring (radians)

  for (_idx, &pole_offset) in [0.0f32, std::f32::consts::PI].iter().enumerate() {
    // pole moves with `rot`, making the teardrops orbit
    let pole = rot + pole_offset;

    // --- TIMBUL TENGGELAM: spike size pulses with music ---
    // Sample frequency bin near this spike's current position
    let freq_pos = ((pole % std::f32::consts::TAU) / std::f32::consts::TAU * freq.len() as f32) as usize;
    let freq_bin = freq_pos.min(freq.len().saturating_sub(1));
    let fv = *freq.get(freq_bin).unwrap_or(&0) as f32 / 255.0;

    // Spike reach grows/shrinks based on local frequency + bass energy
    let dynamic_reach = spike_reach * (0.3 + fv * sensitivity * 0.9);
    // Spread also breathes slightly
    let dynamic_spread = angular_spread * (0.8 + fv * 0.4);

    let mut spike_pts: Vec<(f32, f32)> = Vec::with_capacity(SPIKE_CURVE_PTS * 2 + 3);

    // Inner edge: from (pole - spread) curving outward to tip at pole
    for i in 0..=SPIKE_CURVE_PTS {
      let t = i as f32 / SPIKE_CURVE_PTS as f32;
      let angle = pole - dynamic_spread + t * dynamic_spread;
      let lift = t * t;
      let radius = r + lift * dynamic_reach;
      spike_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
    }

    // Outer edge: from tip at pole curving back to ring at (pole + spread)
    for i in 1..=SPIKE_CURVE_PTS {
      let t = i as f32 / SPIKE_CURVE_PTS as f32;
      let angle = pole + t * dynamic_spread;
      let lift = (1.0 - t) * (1.0 - t);
      let radius = r + lift * dynamic_reach;
      spike_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
    }

    spike_pts.push(spike_pts[0]);

    // Fill the smooth teardrop spike solid white
    c.set_fill(Fill::Solid(Color::WHITE));
    let base_y = spike_pts.iter().map(|p| p.1).fold(f32::NAN, f32::max);
    c.fill_polyline_to_base(&spike_pts, base_y);

    // Stroke outline for clean edges
    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(2.0);
    c.stroke_polyline(&spike_pts);
  }

  c.restore();
}
