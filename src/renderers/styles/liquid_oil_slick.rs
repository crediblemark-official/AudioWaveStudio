//! Liquid Oil Slick style renderer (`liquidOilSlick`).
//!
//! Visual Concept:
//! - A rotating elliptical oil puddle: an iridescent film stretched into an oval
//!   that slowly spins, its rainbow interference rings compressing and relaxing
//!   with the audio — a very different silhouette from the round wave styles.
//! - ZERO AMPLITUDE STEALTH HIDING: When music is quiet (`audio_v == 0`), the
//!   puddle shrinks to `inner_r` behind the central logo disc.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    bin_value, hsl_to_color, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
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

    let audio_v = bin_value(freq, 2, 0) * sensitivity;
    let val = (audio_v * 0.85 + be * 0.30).clamp(0.0, 2.2);

    // STEALTH HIDING: when silent (val == 0) the puddle rests at inner_r.
    let rx = inner_r + (val * 150.0) * user_scale;
    let ry = inner_r * (0.45 + 0.30 * (0.5 + 0.5 * (frame_time * 0.6).sin()));

    // Rotating frame so the oval visibly spins.
    let rot = frame_time * 0.35;
    c.save();
    c.translate(cx, cy);
    c.rotate(rot);

    // Iridescent film: moving rainbow across the ellipse.
    let hue_drift = (frame_time * 45.0) % 360.0;
    let oil_fill = Fill::radial_gradient(
        0.0, 0.0, 0.0,
        0.0, 0.0, rx,
        &[
            (0.00, hsl_to_color(hue_drift, 0.85, 0.55, 0.75)),
            (0.40, hsl_to_color(hue_drift + 130.0, 0.9, 0.5, 0.6)),
            (0.75, hsl_to_color(hue_drift + 260.0, 0.9, 0.45, 0.45)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(oil_fill);
    c.set_shadow(glow, (20.0 + bs * 15.0) * user_scale);
    c.fill_ellipse(0.0, 0.0, rx, ry);

    // Interference rings as nested ellipses compressing/relaxing.
    for ring in 0..3 {
        let ring_f = ring as f32;
        let frac = 0.66 + ring_f * 0.11 + ((frame_time * 0.9 + ring_f * 2.0).sin()) * 0.04;
        let rr = rx * frac;
        let rry = ry * (frac - 0.02);
        let ring_hue = (hue_drift + ring_f * 120.0) % 360.0;
        let ring_col = hsl_to_color(ring_hue, 0.9, 0.62, 0.5 - ring_f * 0.1);

        c.set_stroke(Fill::Solid(ring_col));
        c.set_line_width((1.6 - ring_f * 0.25).max(0.6) * user_scale);
        c.set_shadow(ring_col, (8.0 + bs * 5.0) * user_scale);
        let n = 48usize;
        let mut ring_pts: Vec<(f32, f32)> = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let a = i as f32 / n as f32 * TAU;
            ring_pts.push((a.cos() * rr, a.sin() * rry));
        }
        c.stroke_polyline(&ring_pts);
    }

    // Bright rim highlight at the leading edge.
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(mix(glow, Color::WHITE, 0.8)));
    c.set_line_width((2.4 + bs * 1.4) * user_scale);
    let n = 48usize;
    let mut rim_pts: Vec<(f32, f32)> = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let a = i as f32 / n as f32 * TAU;
        rim_pts.push((a.cos() * rx, a.sin() * ry));
    }
    c.stroke_polyline(&rim_pts);

    c.restore();

    // Center Logo Disc (in the world frame).
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(glow));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(glow, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    let _ = (p_col, s_col, accent);
    c.set_global_alpha(1.0);
    c.restore();
}
