//! Hyper-Speed Warp Drive style renderer (`warpDrive`) — Hyperspace Tunnel Engine.
//!
//! Features:
//! - Hyperspace light-speed tunnel perspective extending into 3D depth.
//! - Radial frequency streak bars accelerating outward towards the viewer.
//! - 60+ starfield streaks bursting with bass energy.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const TUNNEL_BARS: usize = 48;
const STAR_STREAKS: usize = 60;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let _p = theme_primary(theme);
    let _s = theme_secondary(theme);
    let _accent = theme_accent(theme);
    let _glow = theme_glow(theme);

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
    let base_r = 25.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep pitch-black hyperspace backdrop
    c.set_fill(Fill::Solid(Color::hex("#010206")));
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. STARFIELD ACCELERATION STREAKS
    // -------------------------------------------------------------------------
    for s_i in 0..STAR_STREAKS {
        let speed = 0.45 + (s_i % 5) as f32 * 0.15 + be * 0.5;
        let s_t = ((frame_time * speed + s_i as f32 * 0.07) % 1.0).clamp(0.0, 1.0);

        let angle = (s_i as f32 / STAR_STREAKS as f32) * TAU;
        let (sin_a, cos_a) = angle.sin_cos();

        let r0 = base_r + s_t.powi(2) * (width * 0.65 * user_scale);
        let r1 = r0 + (10.0 + s_t * 50.0 + be * 30.0) * user_scale;

        let x0 = cx + cos_a * r0;
        let y0 = cy + sin_a * r0;
        let x1 = cx + cos_a * r1;
        let y1 = cy + sin_a * r1;

        let alpha = (s_t * 0.95).clamp(0.1, 0.95);
        let streak_col = mix(
            Color::rgba(0.0, 0.90, 1.0, alpha),
            Color::rgba(1.0, 0.20, 0.80, alpha),
            (s_i % 3) as f32 / 3.0,
        );

        c.set_stroke(Fill::Solid(streak_col));
        c.set_line_width((1.0 + s_t * 3.5) * user_scale);
        c.stroke_line(x0, y0, x1, y1);
    }

    // -------------------------------------------------------------------------
    // 2. RADIAL TUNNEL FREQUENCY BARS (PERSPECTIVE PROJECTION)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for i in 0..TUNNEL_BARS {
        let angle = (i as f32 / TUNNEL_BARS as f32) * TAU + frame_time * 0.05;
        let bin_k = (i * step_f / (TUNNEL_BARS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let bar_len = (30.0 + fv * 200.0 * sensitivity + be * 60.0) * user_scale;
        let r0 = base_r * (1.0 + be * 0.15);
        let r1 = r0 + bar_len;

        let (sin_a, cos_a) = angle.sin_cos();

        let x0 = cx + cos_a * r0;
        let y0 = cy + sin_a * r0;
        let x1 = cx + cos_a * r1;
        let y1 = cy + sin_a * r1;

        let w0 = 3.0 * user_scale;
        let w1 = (8.0 + fv * 10.0) * user_scale;

        let px0 = -sin_a * (w0 * 0.5);
        let py0 = cos_a * (w0 * 0.5);
        let px1 = -sin_a * (w1 * 0.5);
        let py1 = cos_a * (w1 * 0.5);

        let pts = vec![
            (x0 - px0, y0 - py0),
            (x0 + px0, y0 + py0),
            (x1 + px1, y1 + py1),
            (x1 - px1, y1 - py1),
        ];

        let bar_grad = Fill::linear_gradient(
            x0,
            y0,
            x1,
            y1,
            &[
                (0.0, Color::rgba(0.0, 0.95, 1.0, 0.95)),
                (0.60, Color::rgba(0.20, 0.40, 1.0, 0.80)),
                (1.0, Color::rgba(0.90, 0.10, 0.90, 0.20)),
            ],
        );

        c.set_fill(bar_grad);
        c.fill_polygon(&pts);
    }

    // -------------------------------------------------------------------------
    // 3. HYPERSPACE FOCUS SINGULARITY CORE
    // -------------------------------------------------------------------------
    let sing_r = base_r * (0.80 + be * 0.25 + bs * 0.10);
    c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.95)));
    c.set_shadow(Color::rgba(0.0, 0.90, 1.0, 1.0), 20.0 * user_scale);
    c.fill_circle(cx, cy, sing_r);

    c.set_global_alpha(1.0);
    c.restore();
}
