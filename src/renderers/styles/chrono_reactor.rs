//! Chrono Clockwork Reactor style renderer (`chronoReactor`) — Steampunk Sci-Fi Arc Reactor Engine.
//!
//! Masterpiece Arc Reactor:
//! - Dual counter-rotating mechanical gear rings with 12 gear notches (NO needle spikes!).
//! - 6 concentric arc reactor energy plasma rings pulsing with audio frequency data.
//! - Tachyon particle plasma core driven by bass excursion.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const TEETH_COUNT: usize = 12;
const REACTOR_SECTORS: usize = 6;

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
    let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;
    let reference_size = width.min(height);
    let base_r = 105.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Arc Reactor Core Thermal Glow
    let reactor_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.2,
        cx,
        cy,
        base_r * 2.8,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.35 + be * 0.20), 0.5)),
            (0.45, mix(p_col, Color::rgba(1.0, 0.60, 0.0, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.10, 0.20, 0.40, 0.04), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(reactor_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. ROTATING BRASS & COPPER MECHANICAL GEAR (OUTER CONTAINMENT RING)
    // -------------------------------------------------------------------------
    let gear_r = base_r * (1.25 + be * 0.08);
    let gear_angle = frame_time * 0.25;

    c.set_fill(Fill::Solid(Color::hex("#121b28")));
    c.set_stroke(Fill::Solid(glow_col));
    c.set_line_width(2.5 * user_scale);
    c.set_shadow(glow_col, 16.0 * user_scale);
    c.fill_circle(cx, cy, gear_r);
    c.stroke_circle(cx, cy, gear_r);

    // Render 12 gear notches
    for g in 0..TEETH_COUNT {
        let a = (g as f32 / TEETH_COUNT as f32) * TAU + gear_angle;
        let (sin_a, cos_a) = a.sin_cos();

        let tx0 = cx + cos_a * gear_r;
        let ty0 = cy + sin_a * gear_r;
        let tx1 = cx + cos_a * (gear_r + 14.0 * user_scale);
        let ty1 = cy + sin_a * (gear_r + 14.0 * user_scale);

        let tooth_w = 12.0 * user_scale;
        let px = -sin_a * (tooth_w * 0.5);
        let py = cos_a * (tooth_w * 0.5);

        let t_pts = vec![
            (tx0 - px, ty0 - py),
            (tx0 + px, ty0 + py),
            (tx1 + px * 0.7, ty1 + py * 0.7),
            (tx1 - px * 0.7, ty1 - py * 0.7),
        ];

        c.set_fill(Fill::Solid(Color::hex("#1a293d")));
        c.fill_polygon(&t_pts);
        c.stroke_polyline(&t_pts);
    }

    // -------------------------------------------------------------------------
    // 2. 6 CONCENTRIC ARC REACTOR PLASMA RINGS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for s_i in 0..REACTOR_SECTORS {
        let s_f = s_i as f32;
        let bin_k = (s_i * 2 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let r_sector = base_r * (0.35 + s_f * 0.14 + fv * 0.05 * sensitivity);
        let sec_col = mix(
            mix(p_col, glow_col, s_f / REACTOR_SECTORS as f32),
            mix(accent_col, Color::rgba(1.0, 0.75, 0.0, 0.95), fv),
            fv,
        );

        let start_a = s_f * 0.6 + frame_time * (0.4 + s_f * 0.1);
        let end_a = start_a + TAU * 0.60;

        c.set_stroke(Fill::Solid(sec_col));
        c.set_line_width((3.0 + fv * 4.0) * user_scale);
        c.set_shadow(sec_col, (10.0 + fv * 10.0) * user_scale);
        c.stroke_arc(cx, cy, r_sector, start_a, end_a);
    }

    // -------------------------------------------------------------------------
    // 3. TACHYON PLASMA REACTOR CORE AT CENTER
    // -------------------------------------------------------------------------
    let core_r = base_r * (0.28 + be * 0.12);
    let core_grad = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        core_r,
        &[
            (0.0, Color::rgba(1.0, 1.0, 1.0, 0.98)),
            (0.40, mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 0.85), 0.6)),
            (1.0, mix(p_col, Color::hex("#05080c"), 0.8)),
        ],
    );

    c.set_fill(core_grad);
    c.set_shadow(glow_col, (18.0 + bs * 10.0) * user_scale);
    c.fill_circle(cx, cy, core_r);

    // Center image inside tachyon core (drawn last = on top)
    draw_radial_center_image(c, ctx, cx, cy, core_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
