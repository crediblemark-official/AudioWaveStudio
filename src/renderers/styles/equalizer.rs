//! Equalizer style renderer (`equalizer`) — Neon Spectrum Equalizer.
//!
//! One cohesive design:
//! - Smooth gradient vertical bars spanning full width, rising with frequency.
//! - Per-bar peak-hold dot that lingers then falls.
//! - Mirror floor reflection fading below the baseline.
//! - Subtle vertical scanlines for depth and texture.
//! - All elements share the same gradient color language from primary → accent.

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
    let bar_count    = ctx.config.reactivity.bar_count.clamp(8, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq      = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx      = width  * 0.5 + pos_offset_x;
    // Baseline: where bars sit — slightly below centre
    let base_y  = height * 0.72 + pos_offset_y;
    // Maximum bar height (upward from baseline)
    let max_h   = height * 0.62 * user_scale;

    let n_bars  = bar_count;
    let step_f  = (freq.len() / n_bars).max(1);

    // Bar geometry
    let total_w  = width * 0.92 * user_scale;
    let slot_w   = total_w / n_bars as f32;
    let gap      = (slot_w * 0.18).max(1.0);
    let b_w      = (slot_w - gap).max(1.0);
    let start_x  = cx - total_w * 0.5;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. BACKGROUND — dark void + bass-reactive radial bloom
    // -------------------------------------------------------------------------
//     c.set_fill(Fill::Solid(Color::hex("#020308")));
//     c.fill_rect(0.0, 0.0, width, height);

    let bloom = Fill::radial_gradient(
        cx, base_y, 0.0,
        cx, base_y, width * 0.65,
        &[
            (0.00, mix(glow, p_col, 0.4).with_alpha(0.20 + be * 0.16)),
            (0.40, mix(p_col, acc_col, 0.5).with_alpha(0.08 + be * 0.06)),
            (0.80, s_col.with_alpha(0.03)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bloom);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 2. BASELINE GLOW — horizontal neon line where bars rest
    // -------------------------------------------------------------------------
    let baseline_col = mix(p_col, acc_col, 0.3 + be * 0.2);
    let baseline_grad = Fill::linear_gradient(
        start_x, base_y, start_x + total_w, base_y,
        &[
            (0.00, Color::TRANSPARENT),
            (0.10, baseline_col.with_alpha(0.55 + be * 0.20)),
            (0.50, baseline_col.with_alpha(0.90 + be * 0.08)),
            (0.90, baseline_col.with_alpha(0.55 + be * 0.20)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_stroke(baseline_grad);
    c.set_line_width((1.4 + be * 1.2) * user_scale);
    c.set_shadow(baseline_col, (8.0 + be * 6.0) * user_scale);
    c.stroke_line(start_x, base_y, start_x + total_w, base_y);

    // -------------------------------------------------------------------------
    // 3. BARS + REFLECTIONS + PEAKS
    // -------------------------------------------------------------------------
    for i in 0..n_bars {
        let i_f = i as f32;
        let t   = if n_bars > 1 { i_f / (n_bars - 1) as f32 } else { 0.0 }; // 0..1 across width

        let bin_k = (i * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let x = start_x + i_f * slot_w;

        // Bar height: frequency + bass boost
        let bar_h = (fv * max_h * sensitivity + be * max_h * 0.10)
            .clamp(2.0 * user_scale, max_h);
        let top_y = base_y - bar_h;

        // Color shifts left→right across frequency range: p_col → acc_col → glow
        let bar_col_bot = mix(p_col,   acc_col, t);
        let bar_col_top = mix(acc_col, glow,    t);

        // 3a. BAR FILL — vertical gradient, bright top fades to darker bottom
        let bar_fill = Fill::linear_gradient(
            x, top_y, x, base_y,
            &[
                (0.00, mix(bar_col_top, Color::rgba(1.0, 1.0, 1.0, 1.0), 0.25 + fv * 0.15)),
                (0.25, bar_col_top.with_alpha(0.98)),
                (0.65, bar_col_bot.with_alpha(0.92)),
                (1.00, bar_col_bot.with_alpha(0.70)),
            ],
        );
        c.set_fill(bar_fill);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_rect(x, top_y, b_w, bar_h);

        // 3b. BAR GLOW — subtle shadow bloom from each bar
        c.set_shadow(mix(bar_col_top, glow, 0.4), (6.0 + fv * 8.0) * user_scale);
        c.fill_rect(x, top_y, b_w, bar_h.min(4.0 * user_scale));

        // 3c. TOP EDGE HIGHLIGHT — thin bright cap line
        c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.75 + fv * 0.20)));
        c.set_shadow(mix(bar_col_top, Color::rgba(1.0, 1.0, 1.0, 1.0), 0.5), 6.0 * user_scale);
        c.fill_rect(x, top_y, b_w, (1.5 * user_scale).max(1.0));

        // 3d. PEAK HOLD DOT
        // Simulate peak hold using a slow sine decay from max (no persistent state available)
        let peak_decay = (frame_time * 0.4 + i_f * 0.08).sin() * 0.06 + 0.94;
        let peak_h = bar_h * peak_decay * (1.0 + bs * 0.12);
        let peak_y = base_y - peak_h.clamp(bar_h, max_h * 1.05);
        let pk_col = mix(bar_col_top, Color::rgba(1.0, 1.0, 1.0, 0.95), 0.6);
        c.set_fill(Fill::Solid(pk_col));
        c.set_shadow(pk_col, (8.0 + fv * 6.0) * user_scale);
        c.fill_rect(x, peak_y, b_w, (2.5 * user_scale).max(2.0));

        // 3e. FLOOR REFLECTION — mirrored bar below baseline, fades out quickly
        let refl_h = bar_h * 0.32;
        let refl_fill = Fill::linear_gradient(
            x, base_y, x, base_y + refl_h,
            &[
                (0.00, bar_col_bot.with_alpha(0.35 + fv * 0.10)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(refl_fill);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_rect(x, base_y, b_w, refl_h);
    }

    // -------------------------------------------------------------------------
    // 4. SCANLINES — subtle horizontal bands for CRT/LED panel texture
    //    Drawn as very thin semi-transparent dark lines across the bar area
    // -------------------------------------------------------------------------
    let scanline_alpha = 0.07;
    let scanline_step  = (3.5 * user_scale).max(2.0);
    let mut sy = 0.0f32;
    while sy < height {
        c.set_fill(Fill::Solid(Color::rgba(0.0, 0.0, 0.0, scanline_alpha)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_rect(0.0, sy, width, (0.9 * user_scale).max(0.8));
        sy += scanline_step;
    }

    // suppress unused
    let _ = (s_col, frame_time);

    c.set_global_alpha(1.0);
    c.restore();
}
