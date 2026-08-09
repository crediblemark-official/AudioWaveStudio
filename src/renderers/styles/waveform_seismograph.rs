//! Waveform Seismograph style renderer (`waveformSeismograph`).
//!
//! 3D Paper Roll Seismograph:
//! - Tilted scrolling 3D medical / seismic grid paper roll.
//! - Mechanical needle pen scratching real-time audio waveform.
//! - Glowing needle tip sparks and fading paper ink trail.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

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

    // Seismograph paper bed geometry
    let paper_w = width * 0.88 * user_scale;
    let paper_h = height * 0.55 * user_scale;
    let left_x  = cx - paper_w * 0.5;
    let right_x = cx + paper_w * 0.5;
    let top_y   = cy - paper_h * 0.5;
    let bot_y   = cy + paper_h * 0.5;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Background
//     c.set_fill(Fill::Solid(Color::hex("#030208")));
//     c.fill_rect(0.0, 0.0, width, height);

    // 1. Paper Bed Fill — dark charcoal grid paper
    let paper_fill = Fill::linear_gradient(
        cx, top_y, cx, bot_y,
        &[
            (0.00, Color::rgba(0.06, 0.08, 0.12, 0.98)),
            (0.50, Color::rgba(0.09, 0.11, 0.16, 0.98)),
            (1.00, Color::rgba(0.05, 0.07, 0.10, 0.98)),
        ],
    );
    c.set_fill(paper_fill);
    c.fill_rect(left_x, top_y, paper_w, paper_h);

    // 2. Paper Grid Lines (Seismograph medical chart lines)
    // Horizontal grid lines
    let h_lines = 12usize;
    for j in 0..=h_lines {
        let ty = top_y + j as f32 / h_lines as f32 * paper_h;
        let is_center = j == h_lines / 2;
        let lc = if is_center {
            Color::rgba(0.8, 0.2, 0.3, 0.55) // Center red reference line
        } else {
            Color::rgba(0.2, 0.35, 0.45, 0.22)
        };
        c.set_stroke(Fill::Solid(lc));
        c.set_line_width(if is_center { 1.5 * user_scale } else { 0.8 * user_scale });
        c.stroke_line(left_x, ty, right_x, ty);
    }

    // Vertical scrolling grid lines
    let v_lines = 20usize;
    let scroll_x = (frame_time * 40.0) % (paper_w / v_lines as f32);
    for i in 0..=v_lines + 1 {
        let tx = left_x + i as f32 * (paper_w / v_lines as f32) - scroll_x;
        if tx >= left_x && tx <= right_x {
            c.set_stroke(Fill::Solid(Color::rgba(0.2, 0.35, 0.45, 0.18)));
            c.set_line_width(0.8 * user_scale);
            c.stroke_line(tx, top_y, tx, bot_y);
        }
    }

    // Paper Frame Border
    c.set_stroke(Fill::Solid(mix(p_col, Color::rgba(0.4, 0.6, 0.8, 0.8), 0.5)));
    c.set_line_width(2.0 * user_scale);
    c.set_shadow(glow, 10.0 * user_scale);
    c.stroke_rect(left_x, top_y, paper_w, paper_h);

    // 3. SEISMOGRAPH WAVEFORM TRACE
    let samples = pcm.len().min(256);
    let mut trace_pts: Vec<(f32, f32)> = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / (samples - 1) as f32;
        let x = left_x + t * paper_w;

        let sample = pcm[i] as f32 / 128.0 - 1.0;
        let amp_y = sample * paper_h * 0.40 * sensitivity;

        let y = (cy + amp_y).clamp(top_y + 4.0, bot_y - 4.0);
        trace_pts.push((x, y));
    }

    if trace_pts.len() >= 2 {
        // Outer Seismograph Glow
        let trace_col = mix(acc_col, Color::rgba(0.0, 0.95, 0.70, 0.95), 0.6);
        c.set_stroke(Fill::Solid(trace_col.with_alpha(0.35)));
        c.set_line_width(12.0 * user_scale);
        c.set_shadow(trace_col, 16.0 * user_scale);
        c.stroke_polyline(&trace_pts);

        // Core Ink Line
        c.set_stroke(Fill::Solid(trace_col));
        c.set_line_width(2.2 * user_scale);
        c.set_shadow(trace_col, 8.0 * user_scale);
        c.stroke_polyline(&trace_pts);
    }

    // 4. MECHANICAL NEEDLE PEN (At current right-most reading tip)
    if let Some(&(needle_x, needle_y)) = trace_pts.last() {
        // Needle Arm
        let arm_top_x = needle_x + 30.0 * user_scale;
        let arm_top_y = top_y - 40.0 * user_scale;

        c.set_stroke(Fill::Solid(Color::rgba(0.85, 0.90, 0.95, 0.90)));
        c.set_line_width(2.5 * user_scale);
        c.set_shadow(glow, 6.0 * user_scale);
        c.stroke_line(arm_top_x, arm_top_y, needle_x, needle_y);

        // Needle Spark Flare at Tip
        let spark_col = mix(Color::rgba(1.0, 0.9, 0.2, 0.98), Color::rgba(1.0, 1.0, 1.0, 0.98), bs);
        c.set_fill(Fill::Solid(spark_col));
        c.set_shadow(spark_col, (14.0 + bs * 10.0) * user_scale);
        c.fill_circle(needle_x, needle_y, (4.5 + bs * 3.0) * user_scale);
    }

    let _ = (s_col, be);

    c.set_global_alpha(1.0);
    c.restore();
}
