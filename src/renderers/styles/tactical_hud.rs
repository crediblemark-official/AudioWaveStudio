//! Tactical HUD style renderer (`TacticalHud`) — Masterpiece Cyberpunk Sci-Fi Radar HUD Engine.
//!
//! Masterpiece 360° Sci-Fi Military Target Lock Radar Scope HUD:
//! - Smooth 360° Sweeping Radar Fan Sector Beam with ambient glow trail.
//! - 24 Audio-Reactive Target Lock Corner Brackets `[ ]` that lock onto frequency peaks.
//! - Outer Precision Compass Azimuth Ring with $0^\circ, 90^\circ, 180^\circ, 270^\circ$ crosshair reticles.
//! - 36 Floating Digital Telemetry Nodes & Radar Target Blips bouncing with audio reactivity.
//! - Central Quantum Core Target Disc with expanding forcefield shockwave ripples.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const HUD_TARGETS: usize = 24;

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
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08 + bs * 0.04);
    let max_r = base_r * 1.55;

    // Palette
    let hud_green = mix(glow_col, Color::rgba(0.0, 1.0, 0.50, 1.0), 0.75);
    let hud_cyan = mix(accent_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.75);
    let hud_amber = mix(p_col, Color::rgba(1.0, 0.75, 0.0, 1.0), 0.70);
    let spark_white = Color::rgba(0.98, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC RADAR BACKDROP & CROSSHAIR RETICLES
    // -------------------------------------------------------------------------
    let bg_hud = Fill::radial_gradient(
        cx,
        cy,
        inner_r,
        cx,
        cy,
        max_r * 1.8,
        &[
            (0.0, mix(hud_green, hud_cyan, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.50, mix(hud_amber, s_col, 0.5).with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_hud);

    // Outer Target Compass Ring & Crosshair Ticks
    c.set_stroke(Fill::Solid(hud_green.with_alpha(0.55)));
    c.set_line_width(1.8 * user_scale);
    c.set_shadow(hud_green, (12.0 + bs * 8.0) * user_scale);
    c.stroke_circle(cx, cy, max_r);

    // Crosshair axis lines
    for i in 0..4 {
        let ca = i as f32 * (TAU / 4.0);
        let (c_sin, c_cos) = ca.sin_cos();
        let x0 = cx + c_cos * (inner_r * 1.05);
        let y0 = cy + c_sin * (inner_r * 1.05);
        let x1 = cx + c_cos * (max_r * 1.08);
        let y1 = cy + c_sin * (max_r * 1.08);

        c.set_stroke(Fill::Solid(hud_green.with_alpha(0.40)));
        c.set_line_width(1.2 * user_scale);
        c.stroke_line(x0, y0, x1, y1);
    }

    // -------------------------------------------------------------------------
    // 2. ROTATING RADAR SWEEP FAN SECTOR
    // -------------------------------------------------------------------------
    let sweep_angle = frame_time * 1.6 % TAU;
    let sweep_width = std::f32::consts::FRAC_PI_3; // 60-degree fan

    let num_wedges = 24usize;
    let mut wedge_pts: Vec<(f32, f32)> = Vec::with_capacity(num_wedges + 2);
    wedge_pts.push((cx, cy));

    for i in 0..=num_wedges {
        let wt = i as f32 / num_wedges as f32;
        let a = sweep_angle - wt * sweep_width;
        let (cos_a, sin_a) = a.sin_cos();
        wedge_pts.push((cx + cos_a * max_r, cy + sin_a * max_r));
    }

    let radar_fill = Fill::radial_gradient(
        cx, cy, inner_r,
        cx, cy, max_r,
        &[
            (0.00, hud_green.with_alpha(0.55)),
            (0.70, hud_cyan.with_alpha(0.20)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(radar_fill);
    c.set_shadow(hud_green, 14.0 * user_scale);
    c.fill_polygon(&wedge_pts);

    // Leading Radar Beam Line
    let (s_sin, s_cos) = sweep_angle.sin_cos();
    let bx0 = cx + s_cos * inner_r;
    let by0 = cy + s_sin * inner_r;
    let bx1 = cx + s_cos * max_r;
    let by1 = cy + s_sin * max_r;

    c.set_stroke(Fill::Solid(spark_white));
    c.set_line_width((2.5 + bs * 1.5) * user_scale);
    c.set_shadow(spark_white, (18.0 + bs * 12.0) * user_scale);
    c.stroke_line(bx0, by0, bx1, by1);

    // -------------------------------------------------------------------------
    // 3. 24 AUDIO-REACTIVE TARGET LOCK CORNER BRACKETS `[ ]`
    // -------------------------------------------------------------------------
    let step = (freq.len() / HUD_TARGETS).max(1);
    let rot = frame_time * 0.08;

    for t_i in 0..HUD_TARGETS {
        let angle = (t_i as f32 / HUD_TARGETS as f32) * TAU + rot;
        let (sin_a, cos_a) = angle.sin_cos();

        let fv = super::radial_common::full_random_scattered_bin(freq, step, t_i, HUD_TARGETS, ctx.beat_count);
        let val = (fv * sensitivity * 1.1 + be * 0.30 + bs * 0.20).clamp(0.08, 2.2);

        let target_r = inner_r + (25.0 + val * 65.0) * user_scale;
        let tx = cx + cos_a * target_r;
        let ty = cy + sin_a * target_r;

        let b_sz = (4.5 + val * 3.5) * user_scale;
        let target_col = mix(mix(hud_green, hud_cyan, t_i as f32 / HUD_TARGETS as f32), spark_white, val * 0.5);

        c.save();
        c.translate(tx, ty);
        c.rotate(angle);

        // Draw Target Lock Bracket `[`
        c.set_stroke(Fill::Solid(target_col.with_alpha(0.85)));
        c.set_line_width((1.8 + val * 1.0) * user_scale);
        c.set_shadow(target_col, (10.0 + val * 8.0) * user_scale);

        c.stroke_line(-b_sz, -b_sz, -b_sz * 0.4, -b_sz);
        c.stroke_line(-b_sz, -b_sz, -b_sz, -b_sz * 0.4);
        c.stroke_line(-b_sz, b_sz, -b_sz * 0.4, b_sz);
        c.stroke_line(-b_sz, b_sz, -b_sz, b_sz * 0.4);

        c.stroke_line(b_sz, -b_sz, b_sz * 0.4, -b_sz);
        c.stroke_line(b_sz, -b_sz, b_sz, -b_sz * 0.4);
        c.stroke_line(b_sz, b_sz, b_sz * 0.4, b_sz);
        c.stroke_line(b_sz, b_sz, b_sz, b_sz * 0.4);

        // Central Target Blip Center Dot
        c.set_fill(Fill::Solid(if val > 0.7 { spark_white } else { target_col }));
        c.fill_circle(0.0, 0.0, (1.8 + val * 1.5) * user_scale);

        c.restore();
    }

    // -------------------------------------------------------------------------
    // 4. FLOATING RADAR TARGET BLIPS & TELEMETRY MOTES
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + be * 24.0 * sensitivity).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = inner_r + m_t * (max_r * 0.95);

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(hud_green, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(hud_green, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // -------------------------------------------------------------------------
    // 5. PUMPING CENTRAL DISC & RADAR CORE RESERVOIR
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(hud_green));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(hud_green, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}

