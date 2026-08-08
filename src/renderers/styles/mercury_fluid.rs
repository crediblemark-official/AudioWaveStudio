//! Liquid Mercury Fluid Wave style renderer (`mercuryFluid`) — Molten Liquid Chrome Engine.
//!
//! Masterpiece Molten Liquid Mercury:
//! - Continuous 3D liquid mercury surface with realistic metallic chrome shading and specular reflections.
//! - Audio-reactive concentric liquid mercury waves & molten pool ripples.
//! - Liquid chrome droplets splashing in 3D perspective space.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const WAVE_RINGS: usize = 12;
const MERCURY_DROPLETS: usize = 48;

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
    let cy = height * 0.55 - pos_offset_y;
    let reference_size = width.min(height);
    let base_r = 140.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep liquid metal space backdrop
    c.set_fill(Fill::Solid(Color::hex("#02050a")));
    c.fill_rect(0.0, 0.0, width, height);

    // Liquid mercury metallic ambient glow
    let mercury_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.3,
        cx,
        cy,
        base_r * 3.5,
        &[
            (0.0, mix(glow_col, Color::rgba(0.75, 0.88, 1.0, 0.35 + be * 0.20), 0.5)),
            (0.40, mix(p_col, Color::rgba(0.25, 0.45, 0.85, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.05, 0.10, 0.25, 0.04), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(mercury_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. LIQUID MERCURY REFLECTIVE POOL BASE
    // -------------------------------------------------------------------------
    let pool_rx = base_r * 1.65;
    let pool_ry = pool_rx * 0.38; // Perspective tilt

    let pool_grad = Fill::radial_gradient(
        cx - pool_rx * 0.20,
        cy - pool_ry * 0.20,
        0.0,
        cx,
        cy,
        pool_rx,
        &[
            (0.0, Color::rgba(0.95, 0.98, 1.0, 0.95)),
            (0.35, Color::rgba(0.55, 0.65, 0.80, 0.85)),
            (0.70, Color::rgba(0.18, 0.25, 0.38, 0.90)),
            (1.0, Color::rgba(0.04, 0.08, 0.15, 0.95)),
        ],
    );

    c.set_fill(pool_grad);
    c.set_stroke(Fill::Solid(glow_col));
    c.set_line_width(2.5 * user_scale);
    c.set_shadow(glow_col, 20.0 * user_scale);
    c.fill_ellipse(cx, cy, pool_rx, pool_ry);

    // -------------------------------------------------------------------------
    // 2. CONCENTRIC MOLTEN LIQUID MERCURY RIPPLE WAVES
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for r_i in 1..=WAVE_RINGS {
        let r_f = r_i as f32;
        let bin_k = (r_i * 2 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let rx = base_r * (0.25 + r_f * 0.12 + fv * 0.08 * sensitivity);
        let _ry = rx * 0.38;

        let ripple_col = mix(
            Color::rgba(0.95, 0.98, 1.0, 0.90 + bs * 0.10),
            mix(glow_col, accent_col, fv),
            r_f / WAVE_RINGS as f32,
        );

        c.set_stroke(Fill::Solid(ripple_col));
        c.set_line_width((2.5 + fv * 3.5) * user_scale);
        c.set_shadow(ripple_col, (10.0 + fv * 8.0) * user_scale);

        c.save();
        c.translate(cx, cy + (r_f * 4.0).sin() * (6.0 * user_scale));
        c.stroke_line(cx - rx, cy, cx + rx, cy); // Crisp metallic highlight line
        c.restore();
    }

    // -------------------------------------------------------------------------
    // 3. MOLTEN MERCURY DROPLETS SPLASHING UPWARD
    // -------------------------------------------------------------------------
    for d_i in 0..MERCURY_DROPLETS {
        let d_f = d_i as f32;
        let speed = 0.40 + (d_i % 4) as f32 * 0.15 + be * 0.50;
        let d_t = ((frame_time * speed + d_f * 0.08) % 1.0).clamp(0.0, 1.0);

        let angle = (d_f / MERCURY_DROPLETS as f32) * TAU;
        let bin_k = (d_i * step_f / (MERCURY_DROPLETS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let drop_r = base_r * (0.20 + d_t * 0.85 + fv * 0.20 * sensitivity);
        let drop_y = cy - (d_t * std::f32::consts::PI).sin() * (120.0 * user_scale + be * 60.0);
        let drop_x = cx + angle.cos() * drop_r;

        let drop_sz = (3.0 + (d_i % 3) as f32 * 2.0 + fv * 3.5) * user_scale;
        let drop_grad = Fill::radial_gradient(
            drop_x - drop_sz * 0.3,
            drop_y - drop_sz * 0.3,
            0.0,
            drop_x,
            drop_y,
            drop_sz,
            &[
                (0.0, Color::rgba(1.0, 1.0, 1.0, 0.98)),
                (0.40, Color::rgba(0.70, 0.82, 0.95, 0.90)),
                (1.0, Color::rgba(0.20, 0.30, 0.45, 0.85)),
            ],
        );

        c.set_fill(drop_grad);
        c.set_shadow(Color::rgba(0.90, 0.95, 1.0, 0.90), (8.0 + fv * 6.0) * user_scale);
        c.fill_circle(drop_x, drop_y, drop_sz);
    }

    // -------------------------------------------------------------------------
    // 4. MOLTEN MERCURY LIQUID CORE SPLASH AT CENTER
    // -------------------------------------------------------------------------
    let core_rx = base_r * (0.35 + be * 0.15 + bs * 0.08);
    let core_ry = core_rx * 0.38;

    let core_grad = Fill::radial_gradient(
        cx - core_rx * 0.2,
        cy - core_ry * 0.2,
        0.0,
        cx,
        cy,
        core_rx,
        &[
            (0.0, Color::rgba(1.0, 1.0, 1.0, 0.98)),
            (0.40, mix(glow_col, Color::rgba(0.80, 0.92, 1.0, 0.95), 0.6)),
            (1.0, mix(p_col, Color::hex("#02050a"), 0.85)),
        ],
    );

    c.set_fill(core_grad);
    c.set_shadow(glow_col, (24.0 + bs * 14.0) * user_scale);
    c.fill_circle(cx, cy - (be * 25.0 * user_scale), core_rx * 0.60);

    c.set_global_alpha(1.0);
    c.restore();
}
