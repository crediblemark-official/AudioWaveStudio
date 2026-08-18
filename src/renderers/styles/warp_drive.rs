//! Hyper-Speed Warp Drive style renderer (`warpDrive`) — Hyperspace Tunnel Engine.
//!
//! Masterpiece FTL Warp Tunnel:
//! - 8 concentric hyperspace warp containment rings expanding in 3D perspective depth (NO needle spikes!).
//! - 64 lightspeed starfield streaks bursting outward in FTL speed acceleration.
//! - FTL Singularity Core with blinding white-hot warp flash driven by bass hits.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const WARP_RINGS: usize = 8;
const STAR_STREAKS: usize = 64;

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
    let base_r = 30.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Hyperspace warp thermal glow

    // Hyperspace warp thermal glow
    let warp_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.2,
        cx,
        cy,
        width * 0.70 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.35 + be * 0.20), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.80, 0.10, 0.60, 0.15), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(warp_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    let step_f = (freq.len() / bar_count).max(1);

    // -------------------------------------------------------------------------
    // 1. 8 CONCENTRIC HYPERSPACE WARP CONTAINMENT RINGS
    // -------------------------------------------------------------------------
    for r_i in 0..WARP_RINGS {
        let r_t = ((frame_time * 0.8 + r_i as f32 * 0.125) % 1.0).clamp(0.0, 1.0);

        let bin_k = (r_i * 2 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let ring_r = base_r + r_t.powi(2) * (width * 0.48 * user_scale + fv * 70.0 * sensitivity);
        let ring_alpha = (1.0 - r_t).powi(2) * (0.80 + bs * 0.20);

        let ring_col = mix(
            mix(Color::rgba(0.0, 0.90, 1.0, ring_alpha), glow_col, r_t),
            mix(p_col, accent_col, r_t),
            r_t,
        );

        c.set_stroke(Fill::Solid(ring_col));
        c.set_line_width((4.0 * (1.0 - r_t) + 1.2) * user_scale);
        c.set_shadow(ring_col, (14.0 * (1.0 - r_t)) * user_scale);
        c.stroke_circle(cx, cy, ring_r);
    }

    // -------------------------------------------------------------------------
    // 2. 64 LIGHTSPEED STARFIELD ACCELERATION STREAKS
    // -------------------------------------------------------------------------
    for s_i in 0..STAR_STREAKS {
        let s_f = s_i as f32;
        let speed = 0.50 + (s_i % 5) as f32 * 0.12 + be * 0.40;
        let s_t = ((frame_time * speed + s_f * 0.06) % 1.0).clamp(0.0, 1.0);

        let angle = (s_f / STAR_STREAKS as f32) * TAU;
        let (sin_a, cos_a) = angle.sin_cos();

        let r0 = base_r + s_t.powi(2) * (width * 0.55 * user_scale);
        let r1 = r0 + (12.0 + s_t * 60.0 + be * 35.0) * user_scale;

        let x0 = cx + cos_a * r0;
        let y0 = cy + sin_a * r0;
        let x1 = cx + cos_a * r1;
        let y1 = cy + sin_a * r1;

        let alpha = (s_t * 0.95).clamp(0.1, 0.95);
        let streak_col = mix(
            mix(p_col, glow_col, (s_i % 3) as f32 / 3.0),
            mix(accent_col, s_col, (s_i % 2) as f32),
            alpha,
        );

        c.set_stroke(Fill::Solid(streak_col));
        c.set_line_width((1.2 + s_t * 3.0) * user_scale);
        c.stroke_line(x0, y0, x1, y1);
    }

    // -------------------------------------------------------------------------
    // 3. FTL SINGULARITY WARP FLASH CORE AT CENTER
    // -------------------------------------------------------------------------
    let core_r = base_r * (0.90 + be * 0.35 + bs * 0.15);
    let warp_core = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        core_r,
        &[
            (0.0, Color::rgba(1.0, 1.0, 1.0, 0.98)),
            (0.40, mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 0.85), 0.6)),
            (1.0, mix(p_col, Color::hex("#010206"), 0.85)),
        ],
    );

    c.set_fill(warp_core);
    c.set_shadow(glow_col, (22.0 + bs * 12.0) * user_scale);
    c.fill_circle(cx, cy, core_r);

    // Center image on top of warp core (drawn last = on top of all rings)
    draw_radial_center_image(c, ctx, cx, cy, core_r * 0.88);

    c.set_global_alpha(1.0);
    c.restore();
}
