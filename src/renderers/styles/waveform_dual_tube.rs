//! Waveform Dual Tube style renderer (`waveformDualTube`).
//!
//! Glowing Twin Neon Cathode Glass Tubes:
//! - Two glowing gas-filled glass tubes bending dynamically to positive & negative PCM waveform peaks.
//! - Glass specular highlight reflections on tube walls.
//! - Audio-reactive spark filaments bridging the twin tubes.

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

    let span_w = width * 0.90 * user_scale;
    let start_x = cx - span_w * 0.5;
    let max_amp = height * 0.35 * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Background
//     c.set_fill(Fill::Solid(Color::hex("#020207")));
//     c.fill_rect(0.0, 0.0, width, height);

    // Ambient Tube Chamber Glow
    let chamber_glow = Fill::radial_gradient(
        cx, cy, 0.0, cx, cy, width * 0.60,
        &[
            (0.00, mix(glow, Color::rgba(0.0, 0.85, 1.0, 0.25), 0.5).with_alpha(0.22 + be * 0.15)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(chamber_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    let samples = pcm.len().min(180);
    let mut upper_tube: Vec<(f32, f32)> = Vec::with_capacity(samples);
    let mut lower_tube: Vec<(f32, f32)> = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / (samples - 1) as f32;
        let x = start_x + t * span_w;

        let val = (pcm[i] as f32 / 128.0 - 1.0) * sensitivity;
        let pos_val = val.max(0.0);
        let neg_val = (-val).max(0.0);

        let u_y = cy - (15.0 * user_scale + pos_val * max_amp);
        let l_y = cy + (15.0 * user_scale + neg_val * max_amp);

        upper_tube.push((x, u_y));
        lower_tube.push((x, l_y));
    }

    if upper_tube.len() >= 2 {
        // 1. Bridging Spark Filaments between Twin Tubes
        c.set_shadow(Color::TRANSPARENT, 0.0);
        for i in (0..samples).step_by(4) {
            let (ux, uy) = upper_tube[i];
            let (_, ly)  = lower_tube[i];
            let f_t = i as f32 / samples as f32;

            let spark_col = mix(acc_col, Color::rgba(1.0, 0.85, 0.2, 0.8), (f_t + frame_time * 0.5).fract());
            c.set_stroke(Fill::Solid(spark_col.with_alpha(0.35 + bs * 0.25)));
            c.set_line_width(1.0 * user_scale);
            c.stroke_line(ux, uy, ux, ly);
        }

        // 2. Upper Tube (Primary Theme Tint)
        let tube_upper = mix(p_col, Color::rgba(0.0, 0.92, 1.0, 0.95), 0.4);

        // Outer Gas Glow
        c.set_stroke(Fill::Solid(tube_upper.with_alpha(0.25 + be * 0.10)));
        c.set_line_width(16.0 * user_scale);
        c.set_shadow(tube_upper, (18.0 + be * 10.0) * user_scale);
        c.stroke_polyline(&upper_tube);

        // Glass Tube Wall
        c.set_stroke(Fill::Solid(tube_upper));
        c.set_line_width(6.0 * user_scale);
        c.set_shadow(tube_upper, 10.0 * user_scale);
        c.stroke_polyline(&upper_tube);

        // Inner White Core Stream
        c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.98)));
        c.set_line_width(1.8 * user_scale);
        c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.90), 6.0 * user_scale);
        c.stroke_polyline(&upper_tube);

        // 3. Lower Tube (Accent Theme Tint)
        let tube_lower = mix(acc_col, Color::rgba(1.0, 0.35, 0.15, 0.95), 0.4);

        // Outer Gas Glow
        c.set_stroke(Fill::Solid(tube_lower.with_alpha(0.25 + be * 0.10)));
        c.set_line_width(16.0 * user_scale);
        c.set_shadow(tube_lower, (18.0 + be * 10.0) * user_scale);
        c.stroke_polyline(&lower_tube);

        // Glass Tube Wall
        c.set_stroke(Fill::Solid(tube_lower));
        c.set_line_width(6.0 * user_scale);
        c.set_shadow(tube_lower, 10.0 * user_scale);
        c.stroke_polyline(&lower_tube);

        // Inner White Core Stream
        c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.98)));
        c.set_line_width(1.8 * user_scale);
        c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.90), 6.0 * user_scale);
        c.stroke_polyline(&lower_tube);
    }

    let _ = (p_col, s_col);

    c.set_global_alpha(1.0);
    c.restore();
}
