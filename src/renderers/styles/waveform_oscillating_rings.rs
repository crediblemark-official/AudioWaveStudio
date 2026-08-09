//! Waveform Oscillating Rings style renderer (`waveformOscillatingRings`).
//!
//! Interlocking Concentric Waveform Rings:
//! - 3 nested concentric circular rings distorted by time-domain audio waveform data at phase offsets.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const RING_SAMPLES: usize = 90;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col   = theme_primary(theme);
    let s_col   = theme_secondary(theme);
    let acc_col = theme_accent(theme);
    let glow    = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width  * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let pcm = ctx.time_data;
    let frame_time = ctx.frame_time;

    let cx = width  * 0.5 + pos_offset_x;
    let cy = height * 0.50 + pos_offset_y;

    let base_radius = width.min(height) * 0.22 * user_scale;
    let step_p = (pcm.len() / RING_SAMPLES).max(1);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Background
//     c.set_fill(Fill::Solid(Color::hex("#020108")));
//     c.fill_rect(0.0, 0.0, width, height);

    let bg_glow = Fill::radial_gradient(
        cx, cy, 0.0, cx, cy, base_radius * 2.2,
        &[
            (0.00, mix(glow, Color::rgba(0.0, 0.9, 0.8, 0.3), 0.5).with_alpha(0.25 + be * 0.15)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // 3 Nested Concentric Waveform Rings
    let ring_scales = [0.65, 1.0, 1.35];
    let ring_colors = [p_col, acc_col, glow];

    for r_idx in 0..3usize {
        let r_scale = ring_scales[r_idx];
        let r_col   = ring_colors[r_idx];
        let phase   = frame_time * (0.4 + r_idx as f32 * 0.2) + r_idx as f32 * 1.57;

        let mut ring_pts: Vec<(f32, f32)> = Vec::with_capacity(RING_SAMPLES + 1);

        for i in 0..=RING_SAMPLES {
            let t = (i % RING_SAMPLES) as f32 / RING_SAMPLES as f32;
            let angle = t * TAU + phase;

            let sample_idx = ((i % RING_SAMPLES) * step_p).min(pcm.len().saturating_sub(1));
            let val = (pcm[sample_idx] as f32 / 128.0 - 1.0) * sensitivity;

            let r_curr = base_radius * r_scale + val * (35.0 * user_scale);
            let rx = cx + r_curr * angle.cos();
            let ry = cy + r_curr * angle.sin() * 0.70; // Slightly tilted ellipse

            ring_pts.push((rx, ry));
        }

        if ring_pts.len() >= 2 {
            // Outer Glow
            c.set_stroke(Fill::Solid(r_col.with_alpha(0.25 + be * 0.10)));
            c.set_line_width((10.0 + be * 6.0) * user_scale);
            c.set_shadow(r_col, (14.0 + bs * 8.0) * user_scale);
            c.stroke_polyline(&ring_pts);

            // Core Line
            c.set_stroke(Fill::Solid(mix(r_col, Color::rgba(1.0, 1.0, 1.0, 0.98), 0.4)));
            c.set_line_width((2.0 + (r_idx as f32) * 0.4) * user_scale);
            c.set_shadow(r_col, 6.0 * user_scale);
            c.stroke_polyline(&ring_pts);
        }
    }

    let _ = (s_col, be);

    c.set_global_alpha(1.0);
    c.restore();
}
