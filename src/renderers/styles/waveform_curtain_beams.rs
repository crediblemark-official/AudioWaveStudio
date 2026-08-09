//! Waveform Curtain Beams style renderer (`waveformCurtainBeams`).
//!
//! Volumetric Laser Curtain Waveform:
//! - Vertical laser light beams shot downward from the waveform amplitude curve,
//!   creating a dense volumetric curtain wall of light.

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
    let pcm = ctx.time_data;

    let cx = width  * 0.5 + pos_offset_x;
    let cy = height * 0.50 + pos_offset_y;

    let span_w = width * 0.88 * user_scale;
    let start_x = cx - span_w * 0.5;
    let max_amp = height * 0.36 * user_scale;

    let beam_count = 56usize;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Background
//     c.set_fill(Fill::Solid(Color::hex("#020108")));
//     c.fill_rect(0.0, 0.0, width, height);

    let bg_glow = Fill::radial_gradient(
        cx, cy, 0.0, cx, cy, width * 0.65,
        &[
            (0.00, mix(glow, Color::rgba(0.0, 0.85, 1.0, 0.3), 0.5).with_alpha(0.22 + be * 0.15)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    let sample_step = (pcm.len() / beam_count).max(1);
    let beam_w = span_w / beam_count as f32;

    for i in 0..beam_count {
        let t = i as f32 / beam_count as f32;
        let x = start_x + i as f32 * beam_w;

        let sample_idx = (i * sample_step).min(pcm.len().saturating_sub(1));
        let val = (pcm[sample_idx] as f32 / 128.0 - 1.0) * sensitivity;

        let top_y = cy - val.abs() * max_amp;
        let bot_y = cy + max_amp * 0.8; // Curtain extends down to floor

        let beam_col = mix(
            mix(p_col, acc_col, t),
            mix(glow, Color::rgba(0.0, 0.95, 1.0, 0.9), val.abs()),
            val.abs(),
        );

        // Volumetric Beam Gradient (bright top at waveform curve, fading down)
        let curtain_grad = Fill::linear_gradient(
            x, top_y, x, bot_y,
            &[
                (0.00, beam_col.with_alpha(0.85)),
                (0.30, beam_col.with_alpha(0.40)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(curtain_grad);
        c.set_shadow(beam_col, (6.0 + val.abs() * 8.0) * user_scale);
        c.fill_rect(x, top_y, beam_w * 0.8, bot_y - top_y);

        // Bright Emitter Node Cap at Waveform Height
        c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.95)));
        c.set_shadow(beam_col, 8.0 * user_scale);
        c.fill_circle(x + beam_w * 0.4, top_y, 3.0 * user_scale);
    }

    let _ = (s_col, acc_col);

    c.set_global_alpha(1.0);
    c.restore();
}
