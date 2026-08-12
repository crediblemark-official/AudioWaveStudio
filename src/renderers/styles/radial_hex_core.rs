//! Radial Hex Core Reactor style renderer (`radialHexCore`).
//!
//! Hexagonal fission reactor core:
//! - Concentric hexagonal fuel-rod lattices (glowing rods on hex vertices/edges).
//! - Rotating control-rod spokes + outer hex casing, fully distinct from the
//!   tilted elliptical orbits of the Neon Orbiter.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const HEX_SIDES: usize = 6;

fn hex_points(cx: f32, cy: f32, r: f32, rot: f32) -> Vec<(f32, f32)> {
    let mut pts = Vec::with_capacity(HEX_SIDES + 1);
    for i in 0..HEX_SIDES {
        let a = (i as f32 / HEX_SIDES as f32) * TAU + rot;
        pts.push((cx + a.cos() * r, cy + a.sin() * r));
    }
    pts.push(pts[0]);
    pts
}

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 110.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let rot = frame_time * 0.05;
    let rings = 4usize;
    let step = ((freq.len() as f32) / rings as f32).floor().max(1.0) as usize;

    // -------------------------------------------------------------------------
    // 1. HEXAGONAL FUEL-ROD LATTICE (audio-reactive rods on each ring)
    // -------------------------------------------------------------------------
    for ring in 1..=rings {
        let rf = ring as f32;
        let base_wave = (frame_time * 1.2 + rf).sin() * 0.12 + 0.18;
        let audio_v = radial_common::swept_bin(freq, step, ring - 1, rings, &s) * s.sensitivity;
        let ring_angle = (rf / rings as f32) * TAU;
        let val = (base_wave + audio_v * 0.85 + s.be * 0.25 + radial_common::beat_bump(&s, ring_angle) * 0.6)
            .clamp(0.15, 1.5);

        let ring_r = s.inner_r + (18.0 + rf * 30.0 + val * 24.0) * s.user_scale;
        let rods = HEX_SIDES * ring;

        for k in 0..rods {
            let a = (k as f32 / rods as f32) * TAU + rot * (0.5 + rf * 0.25);
            let rod_r = (1.8 + val * 2.2 + rf * 0.5) * s.user_scale;

            let rod_col = mix(mix(s.p_col, s.accent, rf / rings as f32), s.glow, (val * 0.5).min(1.0));
            c.set_fill(Fill::Solid(rod_col));
            c.set_shadow(rod_col, (8.0 + val * 8.0) * s.user_scale);
            c.fill_circle(s.cx + a.cos() * ring_r, s.cy + a.sin() * ring_r, rod_r);
        }

        // Hex ring casing
        let hex_pts = hex_points(s.cx, s.cy, ring_r, rot * 0.6);
        c.set_stroke(Fill::Solid(mix(s.glow, s.p_col, rf / rings as f32).with_alpha(0.55)));
        c.set_line_width((1.6 + val * 1.2) * s.user_scale);
        c.set_shadow(s.glow, (10.0 + val * 8.0) * s.user_scale);
        c.stroke_polyline(&hex_pts);
    }

    // -------------------------------------------------------------------------
    // 2. ROTATING CONTROL-ROD SPOKES (opposite spin, 3 spokes per ring step)
    // -------------------------------------------------------------------------
    let spokes = 3usize;
    for sp in 0..spokes {
        let a0 = (sp as f32 / spokes as f32) * TAU + rot * -1.2;
        let len = (18.0 + s.be * 14.0) * s.user_scale;
        let outer = s.inner_r + (90.0 + s.bs * 20.0) * s.user_scale;
        let x0 = s.cx + a0.cos() * s.inner_r;
        let y0 = s.cy + a0.sin() * s.inner_r;
        let x1 = s.cx + a0.cos() * (outer + len);
        let y1 = s.cy + a0.sin() * (outer + len);

        c.set_stroke(Fill::Solid(s.accent.with_alpha(0.85)));
        c.set_line_width((2.6 + s.be * 1.5) * s.user_scale);
        c.set_shadow(s.accent, (14.0 + s.bs * 8.0) * s.user_scale);
        c.stroke_line(x0, y0, x1, y1);
    }

    // -------------------------------------------------------------------------
    // 3. CENTRAL HEX CORE SHELL (beats with bass)
    // -------------------------------------------------------------------------
    let core_r = s.inner_r + (4.0 + s.be * 6.0) * s.user_scale;
    let core_pts = hex_points(s.cx, s.cy, core_r, rot * 0.4);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(mix(s.glow, s.accent, 0.35)));
    c.set_line_width((3.0 + s.be * 2.0) * s.user_scale);
    c.set_shadow(s.glow, (18.0 + s.bs * 12.0) * s.user_scale);
    c.stroke_polyline(&core_pts);

    radial_common::finish(c, ctx, &s);
}
