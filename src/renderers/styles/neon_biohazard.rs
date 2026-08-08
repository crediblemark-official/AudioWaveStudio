//! Neon Bio-Hazard Pulse style renderer (`neonBiohazard`) — Cyber Toxic Engine.
//!
//! Masterpiece 3D Biohazard Emblem:
//! - Iconic 3-blade Biohazard trefoil emblem pulsating with bass beats (NO needle spikes!).
//! - Concentric toxic hazard warning rings & rotating hazard sector arcs.
//! - Toxic green & electric cyan plasma core.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

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
    let base_r = 110.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Toxic green ambient glow
    let toxic_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.5,
        cx,
        cy,
        base_r * 3.2,
        &[
            (0.0, mix(glow_col, Color::rgba(0.20, 1.0, 0.10, 0.35 + be * 0.20), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.0, 0.85, 0.40, 0.12), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.0, 0.30, 0.15, 0.04), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(toxic_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CONCENTRIC TOXIC HAZARD WARNING RINGS (ROTATING SECTORS)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for r_i in 1..=6 {
        let r_f = r_i as f32;
        let bin_k = (r_i * 2 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let warn_r = base_r * (0.50 + r_f * 0.18 + fv * 0.08 * sensitivity);
        let warn_col = mix(
            mix(p_col, glow_col, r_f / 6.0),
            mix(accent_col, Color::rgba(1.0, 0.85, 0.0, 0.95), fv),
            fv,
        );

        let start_a = r_f * 0.5 + frame_time * (0.3 + r_f * 0.1);
        let end_a = start_a + TAU * 0.50;

        c.set_stroke(Fill::Solid(warn_col));
        c.set_line_width((3.0 + fv * 5.0) * user_scale);
        c.set_shadow(warn_col, (12.0 + fv * 10.0) * user_scale);
        c.stroke_arc(cx, cy, warn_r, start_a, end_a);
    }

    // -------------------------------------------------------------------------
    // 2. ICONIC 3-BLADE NEON BIOHAZARD TREFOIL EMBLEM
    // -------------------------------------------------------------------------
    let blade_dist = base_r * (0.55 + be * 0.10);
    let blade_r = base_r * 0.45;
    let rotation = frame_time * 0.20;

    for b in 0..3 {
        let angle = (b as f32 / 3.0) * TAU + rotation;
        let bx = cx + angle.cos() * blade_dist;
        let by = cy + angle.sin() * blade_dist;

        let bin_k = (b * 8 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let blade_col = mix(glow_col, Color::rgba(0.20, 1.0, 0.10, 0.90 + bs * 0.10), fv);

        c.set_fill(Fill::Solid(Color::rgba(blade_col.r, blade_col.g, blade_col.b, 0.45 + fv * 0.35)));
        c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.90)));
        c.set_line_width(2.5 * user_scale);
        c.set_shadow(blade_col, (16.0 + bs * 10.0) * user_scale);

        c.fill_circle(bx, by, blade_r * (1.0 + fv * 0.20 * sensitivity));
        c.stroke_circle(bx, by, blade_r * (1.0 + fv * 0.20 * sensitivity));
    }

    // Central Biohazard Cutout Core
    c.set_fill(Fill::Solid(Color::hex("#020a04")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, base_r * 0.28);

    // Inner Glowing Toxic Core
    let core_r = base_r * (0.16 + be * 0.08);
    c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.95)));
    c.set_shadow(glow_col, (14.0 + bs * 8.0) * user_scale);
    c.fill_circle(cx, cy, core_r);

    c.set_global_alpha(1.0);
    c.restore();
}
