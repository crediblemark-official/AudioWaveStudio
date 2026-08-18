//! Liquid Tri-Lobe Aura style renderer (`liquidTriLobeAura`).
//!
//! Visual Concept:
//! - Three-lobed clover / cat-ear organic aura, each lobe a smooth rounded petal
//!   that inflates with its own audio band.
//! - ZERO AMPLITUDE STEALTH HIDING: When music is quiet (`audio_v == 0`), liquid
//!   contracts 100% cleanly behind the central logo disc (`r_curr == inner_r`).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, fill_radial_polygon, mix};
use crate::renderers::{
    bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col  = theme_primary(theme);
    let s_col  = theme_secondary(theme);
    let accent = theme_accent(theme);
    let glow   = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let bass_mult    = ctx.config.reactivity.bass_multiplier;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;
    let bar_count    = ctx.config.reactivity.bar_count.clamp(16, 96);

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.10 + bs * 0.05);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    let step = ((freq.len() as f32) / bar_count as f32).floor().max(1.0) as usize;
    let rot = frame_time * 0.08;

    // Per-band energy so each of the 3 lobes swells independently.
    let lobe0 = bin_value(freq, step, 0) * sensitivity;
    let lobe1 = bin_value(freq, step, bar_count / 3) * sensitivity;
    let lobe2 = bin_value(freq, step, bar_count * 2 / 3) * sensitivity;

    let mut raw_pts: Vec<(f32, f32)> = Vec::with_capacity(bar_count);
    for i in 0..bar_count {
        let t = i as f32 / bar_count as f32;
        let angle = t * TAU + rot;

        let audio_v = crate::renderers::styles::radial_common::smooth_ring_bin(
            freq, step, i, bar_count,
        ) * sensitivity;

        // Trefoil window: peaks at 3 equally spaced angles, pinches in between.
        let lobe_window = ((angle * 3.0).cos() * 0.5 + 0.5).powf(2.0);
        let per_lobe = match i % 3 {
            0 => lobe0,
            1 => lobe1,
            _ => lobe2,
        };
        let val = (audio_v * 0.55 + per_lobe * 1.4 * lobe_window + be * 0.30).clamp(0.0, 2.6);

        // STEALTH HIDING: When music is quiet (val == 0), wave_h == 0 so r_curr == inner_r!
        let wave_h = (val * 140.0) * user_scale;
        let r_curr = inner_r + wave_h;

        let (cos_a, sin_a) = angle.sin_cos();
        raw_pts.push((cx + cos_a * r_curr, cy + sin_a * r_curr));
    }

    let mut smooth_curve: Vec<(f32, f32)> = Vec::new();
    let num_pts = raw_pts.len();
    for i in 0..num_pts {
        let p0 = raw_pts[i];
        let p1 = raw_pts[(i + 1) % num_pts];
        let mid = ((p0.0 + p1.0) * 0.5, (p0.1 + p1.1) * 0.5);

        let seg = GpuCanvas::sample_quadratic(
            if smooth_curve.is_empty() { p0 } else { *smooth_curve.last().unwrap() },
            p0, mid, 4,
        );
        if smooth_curve.is_empty() {
            smooth_curve.extend(seg);
        } else {
            smooth_curve.extend(seg.into_iter().skip(1));
        }
    }
    if let Some(&first) = smooth_curve.first() { smooth_curve.push(first); }

    let fill_grad = Fill::radial_gradient(
        cx, cy, inner_r,
        cx, cy, base_r * 2.2,
        &[
            (0.00, p_col.with_alpha(0.85)),
            (0.50, mix(accent, s_col, 0.4).with_alpha(0.55)),
            (1.00, Color::TRANSPARENT),
        ],
    );

    c.set_fill(fill_grad);
    c.set_shadow(glow, (20.0 + bs * 15.0) * user_scale);
    fill_radial_polygon(c, cx, cy, &smooth_curve);

    c.set_stroke(Fill::Solid(mix(glow, Color::WHITE, 0.70)));
    c.set_line_width((3.0 + bs * 1.8) * user_scale);
    c.stroke_polyline(&smooth_curve);

    // Three lobe accents: a brighter petal vein from center toward each lobe tip.
    for k in 0..3 {
        let lobe_angle = rot + k as f32 * TAU / 3.0;
        let tip_r = inner_r + (lobe_window_radius(&smooth_curve, cx, cy, lobe_angle, inner_r) - inner_r) * 0.85 + inner_r;
        let tip = (cx + lobe_angle.cos() * tip_r, cy + lobe_angle.sin() * tip_r);
        c.set_stroke(Fill::Solid(mix(glow, Color::WHITE, 0.55)));
        c.set_line_width((1.6 + be * 0.8) * user_scale);
        c.stroke_line(cx, cy, tip.0, tip.1);
        c.set_fill(Fill::Solid(mix(glow, Color::WHITE, 0.9)));
        c.set_shadow(glow, (12.0 + bs * 8.0) * user_scale);
        c.fill_circle(tip.0, tip.1, (3.0 + bs * 1.5) * user_scale);
    }

    // Center Logo Disc
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(glow));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(glow, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}

/// Approximate the contour radius at a given angle by interpolating the nearest
/// samples of the (already closed) smooth curve.
fn lobe_window_radius(
    pts: &[(f32, f32)],
    cx: f32, cy: f32,
    angle: f32,
    _inner_r: f32,
) -> f32 {
    if pts.len() < 2 {
        return 0.0;
    }
    let n = pts.len() - 1;
    let target = ((angle % std::f32::consts::TAU) + std::f32::consts::TAU) % std::f32::consts::TAU;
    let seg = target / std::f32::consts::TAU * n as f32;
    let i = seg.floor() as usize % n;
    let j = (i + 1) % n;
    let f = seg - seg.floor();
    let r_i = ((pts[i].0 - cx).powi(2) + (pts[i].1 - cy).powi(2)).sqrt();
    let r_j = ((pts[j].0 - cx).powi(2) + (pts[j].1 - cy).powi(2)).sqrt();
    r_i + (r_j - r_i) * f
}
