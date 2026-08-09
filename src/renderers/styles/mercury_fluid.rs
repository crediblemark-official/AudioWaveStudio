//! Liquid Mercury Fluid Wave style renderer (`mercuryFluid`) - Realistic Liquid Metal Engine.
//!
//! Rewritten for physical realism:
//! - Smooth audio-reactive fluid surface with surface-tension meniscus skew.
//! - High-gloss specular chrome reflections (narrow horizontal streaks).
//! - Gravity-driven parabolic droplet splash arcs.
//! - Dense pool base with wetting-edge meniscus glow.

use std::f32::consts::PI;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const SURFACE_SEGS: usize = 96;
const DROP_COUNT: usize = 22;

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
    // Mercury pools toward bottom due to density
    let cy = height * 0.60 + pos_offset_y;
    let reference_size = width.min(height);
    let pool_w = 0.76 * width * user_scale;
    let pool_h = 0.22 * reference_size * user_scale; // flat = dense liquid
    let left_x = cx - pool_w * 0.5;
    let right_x = cx + pool_w * 0.5;
    let base_y = cy + pool_h * 0.55;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Ambient mercury mist
    let bg_grad = Fill::radial_gradient(
        cx, cy - pool_h, 0.0, cx, cy, pool_w * 0.9,
        &[
            (0.0, mix(p_col, Color::rgba(0.70, 0.85, 1.0, 0.18 + be * 0.12), 0.4)),
            (0.55, mix(s_col, Color::rgba(0.10, 0.15, 0.30, 0.06), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_grad);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. FLUID SURFACE - surface-tension skew: peaks sharp, troughs broad
    //    Mercury surface tension: 487 mN/m (vs water 72 mN/m) -> very taut surface
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / SURFACE_SEGS).max(1);
    let mut surface_pts: Vec<(f32, f32)> = Vec::with_capacity(SURFACE_SEGS + 1);

    for i in 0..=SURFACE_SEGS {
        let t = i as f32 / SURFACE_SEGS as f32;
        let x = left_x + t * pool_w;

        let bin_k = (i * step_f / ((SURFACE_SEGS / bar_count.max(1)).max(1)))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let wave_amp = pool_h * 0.55 * fv * sensitivity;
        let raw = (t * PI * 8.0 + frame_time * 1.8).sin();
        // Surface tension skew: narrow sharp peaks, broad flat troughs
        let skewed = if raw > 0.0 { raw.powf(0.55) } else { -(-raw).powf(1.7) };
        let audio_y = cy - skewed * wave_amp;

        // Beat ripple propagates from left edge inward
        let beat_ripple = bs * 16.0 * user_scale * (-t * 3.0).exp()
            * (t * PI * 14.0 - frame_time * 6.5).sin();

        // Meniscus: mercury is NON-wetting -> curves DOWN at walls (convex meniscus)
        // contact angle ~140 degrees for glass - surface dips at edges
        let edge_dist = (t * 2.0 - 1.0).abs();
        let meniscus = -(1.0 - edge_dist).powf(4.0) * pool_h * 0.10;

        let y = (audio_y - beat_ripple + meniscus)
            .clamp(cy - pool_h * 1.40, cy + pool_h * 0.12);
        surface_pts.push((x, y));
    }

    // Closed pool polygon: surface + bottom
    let mut pool_poly: Vec<(f32, f32)> = surface_pts.clone();
    pool_poly.push((right_x, base_y));
    pool_poly.push((left_x, base_y));

    // Chrome gradient tinted by theme primary — keeps metallic feel, adds color
    // chrome_base: neutral silver; tint toward p_col at mid, s_col at bottom
    let chrome_top    = Color::rgba(0.97, 0.99, 1.00, 0.98); // always bright white at surface
    let chrome_mid    = mix(Color::rgba(0.38, 0.50, 0.68, 0.98), p_col.with_alpha(0.98), 0.30);
    let chrome_deep   = mix(Color::rgba(0.15, 0.22, 0.35, 0.99), s_col.with_alpha(0.99), 0.22);
    let chrome_bottom = mix(Color::rgba(0.04, 0.07, 0.14, 1.00), accent_col.with_alpha(1.0), 0.08);

    let fill_grad = Fill::linear_gradient(
        cx, cy - pool_h, cx, base_y,
        &[
            (0.00, chrome_top),
            (0.08, mix(Color::rgba(0.75, 0.86, 0.96, 0.97), p_col.with_alpha(0.97), 0.20)),
            (0.30, chrome_mid),
            (0.65, chrome_deep),
            (1.00, chrome_bottom),
        ],
    );
    c.set_fill(fill_grad);
    c.fill_polygon(&pool_poly);

    // Sharp chrome surface edge — tinted by glow+accent on bass hits
    let surf_col = mix(
        mix(Color::rgba(0.92, 0.96, 1.0, 0.95), p_col.with_alpha(0.95), 0.18),
        mix(glow_col, accent_col, 0.4),
        be * 0.45,
    );
    c.set_stroke(Fill::Solid(surf_col));
    c.set_line_width((2.8 + be * 2.0) * user_scale);
    c.set_shadow(surf_col, (14.0 + bs * 10.0) * user_scale);
    c.stroke_polyline(&surface_pts);

    // -------------------------------------------------------------------------
    // 2. SPECULAR HIGHLIGHTS - mercury reflects ~70% of visible light
    //    Two narrow horizontal streaks that shift with audio & time
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    for spec_i in 0..2usize {
        let spec_t = 0.28 + spec_i as f32 * 0.20 + be * 0.05;
        let spec_y = cy - pool_h * spec_t
            + (frame_time * 0.7 + spec_i as f32 * 1.4).sin() * 3.5 * user_scale;
        let spec_w = pool_w * (0.52 - spec_i as f32 * 0.16);
        let spec_x = cx - spec_w * 0.5 + (frame_time * 0.45).cos() * 7.0 * user_scale;
        let spec_a = 0.52 - spec_i as f32 * 0.16 + bs * 0.08;

        // Specular tinted slightly by p_col to pick up theme color on reflections
        let spec_tint = mix(Color::rgba(1.0, 1.0, 1.0, spec_a * 0.75), p_col.with_alpha(spec_a * 0.75), 0.12);
        c.set_stroke(Fill::Solid(spec_tint));
        c.set_line_width((0.9 + spec_i as f32 * 0.5) * user_scale);
        c.stroke_line(spec_x, spec_y, spec_x + spec_w, spec_y);

        // Soft fill glow band
        let spec_grad = Fill::linear_gradient(
            spec_x, spec_y, spec_x + spec_w, spec_y,
            &[
                (0.00, Color::TRANSPARENT),
                (0.25, spec_tint.with_alpha(spec_a * 0.38)),
                (0.60, spec_tint.with_alpha(spec_a * 0.16)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(spec_grad);
        c.fill_rect(spec_x, spec_y - 3.0 * user_scale, spec_w, 7.0 * user_scale);
    }

    // -------------------------------------------------------------------------
    // 3. DROPLETS - parabolic arcs under gravity, chrome ball shading
    //    Mercury droplets are perfectly spherical due to high surface tension
    // -------------------------------------------------------------------------
    let gravity = 400.0 * user_scale;
    let step_drop = (freq.len() / DROP_COUNT).max(1);

    for d_i in 0..DROP_COUNT {
        let d_f = d_i as f32;
        let phase = d_f * 0.40 + (d_i % 5) as f32 * 1.2;
        let cycle_dur = 1.5 + (d_i % 4) as f32 * 0.22;
        let d_t = ((frame_time * 0.65 + phase) % cycle_dur) / cycle_dur;

        let bin_k = (d_i * step_drop).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let spawn_t = (d_f + 0.5) / DROP_COUNT as f32;
        let spawn_x = left_x + spawn_t * pool_w;
        // Launch velocity: bass drives bigger eruptions
        let v0_y = -(70.0 + fv * 150.0 * sensitivity + be * 110.0) * user_scale;

        let drop_x = spawn_x + (d_t - 0.5) * 18.0 * user_scale;
        let drop_y = cy + v0_y * d_t + 0.5 * gravity * d_t * d_t;

        // Cull droplets that have hit the surface
        let ti = ((spawn_x - left_x) / pool_w).clamp(0.0, 1.0);
        let surf_idx = (ti * SURFACE_SEGS as f32) as usize;
        let surf_y = surface_pts.get(surf_idx).map(|p| p.1).unwrap_or(cy);
        if drop_y > surf_y + 3.0 {
            continue;
        }

        let drop_sz = (2.5 + (d_i % 3) as f32 * 1.6 + fv * 3.8 + be * 2.0) * user_scale;
        let alpha = (1.0 - d_t * 0.55).clamp(0.25, 1.0);

        // Chrome ball tinted by accent/p_col on frequency peaks
        let drop_mid   = mix(Color::rgba(0.28, 0.40, 0.58, alpha * 0.90), accent_col.with_alpha(alpha * 0.90), fv * 0.35);
        let drop_dark  = mix(Color::rgba(0.05, 0.09, 0.16, alpha * 0.82), p_col.with_alpha(alpha * 0.82), 0.15);
        let drop_glow_col = mix(Color::rgba(0.80, 0.92, 1.0, 0.70 * alpha), glow_col.with_alpha(0.70 * alpha), 0.30);

        let drop_fill = Fill::radial_gradient(
            drop_x - drop_sz * 0.30,
            drop_y - drop_sz * 0.30,
            0.0,
            drop_x,
            drop_y,
            drop_sz,
            &[
                (0.00, Color::rgba(1.0, 1.0, 1.0, alpha)),
                (0.22, Color::rgba(0.84, 0.92, 0.98, alpha * 0.96)),
                (0.62, drop_mid),
                (1.00, drop_dark),
            ],
        );
        c.set_fill(drop_fill);
        c.set_shadow(drop_glow_col, (5.5 + fv * 5.0) * user_scale);
        c.fill_circle(drop_x, drop_y, drop_sz);
    }

    // -------------------------------------------------------------------------
    // 4. POOL EDGES - convex meniscus wetting edge + base glint
    // -------------------------------------------------------------------------
    // Meniscus edge — tinted by glow+s_col
    let edge_col = mix(
        mix(Color::rgba(0.85, 0.94, 1.0, 0.65 + be * 0.20), glow_col.with_alpha(0.65 + be * 0.20), 0.30),
        s_col.with_alpha(0.50),
        0.20,
    );
    c.set_stroke(Fill::Solid(edge_col));
    c.set_line_width(1.4 * user_scale);
    c.set_shadow(edge_col, (7.0 + be * 5.0) * user_scale);
    c.stroke_line(left_x, cy + pool_h * 0.10, left_x, base_y);
    c.stroke_line(right_x, cy + pool_h * 0.10, right_x, base_y);

    // Base glint — tinted with accent_col
    let glint_col = mix(Color::rgba(0.55, 0.72, 0.92, 0.40 + be * 0.12), accent_col.with_alpha(0.40 + be * 0.12), 0.25);
    c.set_stroke(Fill::Solid(glint_col));
    c.set_line_width(1.8 * user_scale);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.stroke_line(left_x + 10.0 * user_scale, base_y, right_x - 10.0 * user_scale, base_y);



    c.set_global_alpha(1.0);
    c.restore();
}
