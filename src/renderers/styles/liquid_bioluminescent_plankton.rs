//! Liquid Spiral Galactic Stream style renderer (`liquidBioluminescentPlankton`).
//!
//! Visual Concept:
//! - Double-Helix Genetic DNA Liquid Ribbon weaving in 3D perspective depth space.
//! - Intertwined 3D double helix strands rotating & expanding with audio spectrum energy.
//! - Nucleotide genetic code bridges & floating luminous bio-motes.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const HELIX_STEPS: usize = 36;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let user_scale = ctx.config.scale.clamp(0.1, 5.0);
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
    let inner_r = base_r * (0.35 + be * 0.08);

    // Palette: Double-Helix DNA Stream
    let dna_cyan = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.80);
    let dna_magenta = mix(accent_col, Color::rgba(1.0, 0.15, 0.75, 1.0), 0.80);
    let spark_white = Color::WHITE;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Ambient DNA Glow Field
    let bg_dna = Fill::radial_gradient(
        cx, cy, inner_r,
        cx, cy, base_r * 2.5,
        &[
            (0.0, mix(dna_cyan, dna_magenta, 0.5).with_alpha(0.35 + be * 0.20)),
            (0.55, s_col.with_alpha(0.15)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_dna);
    c.fill_rect(cx - base_r * 2.5, cy - base_r * 2.5, base_r * 5.0, base_r * 5.0);

    // 2. Double Helix 3D Intertwined Strands
    let step = (freq.len() / HELIX_STEPS).max(1);
    let rot = frame_time * 0.35;

    let mut strand1_pts: Vec<(f32, f32)> = Vec::with_capacity(HELIX_STEPS);
    let mut strand2_pts: Vec<(f32, f32)> = Vec::with_capacity(HELIX_STEPS);

    for i in 0..HELIX_STEPS {
        let ht = i as f32 / HELIX_STEPS as f32;
        let angle = ht * TAU * 3.0 + rot;

        let fv = super::radial_common::full_random_scattered_bin(freq, step, i, HELIX_STEPS, ctx.beat_count);
        let val = (fv * sensitivity * 1.1 + be * 0.30).clamp(0.08, 2.2);

        let h_radius = inner_r + ht * (base_r * 1.3 - inner_r) + (val * 25.0) * user_scale;
        let (sin_a, cos_a) = angle.sin_cos();

        // 3D Perspective Sine Offset
        let off_x = cos_a * (35.0 + val * 15.0) * user_scale;
        let off_y = sin_a * (35.0 + val * 15.0) * user_scale;

        let (rx, ry) = (cx + (angle * 0.5).cos() * h_radius, cy + (angle * 0.5).sin() * h_radius);

        let p1 = (rx + off_x, ry + off_y);
        let p2 = (rx - off_x, ry - off_y);

        strand1_pts.push(p1);
        strand2_pts.push(p2);

        // Nucleotide Base Pair Connecting Bridge
        if i % 3 == 0 {
            c.set_stroke(Fill::Solid(mix(dna_cyan, dna_magenta, ht).with_alpha(0.70)));
            c.set_line_width((1.8 + val * 1.2) * user_scale);
            c.set_shadow(dna_cyan, 8.0 * user_scale);
            c.stroke_line(p1.0, p1.1, p2.0, p2.1);
        }
    }

    // Render Helix Strand 1 (Cyan)
    c.set_stroke(Fill::Solid(dna_cyan));
    c.set_line_width((3.0 + bs * 1.5) * user_scale);
    c.set_shadow(dna_cyan, (14.0 + bs * 10.0) * user_scale);
    c.stroke_polyline(&strand1_pts);

    // Render Helix Strand 2 (Magenta)
    c.set_stroke(Fill::Solid(dna_magenta));
    c.set_line_width((3.0 + bs * 1.5) * user_scale);
    c.set_shadow(dna_magenta, (14.0 + bs * 10.0) * user_scale);
    c.stroke_polyline(&strand2_pts);

    // Core White Filaments
    c.set_stroke(Fill::Solid(spark_white));
    c.set_line_width(1.2 * user_scale);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.stroke_polyline(&strand1_pts);
    c.stroke_polyline(&strand2_pts);

    // 3. Central Core Disc
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(dna_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(dna_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    let _ = p_col;
    c.set_global_alpha(1.0);
    c.restore();
}
