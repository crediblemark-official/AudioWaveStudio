//! Liquid Volcanic Fissure style renderer (`liquidMagmaCrustCore`).
//!
//! Visual Concept:
//! - Horizontal Tectonic Volcanic Rift Fissure splitting the screen horizontally.
//! - Roaring liquid lava river underneath with cracking obsidian crust rock edges.
//! - Audio-reactive lava surge pulses & volcanic ember motes rising from the rift.


use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const RIFT_NODES: usize = 24;
const EMBER_COUNT: usize = 18;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col     = theme_primary(theme);
    let s_col     = theme_secondary(theme);
    let accent    = theme_accent(theme);
    let glow_col  = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let user_scale  = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

    let be = ctx.bass_energy.clamp(0.0, 3.0);
    let bs = ctx.beat_strength.clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 - pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08);

    // Palette: Tectonic Lava Fissure + theme colours blended in
    let magma_fire    = mix(p_col,   Color::rgba(1.0, 0.20, 0.0, 1.0), 0.35);
    let magma_gold    = mix(accent,  Color::rgba(1.0, 0.65, 0.0, 1.0), 0.40);
    let obsidian_dark = Color::rgba(0.06, 0.02, 0.01, 0.95);
    let spark_white   = mix(glow_col, Color::WHITE, 0.80);
    let ember_col     = mix(s_col, Color::rgba(1.0, 0.90, 0.20, 1.0), 0.55);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Ambient Volcanic Fissure Backdrop Glow — rendered to canvas
    let rift_y = cy;
    let bg_fissure = Fill::radial_gradient(
        cx, rift_y, inner_r,
        cx, rift_y, width.max(height),
        &[
            (0.0,  mix(magma_fire, magma_gold, 0.5).with_alpha(0.40 + be * 0.20)),
            (0.45, obsidian_dark.with_alpha(0.18)),
            (1.0,  Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_fissure);
    c.fill_rect(0.0, 0.0, width, height);

    // 2. Horizontal Jagged Tectonic Lava Rift Channel
    let step = (freq.len() / RIFT_NODES).max(1);
    let mut top_pts: Vec<(f32, f32)> = Vec::with_capacity(RIFT_NODES + 2);
    let mut bot_pts: Vec<(f32, f32)> = Vec::with_capacity(RIFT_NODES + 2);

    top_pts.push((0.0, rift_y - 15.0 * user_scale));
    bot_pts.push((0.0, rift_y + 15.0 * user_scale));

    for i in 0..=RIFT_NODES {
        let rt    = i as f32 / RIFT_NODES as f32;
        let x_pos = rt * width;

        let fv  = super::radial_common::full_random_scattered_bin(freq, step, i, RIFT_NODES, ctx.beat_count);
        let val = (fv * sensitivity * 1.1 + be * 0.30 + bs * 0.20).clamp(0.08, 2.2);

        let gap_h  = (15.0 + val * 45.0) * user_scale;
        let jagged = (rt * 27.0 + frame_time * 2.0).sin() * (8.0 * user_scale);

        top_pts.push((x_pos, rift_y - gap_h * 0.5 + jagged));
        bot_pts.push((x_pos, rift_y + gap_h * 0.5 + jagged));
    }

    top_pts.push((width, rift_y - 15.0 * user_scale));
    bot_pts.push((width, rift_y + 15.0 * user_scale));

    // Fill Roaring Lava Channel Inside Fissure
    let mut rift_poly: Vec<(f32, f32)> = Vec::new();
    rift_poly.extend(top_pts.iter().copied());
    rift_poly.extend(bot_pts.iter().copied().rev());

    let lava_fill = Fill::linear_gradient(cx, rift_y - 60.0 * user_scale, cx, rift_y + 60.0 * user_scale, &[
        (0.0, magma_gold.with_alpha(0.90)),
        (0.5, magma_fire),
        (1.0, magma_gold.with_alpha(0.90)),
    ]);
    c.set_fill(lava_fill);
    c.set_shadow(magma_fire, (22.0 + bs * 16.0) * user_scale);
    c.fill_polygon(&rift_poly);

    // Obsidian Crack Edges
    c.set_stroke(Fill::Solid(spark_white.with_alpha(0.80)));
    c.set_line_width((2.0 + bs * 1.5) * user_scale);
    c.set_shadow(magma_gold, (10.0 + bs * 8.0) * user_scale);
    c.stroke_polyline(&top_pts);
    c.stroke_polyline(&bot_pts);

    // 3. Volcanic Ember Motes rising from the rift
    let emb_step = (freq.len() / EMBER_COUNT).max(1);
    for i in 0..EMBER_COUNT {
        let ef   = i as f32;
        let phase = (ef * 0.618_034 + frame_time * (0.55 + ef * 0.07)) % 1.0;

        let fv  = super::radial_common::full_random_scattered_bin(freq, emb_step, i, EMBER_COUNT, ctx.beat_count.wrapping_add(7));
        let val = (fv * sensitivity + be * 0.25 + bs * 0.10).clamp(0.0, 1.5);

        // Embers appear along the rift and float upward
        let ex = (ef / EMBER_COUNT as f32) * width + (ef * 3.14).sin() * (30.0 * user_scale);
        // rift_y is the horizon; embers rise above it
        let rise = phase * (height * 0.35 + val * height * 0.15);
        let ey   = rift_y - rise;
        let alpha = (1.0 - phase) * (0.65 + val * 0.30);

        if alpha > 0.05 {
            let e_col = mix(magma_fire, ember_col, phase);
            let e_sz  = (3.5 + val * 3.5 + (1.0 - phase) * 2.0) * user_scale;

            c.set_fill(Fill::Solid(e_col.with_alpha(alpha)));
            c.set_shadow(magma_gold, (8.0 + val * 6.0) * user_scale);
            c.fill_circle(ex, ey, e_sz);
        }
    }

    // 4. Central Core Disc
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(magma_fire));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(magma_fire, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    let _ = s_col; // used via ember_col mixing above
    c.set_global_alpha(1.0);
    c.restore();
}
