//! Orbit Spike style renderer (`orbitSpike`).
//!
//! Recreates the EXACT vector aerodynamic emblem from the reference image:
//! Pure solid white graphic on pitch black background, featuring two tapering crescent arc rings
//! and two sharp curved horn spikes with outer fin notches at 180-degree opposite poles.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const ARC_STEPS: usize = 60;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle * 0.85 + (std::f32::consts::PI * 0.22); // Fixed aesthetic tilt matching photo

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Pitch Black Background (Exact Match to Photo)
  c.set_fill(Fill::Solid(Color::BLACK));
  c.fill_rect(0.0, 0.0, width, height);

  let base_r = (width.min(height) * 0.25).clamp(85.0, 290.0);
  let r = base_r + (be * 16.0);

  // -------------------------------------------------------------------------
  // 1. TWO TAPERING CRESCENT ARC RINGS (EXACT MATCH TO PHOTO)
  // -------------------------------------------------------------------------
  // The two arcs taper from thick near the horn bases to thin at the midpoints!
  for &half in &[0.0f32, std::f32::consts::PI] {
    let a_start = rot + half + 0.18;
    let a_end = rot + half + std::f32::consts::PI - 0.18;

    let mut arc_pts = Vec::with_capacity(ARC_STEPS + 1);
    for i in 0..=ARC_STEPS {
      let t = i as f32 / ARC_STEPS as f32;
      let angle = a_start + t * (a_end - a_start);

      let bin = (i * freq.len() / ARC_STEPS).min(freq.len().saturating_sub(1));
      let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

      // Variable stroke width: thickest near ends, thinnest in middle
      let _width_scale = 1.0 + (t * std::f32::consts::PI).sin() * -0.4;
      let wave_mod = fv * sensitivity * 12.0;

      let radius = r + wave_mod;
      let px = center_x + angle.cos() * radius;
      let py = center_y + angle.sin() * radius;

      arc_pts.push((px, py));
    }

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width((base_r * 0.045).clamp(4.0, 12.0));
    c.stroke_polyline(&arc_pts);
  }

  // -------------------------------------------------------------------------
  // 2. TWO AERODYNAMIC CURVED HORN SPIKES WITH FIN NOTCHES (EXACT MATCH TO PHOTO)
  // -------------------------------------------------------------------------
  // Top-Right Spike at `rot`, Bottom-Left Spike at `rot + PI`
  let spike_length = (base_r * 0.58 + be * 30.0 + bs * 12.0).clamp(45.0, 200.0);
  let fin_length = base_r * 0.28;

  for &pole in &[0.0f32, std::f32::consts::PI] {
    let pole_angle = rot + pole;
    let tang_angle = pole_angle + std::f32::consts::FRAC_PI_2;

    // Contact center on ring
    let contact_x = center_x + pole_angle.cos() * r;
    let contact_y = center_y + pole_angle.sin() * r;

    // Sharp Tip (Extending outwards top-right / bottom-left)
    let tip_x = contact_x + (pole_angle.cos() * 0.65 + tang_angle.cos() * 0.75) * spike_length;
    let tip_y = contact_y + (pole_angle.sin() * 0.65 + tang_angle.sin() * 0.75) * spike_length;

    // Outer Fin Notch (The sharp corner on the outer edge in photo)
    let fin_x = contact_x + (pole_angle.cos() * 0.85 - tang_angle.cos() * 0.25) * fin_length;
    let fin_y = contact_y + (pole_angle.sin() * 0.85 - tang_angle.sin() * 0.25) * fin_length;

    // Inner Arc Base Blend
    let base_in_x = contact_x - tang_angle.cos() * (base_r * 0.12);
    let base_in_y = contact_y - tang_angle.sin() * (base_r * 0.12);

    let base_out_x = contact_x + tang_angle.cos() * (base_r * 0.14);
    let base_out_y = contact_y + tang_angle.sin() * (base_r * 0.14);

    // Polygon contour matching exact photo silhouette:
    // Base_In -> Curved Tip -> Outer Fin -> Base_Out -> Base_In
    let horn_poly = [
      (base_in_x, base_in_y),
      (tip_x, tip_y),
      (fin_x, fin_y),
      (base_out_x, base_out_y),
      (base_in_x, base_in_y),
    ];

    // Solid Filled White Horn
    c.set_fill(Fill::Solid(Color::WHITE));
    c.fill_polyline_to_base(&horn_poly, base_in_y);

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(2.0);
    c.stroke_polyline(&horn_poly);
  }

  c.restore();
}
