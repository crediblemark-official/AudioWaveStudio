//! Orbit Spike style renderer (`orbitSpike`).
//!
//! Complete redesign: A clean white vector circle with two sharp aerodynamic horn spikes
//! at 180-degree opposite poles. Each horn spike features a curved inner wing, sharp outer fin notch,
//! and tapering tip point that orbits around the circle while expanding to audio beats.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle * 0.7; // Smooth orbit speed

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let base_r = (width.min(height) * 0.25).clamp(80.0, 280.0);
  let r = base_r + (be * 12.0);
  let stroke_w = (base_r * 0.04).clamp(3.5, 10.0);

  // -------------------------------------------------------------------------
  // 1. CIRCULAR BASE VECTOR RING WITH AUDIO WAVE MODULATION
  // -------------------------------------------------------------------------
  let ring_steps = 120;
  let mut ring_pts = Vec::with_capacity(ring_steps + 1);
  for i in 0..=ring_steps {
    let angle = (i as f32 / ring_steps as f32) * std::f32::consts::TAU;
    let bin = (i * freq.len() / ring_steps).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let wave = fv * sensitivity * 6.0;
    let radius = r + wave;
    ring_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
  }

  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(stroke_w);
  c.stroke_polyline(&ring_pts);

  // -------------------------------------------------------------------------
  // 2. TWO AERODYNAMIC CURVED HORN SPIKES (TWIN 180-DEGREE OPPOSING)
  // -------------------------------------------------------------------------
  let spike_reach = (base_r * 0.55 + be * 28.0 + bs * 12.0).clamp(45.0, 190.0);

  for &pole_offset in &[0.0f32, std::f32::consts::PI] {
    let pole = rot + pole_offset;
    let tang = pole + std::f32::consts::FRAC_PI_2;

    // Contact center on ring
    let cx = center_x + pole.cos() * r;
    let cy = center_y + pole.sin() * r;

    // 1. Base In Point (on ring, counter-clockwise)
    let in_angle = pole - 0.25;
    let base_in = (center_x + in_angle.cos() * r, center_y + in_angle.sin() * r);

    // 2. Base Out Point (on ring, clockwise)
    let out_angle = pole + 0.25;
    let base_out = (center_x + out_angle.cos() * r, center_y + out_angle.sin() * r);

    // 3. Sharp Tip Point (extended outward and forward)
    let tip_x = cx + (pole.cos() * 0.75 + tang.cos() * 0.65) * spike_reach;
    let tip_y = cy + (pole.sin() * 0.75 + tang.sin() * 0.65) * spike_reach;

    // 4. Outer Fin Notch Point (sharp corner on outer edge)
    let fin_len = spike_reach * 0.55;
    let fin_x = cx + (pole.cos() * 0.90 - tang.cos() * 0.35) * fin_len;
    let fin_y = cy + (pole.sin() * 0.90 - tang.sin() * 0.35) * fin_len;

    // Construct smooth curved horn contour: BaseIn -> Tip -> Fin -> BaseOut -> BaseIn
    let mut horn_pts = Vec::with_capacity(30);

    // Inner edge curve (BaseIn to Tip)
    for i in 0..=12 {
      let t = i as f32 / 12.0;
      let px = base_in.0 + t * (tip_x - base_in.0) + (1.0 - t) * t * (pole.cos() * 15.0);
      let py = base_in.1 + t * (tip_y - base_in.1) + (1.0 - t) * t * (pole.sin() * 15.0);
      horn_pts.push((px, py));
    }

    // Outer edge tip to fin
    for i in 1..=8 {
      let t = i as f32 / 8.0;
      let px = tip_x + t * (fin_x - tip_x);
      let py = tip_y + t * (fin_y - tip_y);
      horn_pts.push((px, py));
    }

    // Fin back to BaseOut
    for i in 1..=8 {
      let t = i as f32 / 8.0;
      let px = fin_x + t * (base_out.0 - fin_x);
      let py = fin_y + t * (base_out.1 - fin_y);
      horn_pts.push((px, py));
    }

    horn_pts.push(base_in);

    // Solid white vector horn fill
    c.set_fill(Fill::Solid(Color::WHITE));
    let base_y = horn_pts.iter().map(|p| p.1).fold(f32::NAN, f32::max);
    c.fill_polyline_to_base(&horn_pts, base_y);

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(2.0);
    c.stroke_polyline(&horn_pts);
  }

  c.restore();
}
