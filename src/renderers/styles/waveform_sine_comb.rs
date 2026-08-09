//! Waveform Sine Comb style renderer (`waveformSineComb`).
//!
//! Elastic Spring Comb Physics:
//! - Vertical comb needles bridging upper & lower waveform boundaries.
//! - Needles vibrate & rebound with spring physics under audio impact.

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

    let span_w = width * 0.88 * user_scale;
    let start_x = cx - span_w * 0.5;
    let max_h = height * 0.38 * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Background
//     c.set_fill(Fill::Solid(Color::hex("#020108")));
//     c.fill_rect(0.0, 0.0, width, height);

    let bg_glow = Fill::radial_gradient(
        cx, cy, 0.0, cx, cy, width * 0.65,
        &[
            (0.00, mix(glow, Color::rgba(1.0, 0.4, 0.0, 0.3), 0.5).with_alpha(0.25 + be * 0.15)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    let comb_count = 64usize;
    let step_p = (pcm.len() / comb_count).max(1);

    for i in 0..comb_count {
        let t = i as f32 / (comb_count - 1) as f32;
        let x = start_x + t * span_w;

        let sample_idx = (i * step_p).min(pcm.len().saturating_sub(1));
        let val = (pcm[sample_idx] as f32 / 128.0 - 1.0) * sensitivity;

        // Spring rebound oscillation physics
        let spring = (frame_time * 8.0 + t * 12.0).sin() * 4.0 * user_scale * val.abs();
        let h = (val.abs() * max_h + spring + 6.0 * user_scale).clamp(6.0, max_h);

        let needle_top = cy - h;
        let needle_bot = cy + h;

        let needle_col = mix(
            mix(p_col, acc_col, t),
            mix(glow, Color::rgba(1.0, 0.95, 0.3, 1.0), val.abs()),
            val.abs(),
        );

        // Comb Needle Body
        c.set_stroke(Fill::Solid(needle_col.with_alpha(0.85)));
        c.set_line_width((1.5 + val.abs() * 2.0) * user_scale);
        c.set_shadow(needle_col, (6.0 + val.abs() * 8.0) * user_scale);
        c.stroke_line(x, needle_top, x, needle_bot);

        // Glowing Needle Tips (Top & Bottom Caps)
        let cap_col = mix(needle_col, Color::rgba(1.0, 1.0, 1.0, 0.98), bs);
        c.set_fill(Fill::Solid(cap_col));
        c.set_shadow(cap_col, 8.0 * user_scale);
        c.fill_circle(x, needle_top, (3.0 + val.abs() * 2.5) * user_scale);
        c.fill_circle(x, needle_bot, (3.0 + val.abs() * 2.5) * user_scale);
    }

    let _ = (acc_col, s_col);

    c.set_global_alpha(1.0);
    c.restore();
}
