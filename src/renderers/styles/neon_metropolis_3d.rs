//! Neon Cyberpunk Metropolis 3D style renderer (`neonMetropolis3D`).
//!
//! Rewritten for realism & smoothness:
//! - Multi-depth 2D layered skyline with proper perspective scaling.
//! - Per-building neon roof glows, window grid flicker, and antenna spires.
//! - Rain streaks and puddle reflections at street level.
//! - Smooth horizon city-light pollution gradient.
//! - Audio-reactive building heights with per-bin frequency sampling.

use std::f32::consts::PI;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

// Number of buildings per depth layer and layer count
const LAYERS: usize = 4;
const BLDS_PER_LAYER: usize = 18;
const RAIN_DROPS: usize = 60;

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
    let bar_count    = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq      = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx       = width  * 0.5 + pos_offset_x;
    // Horizon sits at ~55% down the screen — city below, sky above
    let horizon  = height * 0.55 + pos_offset_y;
    let step_f   = (freq.len() / bar_count).max(1);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. SKY — deep midnight gradient with city light pollution on horizon
    // -------------------------------------------------------------------------
    let sky_grad = Fill::linear_gradient(
        cx, 0.0, cx, horizon,
        &[
            (0.00, Color::hex("#020408")),
            (0.60, Color::hex("#060b16")),
            (1.00, Color::hex("#0c1428")),
        ],
    );
    c.set_fill(sky_grad);
    c.fill_rect(0.0, 0.0, width, horizon + 2.0);

    // City light pollution glow on horizon
    let haze_glow = Fill::radial_gradient(
        cx, horizon, 0.0,
        cx, horizon, width * 0.70,
        &[
            (0.00, mix(p_col, Color::rgba(0.40, 0.10, 0.80, 0.35 + be * 0.18), 0.5)),
            (0.35, mix(acc_col, Color::rgba(0.0, 0.60, 1.0, 0.18), 0.5)),
            (0.70, mix(s_col, Color::rgba(0.05, 0.08, 0.25, 0.06), 0.5)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(haze_glow);
    c.fill_rect(0.0, horizon - height * 0.45, width, height * 0.50);

    // -------------------------------------------------------------------------
    // 2. GROUND — wet asphalt with neon puddle reflections
    // -------------------------------------------------------------------------
    let street_grad = Fill::linear_gradient(
        cx, horizon, cx, height,
        &[
            (0.00, Color::rgba(0.05, 0.06, 0.12, 1.0)),
            (0.40, Color::rgba(0.03, 0.04, 0.08, 1.0)),
            (1.00, Color::rgba(0.01, 0.02, 0.05, 1.0)),
        ],
    );
    c.set_fill(street_grad);
    c.fill_rect(0.0, horizon, width, height - horizon);

    // Wet street center-line reflection (mirrors the city glow)
    let reflect_glow = Fill::radial_gradient(
        cx, horizon + 20.0, 0.0,
        cx, horizon + 20.0, width * 0.55,
        &[
            (0.00, mix(p_col, Color::rgba(0.5, 0.1, 1.0, 0.22 + be * 0.12), 0.55)),
            (0.50, mix(acc_col, Color::rgba(0.0, 0.5, 1.0, 0.10), 0.5)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(reflect_glow);
    c.fill_rect(0.0, horizon, width, height * 0.28);

    // -------------------------------------------------------------------------
    // 3. SKYLINE BUILDINGS — 4 depth layers, back-to-front
    //    Each layer is scaled smaller for distant buildings (perspective).
    //    Buildings are drawn as 2D filled rects with neon roof/window detail.
    // -------------------------------------------------------------------------
    for layer in 0..LAYERS {
        let l = layer as f32 / (LAYERS - 1) as f32; // 0=back, 1=front

        // Perspective: back layers are smaller, closer to horizon
        let depth_scale = (0.30 + l * 0.70) * user_scale;
        let layer_base  = horizon - 2.0; // all buildings sit on horizon
        let max_h       = height * (0.25 + l * 0.30) * depth_scale;
        let bld_w_base  = width * 0.038 * depth_scale;
        let spacing     = width * 0.052 * depth_scale;
        let total_span  = BLDS_PER_LAYER as f32 * spacing;
        let start_x     = cx - total_span * 0.5;

        // Slight pan with slow camera drift + bass push
        let pan = (frame_time * (0.04 + l * 0.02)).sin() * 12.0 * l + be * 6.0 * l;

        // Darkness: back = very dark, front = slightly brighter
        let dark_t = 0.08 + l * 0.12;

        for b in 0..BLDS_PER_LAYER {
            let b_f = b as f32;

            // Vary width per building (pseudo-random from index)
            let width_var = 0.7 + ((b * 7 + layer * 3) % 10) as f32 * 0.06;
            let bld_w     = bld_w_base * width_var;

            let bx = start_x + b_f * spacing + pan;

            // Sample frequency
            let bin_k = (b * step_f / ((BLDS_PER_LAYER / bar_count.max(1)).max(1)))
                .min(freq.len().saturating_sub(1));
            let fv = freq[bin_k] as f32 / 255.0;

            // Building height: frequency + bass + layer-based minimum
            let min_h = max_h * (0.15 + ((b * 3 + layer) % 7) as f32 * 0.08);
            let bld_h = (min_h + fv * max_h * sensitivity + be * max_h * 0.12)
                .clamp(min_h * 0.8, max_h * 1.35);

            let top_y = layer_base - bld_h;

            // Building body color — dark base with neon tint
            let body_col = mix(
                Color::rgba(dark_t, dark_t + 0.01, dark_t + 0.03, 1.0),
                mix(
                    mix(p_col, s_col, b_f / BLDS_PER_LAYER as f32),
                    mix(acc_col, glow, fv),
                    fv,
                ),
                0.18 + fv * 0.25 + l * 0.08,
            );
            c.set_fill(Fill::Solid(body_col));
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.fill_rect(bx, top_y, bld_w, bld_h);

            // -- WINDOW GRID GLOW (flicker slightly with audio) --
            if bld_h > 25.0 && l > 0.3 {
                let win_rows = ((bld_h / 12.0) as usize).clamp(2, 18);
                let win_cols = ((bld_w  / 10.0) as usize).clamp(1, 5);
                let win_h = bld_h / (win_rows as f32 + 1.0);
                let win_w = bld_w  / (win_cols as f32 + 0.5);

                for wr in 0..win_rows {
                    for wc in 0..win_cols {
                        // Some windows lit, some dark (pseudo-random)
                        let lit = ((wr * 5 + wc * 3 + b + layer) % 4) != 0;
                        if !lit { continue; }
                        let flicker = ((frame_time * (2.0 + (wr + wc) as f32 * 0.3)
                            + b_f * 0.5).sin() * 0.10 + 0.90).clamp(0.0, 1.0);

                        let wx = bx + wc as f32 * win_w + win_w * 0.18;
                        let wy = top_y + wr as f32 * win_h + win_h * 0.22;
                        let ww = win_w * 0.55;
                        let wh = win_h * 0.55;

                        let win_col = mix(
                            Color::rgba(1.0, 0.92, 0.70, 0.55 * flicker), // warm
                            mix(glow, acc_col, fv),                         // neon
                            fv * 0.6,
                        );
                        c.set_fill(Fill::Solid(win_col));
                        c.set_shadow(win_col, 3.0);
                        c.fill_rect(wx, wy, ww.max(1.5), wh.max(1.5));
                    }
                }
            }

            // -- NEON ROOF GLOW --
            let neon_col = mix(
                mix(p_col, acc_col, (b % 3) as f32 / 2.0),
                mix(glow, s_col, fv),
                0.5 + fv * 0.3,
            );
            let neon_r = (bld_w * 0.6 + fv * bld_w * 0.4 + be * 8.0 * l).clamp(2.0, 40.0);
            let roof_glow = Fill::radial_gradient(
                bx + bld_w * 0.5, top_y, 0.0,
                bx + bld_w * 0.5, top_y, neon_r * 2.5,
                &[
                    (0.00, neon_col.with_alpha(0.70 + fv * 0.25 + bs * 0.10)),
                    (0.50, neon_col.with_alpha(0.20)),
                    (1.00, Color::TRANSPARENT),
                ],
            );
            c.set_fill(roof_glow);
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.fill_rect(bx - neon_r, top_y - neon_r * 1.2, bld_w + neon_r * 2.0, neon_r * 2.5);

            // -- ANTENNA SPIRE (tall buildings in front layers) --
            if fv > 0.35 && l > 0.5 && bld_h > max_h * 0.45 {
                let spire_h = (8.0 + fv * 18.0) * depth_scale;
                c.set_stroke(Fill::Solid(neon_col.with_alpha(0.80 + bs * 0.15)));
                c.set_line_width(1.2 * depth_scale);
                c.set_shadow(neon_col, 6.0 * depth_scale);
                c.stroke_line(
                    bx + bld_w * 0.5, top_y,
                    bx + bld_w * 0.5, top_y - spire_h,
                );
                // Red blink light at top
                let blink = ((frame_time * 1.8 + b_f * 0.9).sin() * 0.5 + 0.5).powf(8.0);
                c.set_fill(Fill::Solid(Color::rgba(1.0, 0.15, 0.10, blink)));
                c.set_shadow(Color::rgba(1.0, 0.0, 0.0, blink), 8.0);
                c.fill_circle(bx + bld_w * 0.5, top_y - spire_h, 2.0 * depth_scale);
            }

            // -- NEON SIGN BAND (select some buildings in front layers) --
            if b % 4 == 0 && l > 0.65 && bld_h > max_h * 0.5 {
                let sign_y = top_y + bld_h * 0.25;
                let sign_col = mix(acc_col, p_col, (b % 2) as f32);
                c.set_fill(Fill::Solid(sign_col.with_alpha(0.75 + fv * 0.20)));
                c.set_shadow(sign_col, 8.0 * depth_scale);
                c.fill_rect(bx + bld_w * 0.06, sign_y, bld_w * 0.88, 2.5 * depth_scale);
            }

            // -- WET GROUND REFLECTION below horizon --
            if l > 0.55 {
                let refl_alpha = 0.18 + fv * 0.10 + be * 0.05;
                let refl_h = bld_h * 0.28;
                let refl_grad = Fill::linear_gradient(
                    bx, horizon, bx, horizon + refl_h,
                    &[
                        (0.00, body_col.with_alpha(refl_alpha)),
                        (1.00, Color::TRANSPARENT),
                    ],
                );
                c.set_fill(refl_grad);
                c.set_shadow(Color::TRANSPARENT, 0.0);
                c.fill_rect(bx, horizon, bld_w, refl_h);
            }
        }

        // -- DEPTH FOG between layers --
        if layer < LAYERS - 1 {
            let fog_alpha = 0.06 + (1.0 - l) * 0.10;
            let fog_col = Color::rgba(0.04, 0.06, 0.14, fog_alpha);
            c.set_fill(Fill::Solid(fog_col));
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.fill_rect(0.0, 0.0, width, horizon + 2.0);
        }
    }

    // -------------------------------------------------------------------------
    // 4. HORIZON LINE — thin neon glow line at city skyline base
    // -------------------------------------------------------------------------
    let hl_col = mix(p_col, acc_col, 0.4 + be * 0.2);
    let hl_grad = Fill::linear_gradient(
        0.0, horizon, width, horizon,
        &[
            (0.00, Color::TRANSPARENT),
            (0.15, hl_col.with_alpha(0.55 + be * 0.20)),
            (0.50, hl_col.with_alpha(0.85 + be * 0.12)),
            (0.85, hl_col.with_alpha(0.55 + be * 0.20)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_stroke(hl_grad);
    c.set_line_width((1.5 + be * 1.5) * user_scale);
    c.set_shadow(hl_col, (8.0 + be * 6.0) * user_scale);
    c.stroke_line(0.0, horizon, width, horizon);

    // -------------------------------------------------------------------------
    // 5. RAIN STREAKS — diagonal rain falling across the scene
    // -------------------------------------------------------------------------
    let rain_alpha = 0.30 + be * 0.12;
    c.set_stroke(Fill::Solid(Color::rgba(0.70, 0.82, 0.95, rain_alpha * 0.5)));
    c.set_shadow(Color::TRANSPARENT, 0.0);

    for i in 0..RAIN_DROPS {
        let i_f = i as f32;
        let cycle = 1.2 + (i % 5) as f32 * 0.18;
        let t = ((frame_time * 0.9 + i_f * 0.17) % cycle) / cycle;
        let drop_x = (i_f * 0.618034).fract() * width + t * 18.0; // slight diagonal
        let drop_y = t * (height + 40.0) - 20.0;
        let drop_len = (10.0 + (i % 3) as f32 * 6.0) * user_scale;
        let drop_a = (0.15 + (i % 4) as f32 * 0.05) * rain_alpha;

        c.set_stroke(Fill::Solid(Color::rgba(0.70, 0.84, 0.96, drop_a)));
        c.set_line_width(0.8 * user_scale);
        c.stroke_line(drop_x, drop_y, drop_x + 5.0, drop_y + drop_len);
    }

    // -------------------------------------------------------------------------
    // 6. FOREGROUND STREET NEON STRIP — glowing pavement edge
    // -------------------------------------------------------------------------
    let strip_y = horizon + height * 0.04;
    let strip_col = mix(acc_col, p_col, 0.3);
    let strip_glow = Fill::linear_gradient(
        0.0, strip_y, width, strip_y,
        &[
            (0.00, Color::TRANSPARENT),
            (0.10, strip_col.with_alpha(0.40 + be * 0.15)),
            (0.90, strip_col.with_alpha(0.40 + be * 0.15)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_stroke(strip_glow);
    c.set_line_width((2.0 + be * 1.5) * user_scale);
    c.set_shadow(strip_col, (12.0 + be * 8.0) * user_scale);
    c.stroke_line(width * 0.05, strip_y, width * 0.95, strip_y);

    // suppress unused
    let _ = (s_col, bs, PI);

    c.set_global_alpha(1.0);
    c.restore();
}
