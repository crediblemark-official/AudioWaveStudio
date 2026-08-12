//! Liquid Quantum Ferro Lattice style renderer (`liquidQuantumFluid`).
//!
//! Visual Concept:
//! - 3D Isometric Magnetic Hexagonal Ferrofluid Matrix Pin Array (37 Hexagonal Ferrofluid Pins across 3 Rings).
//! - Each hexagonal ferrofluid pin rises & falls independently like a physical mechanical equalizer matrix.
//! - Multi-layered cyan & violet magnetic flux field lines, specular peak caps, & quantum energy motes.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const HEX_PINS: usize = 37;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col      = theme_primary(theme);
    let _s_col     = theme_secondary(theme);
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
    let cy = height * 0.5 - pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 160.0 * (reference_size / 500.0) * user_scale;

    // Palette: 3D Magnetic Hexagonal Ferrofluid blended with theme
    let ferro_cyan   = mix(glow_col,   Color::rgba(0.0, 0.95, 1.0, 1.0), 0.85);
    let ferro_violet = mix(accent_col, Color::rgba(0.65, 0.05, 1.0, 1.0), 0.85);
    let ferro_mid    = mix(p_col,      mix(ferro_cyan, ferro_violet, 0.5), 0.50);
    let spark_white  = mix(glow_col,   Color::WHITE, 0.95);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Ambient Magnetic Field Backdrop Glow
    let bg_ferro = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, base_r * 2.5,
        &[
            (0.00, mix(ferro_cyan, ferro_violet, 0.5).with_alpha(0.35 + be * 0.20)),
            (0.50, ferro_mid.with_alpha(0.15 + bs * 0.08)),
            (1.00, Color::rgba(0.02, 0.01, 0.08, 1.0)),
        ],
    );
    c.set_fill(bg_ferro);
    c.fill_rect(0.0, 0.0, width, height);

    // Magnetic Flux Wave Rings
    for ring_i in 1..=4 {
        let rf = ring_i as f32;
        let ring_r = base_r * (0.35 + rf * 0.30 + (frame_time * 1.5 + rf).sin() * 0.05) * (1.0 + be * 0.10);
        c.set_stroke(Fill::Solid(mix(ferro_cyan, ferro_violet, rf / 4.0).with_alpha(0.35 - rf * 0.06)));
        c.set_line_width((2.0 + be * 1.0) * user_scale);
        c.stroke_circle(cx, cy, ring_r);
    }

    // 2. 37 Hexagonal Magnetic Ferrofluid Pins in 3D Isometric Concentric Rings
    let step = (freq.len() / HEX_PINS).max(1);

    for h_i_rev in 0..HEX_PINS {
        let h_i = HEX_PINS - 1 - h_i_rev;
        let hf  = h_i as f32;
        let ring_layer = if h_i == 0 { 0usize } else if h_i <= 6 { 1 } else if h_i <= 18 { 2 } else { 3 };

        let angle = if h_i == 0 {
            0.0f32
        } else {
            let pins_in_ring = match ring_layer { 1 => 6, 2 => 12, _ => 18 };
            let ring_slot = match ring_layer {
                1 => h_i - 1,
                2 => h_i - 7,
                _ => h_i - 19,
            };
            (ring_slot as f32 / pins_in_ring as f32) * TAU + frame_time * (0.05 + ring_layer as f32 * 0.02)
        };

        let bin_idx = (h_i * step) % freq.len().max(1);
        let fv = super::radial_common::smooth_ring_bin(freq, step, bin_idx, HEX_PINS);
        let val = (fv * sensitivity * 1.2 + be * 0.30 + bs * 0.20).clamp(0.08, 2.4);

        let pin_dist = ring_layer as f32 * (36.0 * user_scale);
        let (sin_a, cos_a) = angle.sin_cos();

        let px = cx + cos_a * pin_dist;
        let py = cy + sin_a * pin_dist * 0.72; // 3D isometric Y compression

        let pin_h   = (12.0 + val * 35.0) * user_scale;
        let hex_r   = (12.0 + val * 7.0)  * user_scale;
        let hex_col = mix(ferro_cyan, ferro_violet, hf / HEX_PINS as f32);

        // 3D Isometric Side Face (dark shadow quad below top hexagon)
        if pin_h > 2.0 {
            let mut side_top: Vec<(f32, f32)> = Vec::with_capacity(6);
            let mut side_bot: Vec<(f32, f32)> = Vec::with_capacity(6);
            for k in 0..6 {
                let ka = (k as f32 / 6.0) * TAU + std::f32::consts::FRAC_PI_6;
                let (ksin, kcos) = ka.sin_cos();
                side_top.push((px + kcos * hex_r,              py + ksin * (hex_r * 0.70)));
                side_bot.push((px + kcos * hex_r * 0.80, py + ksin * (hex_r * 0.70) + pin_h));
            }
            for k in 0..3 {
                let k2 = (k + 1) % 6;
                let face = vec![side_top[k], side_top[k2], side_bot[k2], side_bot[k]];
                let face_dark = mix(hex_col, Color::BLACK, 0.45).with_alpha(0.60);
                c.set_fill(Fill::Solid(face_dark));
                c.set_shadow(Color::TRANSPARENT, 0.0);
                c.fill_polygon(&face);
            }
        }

        // 3D Hexagonal Pin Top Face
        let mut hex_pts: Vec<(f32, f32)> = Vec::with_capacity(7);
        for k in 0..6 {
            let ka = (k as f32 / 6.0) * TAU + std::f32::consts::FRAC_PI_6;
            let (ksin, kcos) = ka.sin_cos();
            hex_pts.push((px + kcos * hex_r, py + ksin * (hex_r * 0.70)));
        }
        hex_pts.push(hex_pts[0]);

        c.set_fill(Fill::Solid(hex_col.with_alpha(0.85)));
        c.set_shadow(hex_col, (14.0 + val * 12.0) * user_scale);
        c.fill_polygon(&hex_pts);

        // Specular Top Cap Highlight
        c.set_stroke(Fill::Solid(spark_white.with_alpha(0.80)));
        c.set_line_width((1.8 + val * 1.0) * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_polyline(&hex_pts);

        // Energy quantum particle at pin peak
        if val > 0.4 {
            c.set_fill(Fill::Solid(spark_white.with_alpha((val - 0.4) * 0.85)));
            c.set_shadow(ferro_cyan, (10.0 + val * 10.0) * user_scale);
            c.fill_circle(px, py, (2.5 + val * 2.0) * user_scale);
        }
    }

    // 3. Central Quantum Core Node (glowing energy sphere)
    let core_r = (22.0 + be * 12.0 + bs * 8.0) * user_scale;
    let core_fill = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, core_r * 1.8,
        &[
            (0.00, spark_white.with_alpha(0.95)),
            (0.40, ferro_cyan.with_alpha(0.80)),
            (0.80, ferro_violet.with_alpha(0.45)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(core_fill);
    c.set_shadow(ferro_cyan, (24.0 + bs * 16.0) * user_scale);
    c.fill_circle(cx, cy, core_r);

    c.set_global_alpha(1.0);
    c.restore();
}

