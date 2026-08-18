//! Cyber Horizon 3D style renderer (`cyberHorizon`) — Synthwave Horizon Grid Engine.
//!
//! Cinematic Masterpiece Synthwave Horizon:
//! - Receding 3D perspective synthwave grid floor stretching to the horizon line.
//! - Glowing gradient Synthwave Sun with horizontal laser cutouts sitting on the horizon.
//! - Smooth audio-reactive wireframe cyber mountain peaks contouring along the horizon.
//! - Floating starfield particles & horizon neon glow.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const HORIZON_MOUNTAIN_PTS: usize = 128;
const GRID_LINES_V: usize = 20;
const GRID_LINES_H: usize = 12;

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
    let cy = height * 0.55 + pos_offset_y; // Horizon line height

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Horizon Sunset Radial Glow
    let horizon_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.70 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(1.0, 0.0, 0.55, 0.35 + be * 0.20), 0.5)),
            (0.40, mix(p_col, Color::rgba(0.0, 0.85, 1.0, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.15, 0.0, 0.30, 0.04), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(horizon_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. SYNTHWAVE HORIZON SUN WITH SEGMENTED CUTOUTS
    // -------------------------------------------------------------------------
    let sun_r = 110.0 * user_scale;
    let sun_y = cy - sun_r * 0.30;

    let sun_grad = Fill::radial_gradient(
        cx,
        sun_y - sun_r * 0.4,
        0.0,
        cx,
        sun_y,
        sun_r,
        &[
            (0.0, Color::rgba(1.0, 0.95, 0.40, 0.98)),
            (0.40, mix(glow_col, Color::rgba(1.0, 0.20, 0.60, 0.90), 0.6)),
            (1.0, mix(p_col, Color::rgba(0.50, 0.0, 0.80, 0.85), 0.7)),
        ],
    );

    c.set_fill(sun_grad);
    c.set_shadow(glow_col, (24.0 + bs * 12.0) * user_scale);
    c.fill_circle(cx, sun_y, sun_r);

    // Sun Horizontal Cutout Lines (Vaporwave Sun Aesthetic)
    c.set_fill(Fill::Solid(Color::hex("#05020c")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    for line_i in 1..=6 {
        let l_y = sun_y + (line_i as f32 * 14.0 - 10.0) * user_scale;
        let l_h = (2.0 + line_i as f32 * 0.8) * user_scale;
        if l_y < sun_y + sun_r {
            c.fill_rect(cx - sun_r * 1.1, l_y, sun_r * 2.2, l_h);
        }
    }

    // -------------------------------------------------------------------------
    // 2. RECEDING 3D PERSPECTIVE SYNTHWAVE GRID FLOOR
    // -------------------------------------------------------------------------
    let grid_col = mix(Color::rgba(0.0, 0.85, 1.0, 0.60), glow_col, 0.5);
    c.set_stroke(Fill::Solid(grid_col));
    c.set_line_width(1.5 * user_scale);
    c.set_shadow(grid_col, 8.0 * user_scale);

    // Vertical Perspective Grid Lines
    for v in 0..=GRID_LINES_V {
        let v_t = v as f32 / GRID_LINES_V as f32;
        let x_top = cx + (v_t - 0.5) * (width * 0.25 * user_scale);
        let x_bot = cx + (v_t - 0.5) * (width * 1.40 * user_scale);

        c.stroke_line(x_top, cy, x_bot, height);
    }

    // Horizontal Scrolling Perspective Grid Lines
    let scroll_offset = (frame_time * 0.80) % 1.0;
    for h in 0..GRID_LINES_H {
        let h_t = ((h as f32 + scroll_offset) / GRID_LINES_H as f32).powf(2.0);
        let y_pos = cy + h_t * (height - cy);

        let grid_w_half = (width * 0.125 + h_t * width * 0.70) * user_scale;
        c.stroke_line(cx - grid_w_half, y_pos, cx + grid_w_half, y_pos);
    }

    // -------------------------------------------------------------------------
    // 3. AUDIO-REACTIVE CYBERPUNK MOUNTAIN HORIZON CONTOUR
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);
    let mountain_w = width * 0.90 * user_scale;
    let start_x = cx - mountain_w * 0.5;

    let mut mtn_pts: Vec<(f32, f32)> = Vec::with_capacity(HORIZON_MOUNTAIN_PTS);

    for i in 0..HORIZON_MOUNTAIN_PTS {
        let t = i as f32 / (HORIZON_MOUNTAIN_PTS - 1) as f32;
        let px = start_x + t * mountain_w;

        let bin_k = (i * step_f / (HORIZON_MOUNTAIN_PTS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let wave1 = (t * 8.0).sin() * 25.0 * user_scale;
        let wave2 = (t * 16.0 + frame_time * 1.5).cos() * (15.0 + fv * 75.0 * sensitivity) * user_scale;
        let py = cy - (wave1.abs() + wave2.abs() + be * 20.0 * user_scale);

        mtn_pts.push((px, py));
    }

    let mtn_col = mix(p_col, accent_col, 0.7);
    c.set_stroke(Fill::Solid(mtn_col));
    c.set_line_width(2.5 * user_scale);
    c.set_shadow(mtn_col, (12.0 + bs * 8.0) * user_scale);
    c.stroke_polyline(&mtn_pts);

    c.set_global_alpha(1.0);
    c.restore();
}
