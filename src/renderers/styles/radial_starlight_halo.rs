//! Radial Starlight Halo style renderer (`radialStarlightHalo`).
//!
//! Twinkling star-cross halo:
//! - A rotating ring of four-point sparkle stars with long diffraction spikes,
//!   each star twinkling at its own phase — a crystalline star field, fully
//!   distinct from the solid teardrop spike blades.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const NUM_STARS: usize = 12;

/// Draws a 4-point sparkle star (two crossing thin diamonds) at (cx, cy).
fn draw_sparkle(c: &mut GpuCanvas, s: &radial_common::RadialSetup, cx: f32, cy: f32, size: f32, col: Color, alpha: f32) {
    let s2 = size * 0.24f32;
    // Vertical blade
    let v = vec![
        (cx, cy - size),
        (cx + s2, cy),
        (cx, cy + size),
        (cx - s2, cy),
    ];
    // Horizontal blade
    let h = vec![
        (cx - size, cy),
        (cx, cy - s2),
        (cx + size, cy),
        (cx, cy + s2),
    ];

    c.set_fill(Fill::Solid(col.with_alpha(alpha)));
    c.set_shadow(col, (8.0 + size * 0.8) * s.user_scale);
    c.fill_polygon(&v);
    c.fill_polygon(&h);

    // Central hot dot
    c.set_fill(Fill::Solid(mix(col, Color::WHITE, 0.85)));
    c.set_shadow(s.glow, (10.0 + s.bs * 8.0) * s.user_scale);
    c.fill_circle(cx, cy, (1.8 + size * 0.18) * s.user_scale);
}

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.05, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let rot = frame_time * 0.08;
    let step = ((freq.len() as f32) / NUM_STARS as f32).floor().max(1.0) as usize;

    // -------------------------------------------------------------------------
    // 1. ROTATING RING OF TWINKLING SPARKLE STARS
    // -------------------------------------------------------------------------
    for st in 0..NUM_STARS {
        let angle = (st as f32 / NUM_STARS as f32) * TAU + rot;

        let base_wave = (frame_time * 1.9 + st as f32 * 1.3).sin() * 0.5 + 0.5;
        let audio_v = radial_common::swept_bin(freq, step, st, NUM_STARS, &s) * s.sensitivity;
        let val = (base_wave + audio_v * 0.85 + s.be * 0.25
            + radial_common::beat_bump(&s, angle) * 0.5)
            .clamp(0.1, 1.4);

        let star_r = s.inner_r + (40.0 + val * 80.0 + s.bs * 15.0) * s.user_scale;
        let size = (5.0 + val * 10.0) * s.user_scale;

        let sx = s.cx + angle.cos() * star_r;
        let sy = s.cy + angle.sin() * star_r;

        let star_col = mix(s.p_col, s.glow, st as f32 / NUM_STARS as f32);
        draw_sparkle(c, &s, sx, sy, size, star_col, 0.45 + base_wave * 0.35);

        // Small orbit twin at a 180° offset for a fuller halo
        if st % 2 == 0 {
            let a2 = angle + std::f32::consts::PI;
            let sx2 = s.cx + a2.cos() * (s.inner_r + 70.0 * s.user_scale);
            let sy2 = s.cy + a2.sin() * (s.inner_r + 70.0 * s.user_scale);
            draw_sparkle(c, &s, sx2, sy2, size * 0.55, s.accent, 0.35);
        }
    }

    // -------------------------------------------------------------------------
    // 2. LONG DIFFRACTION SPIKES (slow counter-rotation, faint)
    // -------------------------------------------------------------------------
    let spikes = 4usize;
    for sp in 0..spikes {
        let a0 = (sp as f32 / spikes as f32) * TAU + frame_time * -0.03;
        let spike_len = s.inner_r + (95.0 + s.be * 25.0) * s.user_scale;

        c.set_stroke(Fill::Solid(s.glow.with_alpha(0.20)));
        c.set_line_width(1.2 * s.user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_line(
            s.cx + a0.cos() * s.inner_r,
            s.cy + a0.sin() * s.inner_r,
            s.cx + a0.cos() * spike_len,
            s.cy + a0.sin() * spike_len,
        );
    }

    radial_common::finish(c, ctx, &s);
}
