//! Liquid Crystal Sound Lattice style renderer (`liquidFerrofluidSpikes`).
//!
//! Visual Concept:
//! - Cymatics Chladni Standing Wave Pattern: Audio-frequency vibrating metal plate
//!   revealing sacred geometry sand node patterns.
//! - 64-node Chladni resonance nodal grid: sand grains accumulate at quiet zones
//!   forming geometric patterns that transform with each frequency band.
//! - Resonance rings at 3 concentric radii with bright sand mote accumulation at
//!   node crossings and dark anti-node emptiness zones in between.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const CHLADNI_NODES: usize = 64;
const RADII_RINGS: usize = 4;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col  = theme_primary(theme);
    let s_col  = theme_secondary(theme);
    let accent = theme_accent(theme);
    let glow   = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

    let be = ctx.bass_energy.clamp(0.0, 3.0);
    let bs = ctx.beat_strength.clamp(0.0, 3.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 - pos_offset_y;

    let reference_size = width.min(height);
    let base_r = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.06);

    // Palette: Chladni Sand Pattern (amber/ochre sand on black plate)
    let sand_warm   = Color::rgba(0.90, 0.75, 0.30, 1.0);  // Sandy gold
    let sand_bright = Color::rgba(1.0, 0.95, 0.60, 1.0);   // Bright node accumulation
    let ring_col    = mix(glow, Color::rgba(0.80, 0.65, 0.20, 1.0), 0.70);
    let spark_white = Color::WHITE;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Ambient Cymatics Plate Glow
    let bg = Fill::radial_gradient(
        cx, cy, inner_r,
        cx, cy, base_r * 2.2,
        &[
            (0.0, mix(sand_warm, accent, 0.40).with_alpha(0.35 + be * 0.18)),
            (0.55, s_col.with_alpha(0.10)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg);
    c.fill_rect(0.0, 0.0, width, height);

    // 2. Resonance Rings (concentric Chladni nodal circles)
    let step = (freq.len() / CHLADNI_NODES).max(1);

    for r_idx in 0..RADII_RINGS {
        let rf = r_idx as f32;
        let ring_r = inner_r + (rf + 1.0) * ((base_r - inner_r) / (RADII_RINGS + 1) as f32);

        let fv_band = if freq.is_empty() { 0.0 } else {
            let band_idx = ((r_idx * freq.len()) / RADII_RINGS).min(freq.len() - 1);
            (freq[band_idx] as f32 / 255.0) * sensitivity
        };
        let ring_alpha = (0.40 + fv_band * 0.45 + bs * 0.20).clamp(0.20, 0.90);

        c.set_stroke(Fill::Solid(ring_col.with_alpha(ring_alpha)));
        c.set_line_width((1.5 + fv_band * 1.5) * user_scale);
        c.set_shadow(sand_bright, (8.0 + fv_band * 8.0) * user_scale);
        c.stroke_circle(cx, cy, ring_r);
    }

    // 3. 64 Chladni Sand Node Accumulation Motes
    for n_i in 0..CHLADNI_NODES {
        let nt = n_i as f32 / CHLADNI_NODES as f32;
        let angle = nt * TAU + frame_time * 0.04;

        let fv = if freq.is_empty() { 0.0 } else {
            let idx = (n_i * step).min(freq.len() - 1);
            (freq[idx] as f32 / 255.0) * sensitivity
        };
        let val = (fv + be * 0.25 + bs * 0.15).clamp(0.0, 2.0);

        // Chladni node radius: quiet nodes at specific harmonic radii
        let harmonic = (n_i % RADII_RINGS) as f32 + 1.0;
        let ring_r = inner_r + harmonic * ((base_r - inner_r) / (RADII_RINGS + 1) as f32);
        let wobble = (angle * 5.0 + frame_time * 1.5).sin() * (4.0 * user_scale);

        let (sin_a, cos_a) = angle.sin_cos();
        let nx = cx + cos_a * (ring_r + wobble);
        let ny = cy + sin_a * (ring_r + wobble);

        // Sand grain size proportional to node energy
        let grain_sz = (2.0 + val * 5.0) * user_scale;
        let grain_col = mix(sand_warm, spark_white, val * 0.5).with_alpha(0.55 + val * 0.35);

        c.set_fill(Fill::Solid(grain_col));
        c.set_shadow(sand_bright, (6.0 + val * 8.0) * user_scale);
        c.fill_circle(nx, ny, grain_sz);
    }

    // 4. Geometric Chladni Pattern Lines (radial spokes at harmonic angles)
    let spoke_count = 8usize;
    for sk in 0..spoke_count {
        let spoke_angle = (sk as f32 / spoke_count as f32) * TAU + frame_time * 0.02;
        let (sin_a, cos_a) = spoke_angle.sin_cos();
        let spoke_val = if freq.is_empty() { 0.0 } else {
            let idx = ((sk * freq.len()) / spoke_count).min(freq.len() - 1);
            (freq[idx] as f32 / 255.0) * sensitivity * 0.8
        };

        c.set_stroke(Fill::Solid(sand_warm.with_alpha(0.30 + spoke_val * 0.40)));
        c.set_line_width((0.8 + spoke_val * 1.2) * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_line(
            cx + cos_a * inner_r,
            cy + sin_a * inner_r,
            cx + cos_a * base_r,
            cy + sin_a * base_r,
        );
    }

    // 5. Central Core Disc
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(sand_bright));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(sand_bright, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    let _ = p_col;
    c.set_global_alpha(1.0);
    c.restore();
}
