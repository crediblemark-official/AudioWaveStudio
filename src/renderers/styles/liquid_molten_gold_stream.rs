//! Liquid Gold Ingot Fountain style renderer (`liquidMoltenGoldStream`).
//!
//! Visual Concept:
//! - 3D Liquid Gold Geyser Fountain shooting 360° parabolic fluid arcs outward from the central logo disc.
//! - 24 Smooth parabolic liquid gold geyser jets emerging directly from `inner_r` with 24K gold dust motes.
//! - Photorealistic metallic gold specular sheen highlights & warm amber glow.
//! - ZERO AMPLITUDE STEALTH HIDING: When music is quiet, geysers retract to `inner_r`.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const FOUNTAIN_JETS: usize = 24;
const DUST_MOTES:    usize = 45;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col      = theme_primary(theme);
    let s_col      = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col   = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

    let be = ctx.bass_energy.clamp(0.0, 3.0);
    let bs = ctx.beat_strength.clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08);

    // Palette: 24K Molten Gold Fountain blended with theme
    let gold_24k    = mix(accent_col, Color::rgba(1.0, 0.84, 0.0, 1.0), 0.50);
    let gold_molten = mix(p_col,      Color::rgba(1.0, 0.65, 0.0, 1.0), 0.40);
    let gold_amber  = mix(s_col,      Color::rgba(0.85, 0.45, 0.0, 1.0), 0.35);
    let spark_white = mix(glow_col,   Color::WHITE, 0.85);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Ambient Gold Backdrop Glow — rendered to canvas
    let bg_gold = Fill::radial_gradient(
        cx, cy, inner_r,
        cx, cy, base_r * 2.8,
        &[
            (0.0,  mix(gold_24k, gold_molten, 0.5).with_alpha(0.38 + be * 0.20)),
            (0.50, gold_amber.with_alpha(0.14)),
            (1.0,  Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_gold);
    c.fill_rect(cx - base_r * 2.8, cy - base_r * 2.8, base_r * 5.6, base_r * 5.6);

    // 2. 24 Parabolic Liquid Gold Geyser Jets fanning 360° from inner_r
    let step = (freq.len() / FOUNTAIN_JETS).max(1);
    let rot  = frame_time * 0.08;

    for i in 0..FOUNTAIN_JETS {
        let jf     = i as f32;
        let angle0 = (jf / FOUNTAIN_JETS as f32) * TAU + rot;
        let angle1 = ((jf + 0.85) / FOUNTAIN_JETS as f32) * TAU + rot;
        let mid_a  = (angle0 + angle1) * 0.5;

        // Smooth spatial frequency sampling across adjacent jets for fluid wave motion
        let fv  = super::radial_common::smooth_ring_bin(freq, step, i, FOUNTAIN_JETS);
        let val = (fv * sensitivity * 1.1 + be * 0.30 + bs * 0.20).clamp(0.0, 2.2);

        // Geyser height extends radially outward from inner_r
        let jet_h = (val * 110.0) * user_scale;
        let apex_r = inner_r + jet_h;

        let (cos0, sin0) = angle0.sin_cos();
        let (cos1, sin1) = angle1.sin_cos();
        let (cosm, sinm) = mid_a.sin_cos();

        // Curve start at inner_r, apex at mid_a, end back at inner_r
        let p0    = (cx + cos0 * inner_r, cy + sin0 * inner_r);
        let p_mid = (cx + cosm * apex_r, cy + sinm * apex_r);
        let p_end = (cx + cos1 * inner_r, cy + sin1 * inner_r);

        // High-resolution silky smooth quadratic curve (24 points per arc)
        let jet_pts = GpuCanvas::sample_quadratic(p0, p_mid, p_end, 24);

        // Gold Stream Outer Sheath
        let jet_alpha = (0.55 + val * 0.35).clamp(0.10, 0.90);
        c.set_stroke(Fill::Solid(gold_molten.with_alpha(jet_alpha)));
        c.set_line_width((3.5 + val * 2.5) * user_scale);
        c.set_shadow(gold_24k, (14.0 + val * 10.0) * user_scale);
        c.stroke_polyline(&jet_pts);

        // Core White Specular Filament
        c.set_stroke(Fill::Solid(spark_white.with_alpha(jet_alpha)));
        c.set_line_width((1.2 + val * 0.6) * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_polyline(&jet_pts);

        // Apex Splatter Droplet Node (glowing gold drop at jet peak)
        if val > 0.1 {
            c.set_fill(Fill::Solid(spark_white));
            c.set_shadow(gold_24k, (14.0 + val * 8.0) * user_scale);
            c.fill_circle(p_mid.0, p_mid.1, (3.0 + val * 2.5) * user_scale);
        }
    }

    // 3. 24K Gold Dust Motes floating radially outward from inner_r in 360°
    let dust_step = (freq.len() / DUST_MOTES).max(1);
    for d in 0..DUST_MOTES {
        let df    = d as f32;
        let da    = (df / DUST_MOTES as f32) * TAU + frame_time * 0.05;
        let phase = ((frame_time * (0.25 + df * 0.01) + df * 0.413) % 1.0).clamp(0.0, 1.0);

        let fv  = super::radial_common::smooth_ring_bin(freq, dust_step, d % DUST_MOTES, DUST_MOTES);
        let val = (fv * sensitivity * 0.9 + be * 0.20).clamp(0.0, 1.8);

        // Motes float outward from inner_r to inner_r + base_r * 1.5
        let dist  = inner_r + phase * (base_r * 1.35 + val * 40.0 * user_scale);
        let (cos_d, sin_d) = da.sin_cos();
        let dx = cx + cos_d * dist;
        let dy = cy + sin_d * dist;

        let alpha = (1.0 - phase).powi(2) * (0.65 + val * 0.30);

        if alpha > 0.04 {
            let d_col = mix(gold_24k, spark_white, phase * 0.6);
            let d_sz  = (1.5 + (1.0 - phase) * 2.5 + val * 2.0) * user_scale;

            c.set_fill(Fill::Solid(d_col.with_alpha(alpha)));
            c.set_shadow(gold_molten, (5.0 + val * 5.0) * user_scale);
            c.fill_circle(dx, dy, d_sz);
        }
    }

    // 4. Central Core Disc
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(gold_24k));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(gold_24k, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
