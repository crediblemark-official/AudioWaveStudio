//! Cosmic Nebula Particle Cloud 3D style renderer (`nebulaCloud3D`) — Volumetric Interstellar Gas Engine.
//!
//! Masterpiece Volumetric Interstellar Cloud:
//! - Multi-layered translucent cosmic gas puffs that blend into a glowing interstellar cloud.
//! - 120+ glowing starlight embers & cosmic dust filaments swirling in spiral galaxy arms.
//! - Audio-reactive stellar core pulses & bass shockwave expansion.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const NEBULA_CLOUDS: usize = 48;
const STAR_EMBERS: usize = 120;

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

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Ambient interstellar cloud glow
    let amb_nebula = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.70 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.85, 1.0, 0.35 + be * 0.15), 0.5)),
            (0.40, mix(p_col, Color::rgba(0.80, 0.10, 0.60, 0.18), 0.5)),
            (0.75, mix(s_col, Color::rgba(0.20, 0.0, 0.40, 0.06), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(amb_nebula);
//     c.fill_rect(0.0, 0.0, width, height);

    let step_f = (freq.len() / bar_count).max(1);

    // -------------------------------------------------------------------------
    // 1. VOLUMETRIC INTERSTELLAR GAS PUFFS (SOFT BLENDED NEBULA CLOUDS)
    // -------------------------------------------------------------------------
    for g_i in 0..NEBULA_CLOUDS {
        let g_f = g_i as f32;
        let bin_k = (g_i * step_f / (NEBULA_CLOUDS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let orbit_r = (50.0 + g_f * 4.5 + fv * 90.0 * sensitivity + be * 40.0) * user_scale;
        let angle = g_f * 0.38 + frame_time * (0.25 + (g_i % 3) as f32 * 0.05);

        let cloud_x = cx + angle.cos() * orbit_r;
        let cloud_y = cy + (angle * 1.5 + g_f * 0.4).sin() * (60.0 * user_scale);

        let cloud_r = (45.0 + (g_i % 4) as f32 * 12.0 + fv * 35.0) * user_scale;

        let gas_color = mix(
            mix(p_col, glow_col, (g_i % 3) as f32 / 3.0),
            mix(accent_col, s_col, (g_i % 2) as f32),
            g_f / NEBULA_CLOUDS as f32,
        );

        let soft_gas = Fill::radial_gradient(
            cloud_x,
            cloud_y,
            0.0,
            cloud_x,
            cloud_y,
            cloud_r,
            &[
                (0.0, Color::rgba(gas_color.r, gas_color.g, gas_color.b, 0.28 + fv * 0.15)),
                (0.50, Color::rgba(gas_color.r, gas_color.g, gas_color.b, 0.10)),
                (1.0, Color::TRANSPARENT),
            ],
        );

        c.set_fill(soft_gas);
        c.fill_circle(cloud_x, cloud_y, cloud_r);
    }

    // -------------------------------------------------------------------------
    // 2. SWIRLING STARLIGHT EMBERS & COSMIC DUST SPARKLES
    // -------------------------------------------------------------------------
    for s_i in 0..STAR_EMBERS {
        let s_f = s_i as f32;
        let bin_k = (s_i * step_f / (STAR_EMBERS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let orbit_r = (30.0 + s_f * 2.8 + fv * 80.0 * sensitivity + be * 35.0) * user_scale;
        let angle = s_f * 0.24 + frame_time * 0.35;

        let star_x = cx + angle.cos() * orbit_r;
        let star_y = cy + (angle * 2.0 + s_f * 0.5).sin() * (50.0 * user_scale);

        let star_r = (1.8 + (s_i % 3) as f32 * 1.2 + fv * 2.5) * user_scale;

        let star_color = mix(
            Color::rgba(1.0, 1.0, 1.0, 0.95),
            glow_col,
            (s_i % 3) as f32 / 3.0,
        );

        c.set_fill(Fill::Solid(star_color));
        c.set_shadow(star_color, (6.0 + fv * 8.0) * user_scale);
        c.fill_circle(star_x, star_y, star_r);
    }

    // -------------------------------------------------------------------------
    // 3. INTENSE GLOWING STELLAR CORE AT NEBULA CENTER
    // -------------------------------------------------------------------------
    let core_r = (30.0 + be * 20.0 * sensitivity + bs * 10.0) * user_scale;
    let core_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        core_r * 2.5,
        &[
            (0.0, Color::rgba(1.0, 1.0, 1.0, 0.98)),
            (0.30, mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 0.85), 0.6)),
            (0.70, mix(p_col, Color::rgba(0.90, 0.10, 0.70, 0.35), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(core_glow);
    c.fill_circle(cx, cy, core_r * 2.5);

    c.set_global_alpha(1.0);
    c.restore();
}
