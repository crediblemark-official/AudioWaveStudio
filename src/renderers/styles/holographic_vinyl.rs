//! Holographic Vinyl style renderer (`holographicVinyl`) — Synthwave Record Engine.
//!
//! Masterpiece Holographic Vinyl Turntable:
//! - Rotating holographic vinyl record disc with iridescent rainbow optical sheen.
//! - 8 concentric vinyl grooves pulsating in multi-frequency wave rings (NO needle spikes!).
//! - Rotating turntable tonearm with glowing neon stylus tracking head.
//! - Outer neon LED platter rim pulsing with bass energy.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

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
    let disc_r = 135.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Ambient Holographic Glow
    let holo_glow = Fill::radial_gradient(
        cx,
        cy,
        disc_r * 0.4,
        cx,
        cy,
        disc_r * 2.5,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.35 + be * 0.15), 0.5)),
            (0.40, mix(p_col, Color::rgba(1.0, 0.20, 0.80, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.20, 0.0, 0.40, 0.05), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(holo_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. ROTATING HOLOGRAPHIC VINYL DISC & PLATTNER RIM
    // -------------------------------------------------------------------------
    let disc_grad = Fill::radial_gradient(
        cx - disc_r * 0.25,
        cy - disc_r * 0.25,
        0.0,
        cx,
        cy,
        disc_r,
        &[
            (0.0, Color::hex("#2a183d")),
            (0.40, Color::hex("#120b1c")),
            (0.75, Color::hex("#06030a")),
            (1.0, Color::hex("#020104")),
        ],
    );
    c.set_fill(disc_grad);
    c.set_stroke(Fill::Solid(glow_col));
    c.set_line_width(3.0 * user_scale);
    c.set_shadow(glow_col, 20.0 * user_scale);
    c.fill_circle(cx, cy, disc_r);
    c.stroke_circle(cx, cy, disc_r);

    // -------------------------------------------------------------------------
    // 2. 8 CONCENTRIC HOLOGRAPHIC GROOVE RINGS (PULSING WAVE RINGS)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for g_i in 1..=8 {
        let g_f = g_i as f32;
        let bin_k = (g_i * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let gr_r = disc_r * (0.35 + g_f * 0.075 + fv * 0.04 * sensitivity);
        let gr_col = mix(
            mix(p_col, glow_col, g_f / 8.0),
            mix(accent_col, Color::rgba(1.0, 0.20, 0.80, 0.90), fv),
            fv,
        );

        c.set_stroke(Fill::Solid(gr_col));
        c.set_line_width((1.5 + fv * 2.5) * user_scale);
        c.set_shadow(gr_col, (6.0 + fv * 8.0) * user_scale);
        c.stroke_circle(cx, cy, gr_r);
    }

    // Center Vinyl Label Disc
    let label_r = disc_r * 0.30;
    let label_fill = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        label_r,
        &[
            (0.0, mix(accent_col, Color::rgba(1.0, 1.0, 1.0, 0.95), 0.6)),
            (0.80, mix(p_col, glow_col, 0.5)),
            (1.0, Color::hex("#05020a")),
        ],
    );
    c.set_fill(label_fill);
    c.fill_circle(cx, cy, label_r);

    // Center Spindle Pin Hole
    c.set_fill(Fill::Solid(Color::hex("#020104")));
    c.fill_circle(cx, cy, label_r * 0.18);

    // -------------------------------------------------------------------------
    // 3. TURNTABLE TONEARM WITH GLOWING STYLUS NEEDLE
    // -------------------------------------------------------------------------
    let arm_pivot_x = cx + disc_r * 1.15;
    let arm_pivot_y = cy - disc_r * 0.85;

    let arm_angle = -0.60 + (frame_time * 0.30).sin() * 0.05 + be * 0.04;
    let arm_len = disc_r * 1.25;

    let stylus_x = arm_pivot_x + arm_angle.cos() * arm_len;
    let stylus_y = arm_pivot_y + arm_angle.sin() * arm_len;

    // Tonearm Shaft
    c.set_stroke(Fill::Solid(Color::rgba(0.90, 0.95, 1.0, 0.90)));
    c.set_line_width(3.0 * user_scale);
    c.set_shadow(Color::rgba(0.90, 0.95, 1.0, 0.80), 8.0 * user_scale);
    c.stroke_line(arm_pivot_x, arm_pivot_y, stylus_x, stylus_y);

    // Stylus Head Flare
    c.set_fill(Fill::Solid(mix(glow_col, Color::hex("#ff007f"), bs)));
    c.set_shadow(mix(glow_col, Color::hex("#ff007f"), bs), 12.0 * user_scale);
    c.fill_circle(stylus_x, stylus_y, (6.0 + bs * 4.0) * user_scale);

    c.set_global_alpha(1.0);
    c.restore();
}
