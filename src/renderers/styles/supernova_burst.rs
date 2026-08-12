//! Supernova Shockwave Burst style renderer (`supernovaBurst`) — Stellar Detonation Engine.
//!
//! Masterpiece Supernova Detonation:
//! - Blinding white dwarf stellar core with pulsating bass excursion.
//! - 6 expanding volumetric plasma shockwave rings (NO needle spikes!).
//! - 64 glowing stellar embers accelerating outward into deep space.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const STAR_EMBERS: usize = 64;

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
    let cy = height * 0.5 - pos_offset_y;
    let reference_size = width.min(height);
    let base_r = 90.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Radial supernova glow aura
    let super_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.2,
        cx,
        cy,
        base_r * 4.2,
        &[
            (0.0, mix(glow_col, Color::rgba(1.0, 0.90, 0.40, 0.35 + be * 0.25), 0.5)),
            (0.35, mix(p_col, Color::rgba(0.0, 0.85, 1.0, 0.15), 0.5)),
            (0.75, mix(s_col, Color::rgba(0.40, 0.0, 0.80, 0.05), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(super_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. 6 EXPANDING VOLUMETRIC PLASMA SHOCKWAVE RINGS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for shock_i in 0..6 {
        let shock_t = ((frame_time * 0.5 + shock_i as f32 * 0.16) % 1.0).clamp(0.0, 1.0);

        let bin_k = (shock_i * 2 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let shock_r = base_r + shock_t * (width * 0.42 * user_scale + fv * 80.0 * sensitivity);
        let shock_alpha = (1.0 - shock_t).powi(2) * (0.75 + bs * 0.25);

        let shock_col = mix(
            mix(Color::rgba(1.0, 0.90, 0.30, shock_alpha), glow_col, shock_t),
            mix(p_col, accent_col, shock_t),
            shock_t,
        );

        c.set_stroke(Fill::Solid(shock_col));
        c.set_line_width((5.0 * (1.0 - shock_t) + 1.2) * user_scale);
        c.set_shadow(shock_col, (12.0 * (1.0 - shock_t)) * user_scale);
        c.stroke_circle(cx, cy, shock_r);
    }

    // -------------------------------------------------------------------------
    // 2. 64 SWIRLING STELLAR DETONATION EMBERS
    // -------------------------------------------------------------------------
    for s_i in 0..STAR_EMBERS {
        let s_f = s_i as f32;
        let angle = (s_f / STAR_EMBERS as f32) * TAU + frame_time * 0.15;

        let bin_k = (s_i * step_f / (STAR_EMBERS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let ember_dist = base_r * (1.10 + fv * 0.80 * sensitivity + be * 0.30) + (s_f * 4.0).sin() * (12.0 * user_scale);
        let ex = cx + angle.cos() * ember_dist;
        let ey = cy + angle.sin() * ember_dist;

        let ember_sz = (2.2 + (s_i % 3) as f32 * 1.5 + fv * 3.0) * user_scale;
        let ember_col = mix(Color::rgba(1.0, 1.0, 1.0, 0.95), glow_col, (s_i % 3) as f32 / 3.0);

        c.set_fill(Fill::Solid(ember_col));
        c.set_shadow(ember_col, (8.0 + fv * 6.0) * user_scale);
        c.fill_circle(ex, ey, ember_sz);
    }

    // -------------------------------------------------------------------------
    // 3. BLINDING WHITE DWARF STAR DETONATION CORE
    // -------------------------------------------------------------------------
    let core_r = base_r * (0.55 + be * 0.20 + bs * 0.10);
    let star_core = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        core_r,
        &[
            (0.0, Color::rgba(1.0, 1.0, 1.0, 0.98)),
            (0.40, mix(glow_col, Color::rgba(1.0, 0.85, 0.20, 0.90), 0.6)),
            (1.0, mix(p_col, Color::hex("#03020a"), 0.8)),
        ],
    );

    c.set_fill(star_core);
    c.set_shadow(Color::rgba(1.0, 0.90, 0.30, 0.95), (20.0 + bs * 12.0) * user_scale);
    c.fill_circle(cx, cy, core_r);

    // Center image on top of star detonation core (drawn last = on top)
    draw_radial_center_image(c, ctx, cx, cy, core_r * 0.70);

    c.set_global_alpha(1.0);
    c.restore();
}
