//! Synthwave Sun style renderer (`SynthwaveSun`) — 80s Retrowave Outrun Sun Engine.
//!
//! Masterpiece 80s Retrowave Outrun Visualizer:
//! - Radiant 80s Synthwave Sun Disc with gradient transition (white-hot gold -> neon orange -> hot magenta).
//! - Venetian blind horizontal cutouts tapering towards the horizon with audio-reactive spectrum modulation.
//! - 3D Perspective Outrun Cyber Grid Floor with receding perspective lines & audio bass pulses.
//! - Dark 80s Synthwave Vector Mountain Ridge Silhouettes against the glowing sunset horizon.
//! - 45+ Floating retrowave stardust particles & neon energy sparks.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const SUN_SLICES: usize = 16;
const GRID_LINES: usize = 16;

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
    let cy = height * 0.42 + pos_offset_y;

    let reference_size = width.min(height);
    let sun_r = 135.0 * (reference_size / 500.0) * user_scale * (1.0 + be * 0.08);

    // Curated 80s Synthwave Palette
    let sun_top = Color::rgba(1.0, 0.95, 0.20, 1.0);     // White-hot yellow
    let sun_mid = mix(p_col, Color::rgba(1.0, 0.35, 0.0, 1.0), 0.75); // Neon Orange
    let sun_bot = mix(accent_col, Color::rgba(0.95, 0.0, 0.55, 1.0), 0.75); // Hot Magenta
    let grid_cyan = mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 1.0), 0.75);
    let grid_purple = mix(s_col, Color::rgba(0.55, 0.0, 0.95, 1.0), 0.75);
    let spark_white = Color::rgba(0.98, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC 80s RETRO SUNSET BACKDROP GLOW
    // -------------------------------------------------------------------------
    let bg_sun = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        sun_r * 2.4,
        &[
            (0.0, mix(sun_top, sun_mid, 0.5).with_alpha(0.40 + be * 0.20)),
            (0.55, mix(sun_bot, grid_purple, 0.5).with_alpha(0.20)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_sun);

    // -------------------------------------------------------------------------
    // 2. 3D PERSPECTIVE OUTRUN CYBER GRID FLOOR (HORIZON TO BOTTOM)
    // -------------------------------------------------------------------------
    let horizon_y = cy + sun_r * 0.45;
    let floor_h = height - horizon_y;

    // Horizon Glow Line
    c.set_stroke(Fill::Solid(mix(sun_bot, grid_cyan, 0.5)));
    c.set_line_width((2.8 + bs * 2.0) * user_scale);
    c.set_shadow(grid_cyan, (16.0 + bs * 10.0) * user_scale);
    c.stroke_line(0.0, horizon_y, width, horizon_y);

    // Horizontal Receding Perspective Grid Lines
    for g_i in 1..=GRID_LINES {
        let g_t = (g_i as f32 / GRID_LINES as f32).powf(2.2); // Exponential perspective spacing
        let line_y = horizon_y + g_t * floor_h;

        let alpha = (g_t * 0.75 + 0.15).clamp(0.0, 0.90);
        let col = mix(grid_purple, grid_cyan, g_t);

        c.set_stroke(Fill::Solid(col.with_alpha(alpha)));
        c.set_line_width((1.2 + g_t * 1.8) * user_scale);
        c.set_shadow(col, (6.0 + g_t * 8.0) * user_scale);
        c.stroke_line(0.0, line_y, width, line_y);
    }

    // Perspective Vanishing Longitudinal Grid Lines
    let v_lines = 14usize;
    for v_i in 0..=v_lines {
        let vt = v_i as f32 / v_lines as f32;
        let top_x = cx + (vt - 0.5) * (width * 0.15);
        let bot_x = cx + (vt - 0.5) * (width * 1.60);

        c.set_stroke(Fill::Solid(grid_purple.with_alpha(0.50)));
        c.set_line_width(1.4 * user_scale);
        c.set_shadow(grid_purple, 8.0 * user_scale);
        c.stroke_line(top_x, horizon_y, bot_x, height);
    }

    // -------------------------------------------------------------------------
    // 3. SYNTHWAVE MOUNTAIN RIDGE SILHOUETTES
    // -------------------------------------------------------------------------
    let mtn_pts = [
        (0.0, horizon_y),
        (width * 0.10, horizon_y - 25.0 * user_scale),
        (width * 0.22, horizon_y - 45.0 * user_scale),
        (width * 0.35, horizon_y - 15.0 * user_scale),
        (width * 0.48, horizon_y - 65.0 * user_scale),
        (width * 0.62, horizon_y - 30.0 * user_scale),
        (width * 0.76, horizon_y - 50.0 * user_scale),
        (width * 0.88, horizon_y - 20.0 * user_scale),
        (width, horizon_y),
    ];
    c.set_fill(Fill::Solid(Color::rgba(0.05, 0.0, 0.12, 0.88)));
    c.fill_polygon(&mtn_pts);

    // -------------------------------------------------------------------------
    // 4. RADIANT 80s SYNTHWAVE SUN DISC & VENETIAN BLIND HORIZON CUTOUTS
    // -------------------------------------------------------------------------
    // Draw full glowing sun disc first
    let sun_grad = Fill::radial_gradient(
        cx,
        cy - sun_r * 0.3,
        0.0,
        cx,
        cy,
        sun_r * 1.1,
        &[
            (0.0, sun_top),
            (0.5, sun_mid),
            (1.0, sun_bot),
        ],
    );
    c.set_fill(sun_grad);
    c.set_shadow(sun_mid, (24.0 + bs * 16.0) * user_scale);
    c.fill_circle(cx, cy, sun_r);

    // Venetian blind horizontal laser cutouts on lower sun half
    let step = (freq.len() / SUN_SLICES).max(1);
    let cut_count = 10usize;
    let cut_start_y = cy - sun_r * 0.10;
    let cut_end_y = cy + sun_r * 0.95;

    for s in 0..cut_count {
        let sf = s as f32;
        let st = sf / cut_count as f32;
        let y_pos = cut_start_y + st * (cut_end_y - cut_start_y);

        let fv = super::radial_common::full_random_scattered_bin(freq, step, s, cut_count, ctx.beat_count);
        let cut_h = (2.0 + st * 10.0 + fv * sensitivity * 8.0) * user_scale;

        let dy = y_pos - cy;
        if dy.abs() < sun_r {
            let half_w = (sun_r * sun_r - dy * dy).sqrt() * 1.05;
            c.set_fill(Fill::Solid(Color::hex("#050010"))); // Deep dark background cutout slice
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.fill_rect(cx - half_w, y_pos - cut_h * 0.5, half_w * 2.0, cut_h);
        }
    }

    // -------------------------------------------------------------------------
    // 5. FLOATING RETRO STARDUST PARTICLES & NEON SPARKS
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + be * 24.0).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.35 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = m_t * (sun_r * 1.6);

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist - m_t * (50.0 * user_scale); // Rising upward into sunset

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(sun_top, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(sun_top, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // -------------------------------------------------------------------------
    // 6. PUMPING CENTRAL DISC & SUN HUB RESERVOIR
    // -------------------------------------------------------------------------
    let inner_r = sun_r * 0.32;
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(sun_top));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(sun_top, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}

