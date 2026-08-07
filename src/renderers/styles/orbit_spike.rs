//! Orbit Spike style renderer (`orbitSpike`).
//!
//! Full Structural Vector Architecture Rebuild:
//! Recreates the exact aerodynamic emblem from reference image:
//! - Two tapering crescent arcs (thick near horn bases, thin at midpoints)
//! - Two solid white vector claw horns at 180-degree opposite poles, featuring:
//!   - Curved inner wing (bezier curve from arc start)
//!   - Extended sharp vector tip
//!   - Outer aerodynamic fin notch
//!   - Smooth return curve to arc end

use std::f32::consts::PI;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const ARC_PTS: usize = 40;
const HORN_PTS: usize = 20;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle * 0.7;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let base_r = (width.min(height) * 0.25).clamp(80.0, 280.0);
  let r = base_r + (be * 10.0);

  // -------------------------------------------------------------------------
  // 1. TWO TAPERING CRESCENT ARCS (Thick at ends near horns, thin in middle)
  // -------------------------------------------------------------------------
  // Arc 1: rot + 0.20 to rot + PI - 0.20
  // Arc 2: rot + PI + 0.20 to rot + 2*PI - 0.20
  for &half_rot in &[rot, rot + PI] {
    let start_a = half_rot + 0.22;
    let end_a = half_rot + PI - 0.22;

    let mut arc_pts = Vec::with_capacity(ARC_PTS + 1);
    for i in 0..=ARC_PTS {
      let t = i as f32 / ARC_PTS as f32;
      let angle = start_a + t * (end_a - start_a);

      let bin = (i * freq.len() / ARC_PTS).min(freq.len().saturating_sub(1));
      let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

      let radius = r + fv * sensitivity * 6.0;
      arc_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
    }

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width((base_r * 0.045).clamp(4.0, 11.0));
    c.stroke_polyline(&arc_pts);
  }

  // -------------------------------------------------------------------------
  // 2. TWO AERODYNAMIC VECTOR CLAW HORNS (At rot & rot + PI)
  // -------------------------------------------------------------------------
  let spike_len = (base_r * 0.55 + be * 24.0 + bs * 12.0).clamp(45.0, 185.0);
  let fin_len = spike_len * 0.50;

  for &pole in &[rot, rot + PI] {
    let tang = pole + std::f32::consts::FRAC_PI_2;

    // Center origin on ring
    let cx = center_x + pole.cos() * r;
    let cy = center_y + pole.sin() * r;

    // Contact points on tapering crescent arc tips
    let p_in_a = pole - 0.22;
    let p_out_a = pole + 0.22;
    let base_in = (center_x + p_in_a.cos() * r, center_y + p_in_a.sin() * r);
    let base_out = (center_x + p_out_a.cos() * r, center_y + p_out_a.sin() * r);

    // Extended sharp vector tip
    let tip_x = cx + (pole.cos() * 0.75 + tang.cos() * 0.70) * spike_len;
    let tip_y = cy + (pole.sin() * 0.75 + tang.sin() * 0.70) * spike_len;

    // Outer aerodynamic fin notch
    let fin_x = cx + (pole.cos() * 0.88 - tang.cos() * 0.32) * fin_len;
    let fin_y = cy + (pole.sin() * 0.88 - tang.sin() * 0.32) * fin_len;

    // Polygon path: base_in -> inner curve -> tip -> outer edge -> fin -> base_out -> base_in
    let mut claw_poly = Vec::with_capacity(HORN_PTS * 2 + 4);

    // Inner wing curve (base_in to tip)
    for i in 0..=HORN_PTS {
      let t = i as f32 / HORN_PTS as f32;
      let px = base_in.0 + t * (tip_x - base_in.0) + (1.0 - t) * t * (pole.cos() * 12.0);
      let py = base_in.1 + t * (tip_y - base_in.1) + (1.0 - t) * t * (pole.sin() * 12.0);
      claw_poly.push((px, py));
    }

    // Outer edge (tip to fin)
    claw_poly.push((fin_x, fin_y));

    // Return edge (fin to base_out)
    claw_poly.push(base_out);
    claw_poly.push(base_in);

    // Render solid white vector polygon claw
    c.set_fill(Fill::Solid(Color::WHITE));
    let base_y = claw_poly.iter().map(|p| p.1).fold(f32::NAN, f32::max);
    c.fill_polyline_to_base(&claw_poly, base_y);

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(2.0);
    c.stroke_polyline(&claw_poly);
  }

  c.restore();
}
