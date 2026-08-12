//! Radial Clockwork style renderer (`radialClockwork`) — Steampunk Skeleton Horology Engine.
//!
//! Masterpiece Steampunk Skeleton Clockwork Dial:
//! - Roman Numeral dial face (XII, III, VI, IX) with 60 minute ticks & 12 metallic hour studs.
//! - Visible skeleton gear movement with oscillating balance wheel & ruby jewel bearings.
//! - 3 Ornate Breguet-style metallic hands (Hour, Minute, & Audio-reactive Second hand).
//! - Escapement tick-tock oscillations, gear train torque, & balance spring sparks.
//! - 40+ Floating horology gear dust motes & balance spring sparks.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const TICKS: usize = 60;
const ROMAN_NUMERALS: [&str; 12] = [
    "XII", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI",
];

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let step = ((freq.len() as f32) / TICKS as f32).floor().max(1.0) as usize;

    // Curated Horology Palette (theme-dominant, hardcoded hue only as character accent)
    let brass_gold = mix(Color::rgba(0.95, 0.78, 0.15, 1.0), s.accent, 0.75);
    let bronze_dark = mix(Color::rgba(0.70, 0.40, 0.10, 1.0), s.accent, 0.75);
    let steel_blue = mix(Color::rgba(0.20, 0.55, 0.90, 1.0), s.p_col, 0.70);
    let ruby_red = mix(Color::rgba(0.95, 0.10, 0.35, 1.0), s.s_col, 0.70);
    let spark_white = mix(Color::rgba(1.0, 0.98, 0.90, 0.98), s.glow, 0.15);

    let face_r = s.base_r * 1.45;

    // -------------------------------------------------------------------------
    // 1. HOROLOGY CHAPTER RING & WATCH BEZEL CASING
    // -------------------------------------------------------------------------
    let bezel_grad = Fill::radial_gradient(
        s.cx - face_r * 0.20,
        s.cy - face_r * 0.20,
        0.0,
        s.cx,
        s.cy,
        face_r * 1.15,
        &[
            (0.0, mix(brass_gold, bronze_dark, 0.5).with_alpha(0.30 + s.be * 0.18)),
            (0.65, steel_blue.with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bezel_grad);

    // Bezel Outer Ring Highlight
    c.set_stroke(Fill::Solid(brass_gold.with_alpha(0.75)));
    c.set_line_width(2.0 * s.user_scale);
    c.set_shadow(brass_gold, 12.0 * s.user_scale);
    c.stroke_circle(s.cx, s.cy, face_r);

    // -------------------------------------------------------------------------
    // 2. ROMAN NUMERAL HOUR MARKERS & 60 MINUTE TICKS
    // -------------------------------------------------------------------------
    for i in 0..TICKS {
        let angle = (i as f32 / TICKS as f32) * TAU - std::f32::consts::FRAC_PI_2;
        let is_major = i % 5 == 0;

        let sample_bin = (i * step).min(freq.len().saturating_sub(1));
        let fv = freq[sample_bin] as f32 / 255.0;

        let react = (fv * s.sensitivity * 1.1 + s.be * 0.35).clamp(0.0, 1.8);
        let tick_len = (if is_major { 14.0 } else { 7.0 } + react * 9.0) * s.user_scale;
        let (sin_a, cos_a) = angle.sin_cos();

        let x0 = s.cx + cos_a * face_r;
        let y0 = s.cy + sin_a * face_r;
        let x1 = s.cx + cos_a * (face_r - tick_len);
        let y1 = s.cy + sin_a * (face_r - tick_len);

        let tick_col = mix(brass_gold, spark_white, if is_major { 0.70 } else { 0.20 });
        c.set_stroke(Fill::Solid(tick_col.with_alpha(if is_major { 0.90 } else { 0.60 })));
        c.set_line_width((if is_major { 2.4 } else { 1.2 }) * s.user_scale);
        c.set_shadow(brass_gold, (4.0 + react * 6.0) * s.user_scale);
        c.stroke_line(x0, y0, x1, y1);

        // Roman Numerals on Major 5-Minute Marks
        if is_major {
            let h_idx = i / 5;
            let num_text = ROMAN_NUMERALS[h_idx];
            let tx = s.cx + cos_a * (face_r - 24.0 * s.user_scale);
            let ty = s.cy + sin_a * (face_r - 24.0 * s.user_scale);

            c.draw_text(
                num_text,
                tx,
                ty,
                (11.0 * s.user_scale).clamp(7.0, 16.0),
                "serif",
                700.0,
                false,
                TextAlign::Center,
                Fill::Solid(mix(brass_gold, Color::WHITE, 0.70)),
                1.0,
                &Default::default(),
            );
        }
    }

    // -------------------------------------------------------------------------
    // 3. VISIBLE SKELETON GEAR MOVEMENT & OSCILLATING BALANCE WHEEL
    // -------------------------------------------------------------------------
    let rot_gear = frame_time * 0.35 * (1.0 + s.bs * 0.5);

    // Sub-dial Skeleton Gear (Hour Drive)
    draw_clock_gear(c, &s, s.cx, s.cy, s.base_r * 0.75, 20, rot_gear, brass_gold);
    draw_clock_gear(c, &s, s.cx, s.cy, s.base_r * 0.45, 12, -rot_gear * 1.5, bronze_dark);

    // Oscillating Balance Wheel & Hairspring at Center
    let balance_angle = (frame_time * 12.0).sin() * 0.8;
    c.set_stroke(Fill::Solid(brass_gold.with_alpha(0.85)));
    c.set_line_width(2.2 * s.user_scale);
    c.set_shadow(brass_gold, 10.0 * s.user_scale);
    c.stroke_circle(s.cx, s.cy, s.base_r * 0.30);

    // Balance Wheel 3-Spokes
    for sp in 0..3 {
        let sa = balance_angle + sp as f32 * (TAU / 3.0);
        let sx = s.cx + sa.cos() * (s.base_r * 0.30);
        let sy = s.cy + sa.sin() * (s.base_r * 0.30);

        c.set_stroke(Fill::Solid(mix(brass_gold, Color::WHITE, 0.60)));
        c.set_line_width(1.6 * s.user_scale);
        c.stroke_line(s.cx, s.cy, sx, sy);
    }

    // Synthetic Ruby Jewel Bearing at Center Pivot
    c.set_fill(Fill::Solid(ruby_red));
    c.set_shadow(ruby_red, (14.0 + s.bs * 10.0) * s.user_scale);
    c.fill_circle(s.cx, s.cy, 4.5 * s.user_scale);

    // -------------------------------------------------------------------------
    // 4. 3 ORNATE BREGUET SKELETON CLOCK HANDS
    // -------------------------------------------------------------------------
    let sec_angle = frame_time * (TAU / 60.0) * 10.0 + s.bs * 0.4; // Sweeping high-speed audio second hand
    let min_angle = frame_time * (TAU / 3600.0) * 10.0;
    let hour_angle = frame_time * (TAU / 43200.0) * 10.0;

    // Hour Hand (Short & Ornate)
    draw_breguet_hand(
        c,
        &s,
        hour_angle,
        s.base_r * 0.58,
        4.0,
        brass_gold,
        true,
    );

    // Minute Hand (Long & Sleek)
    draw_breguet_hand(
        c,
        &s,
        min_angle,
        s.base_r * 0.88,
        2.8,
        mix(steel_blue, Color::WHITE, 0.30),
        true,
    );

    // Seconds Hand (High-Precision Audio Needle)
    draw_breguet_hand(
        c,
        &s,
        sec_angle,
        face_r * 0.98 * (1.0 + s.be * 0.12),
        1.6,
        mix(ruby_red, spark_white, 0.70),
        false,
    );

    // -------------------------------------------------------------------------
    // 5. FLOATING HOROLOGY GEAR DUST & ESCAPEMENT SPARKS
    // -------------------------------------------------------------------------
    let mote_count = (20.0 + s.be * 24.0 * s.sensitivity).clamp(12.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 23.0).sin() * TAU;
        let m_dist = s.inner_r * 0.5 + m_t * (face_r * 0.95);

        let mx = s.cx + m_angle.cos() * m_dist;
        let my = s.cy + m_angle.sin() * m_dist;

        let m_sz = (2.2 * (1.0 - m_t) + 1.2 + s.bs * 1.8).clamp(1.0, 5.0) * s.user_scale;
        let m_col = mix(brass_gold, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(brass_gold, 6.0 * s.user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    radial_common::finish(c, ctx, &s);
}

/// Renders an ornate Breguet-style skeleton clock hand with moon ring tip.
fn draw_breguet_hand(
    c: &mut GpuCanvas,
    s: &radial_common::RadialSetup,
    angle: f32,
    len: f32,
    width: f32,
    col: Color,
    has_moon_ring: bool,
) {
    let (sin_a, cos_a) = angle.sin_cos();

    let x0 = s.cx - cos_a * (len * 0.18); // Counterweight tail
    let y0 = s.cy - sin_a * (len * 0.18);
    let x1 = s.cx + cos_a * len;
    let y1 = s.cy + sin_a * len;

    // Hand Stem
    c.set_stroke(Fill::Solid(col));
    c.set_line_width(width * s.user_scale);
    c.set_shadow(col, (8.0 + s.bs * 6.0) * s.user_scale);
    c.stroke_line(x0, y0, x1, y1);

    // Breguet Open Moon Ring Tip
    if has_moon_ring {
        let moon_cx = s.cx + cos_a * (len * 0.72);
        let moon_cy = s.cy + sin_a * (len * 0.72);
        let moon_r = (width * 2.2).clamp(4.0, 10.0) * s.user_scale;

        c.set_stroke(Fill::Solid(col));
        c.set_line_width(1.8 * s.user_scale);
        c.stroke_circle(moon_cx, moon_cy, moon_r);
    }

    // Hand Tip Mote
    c.set_fill(Fill::Solid(mix(col, Color::WHITE, 0.70)));
    c.fill_circle(x1, y1, (2.2 + width * 0.3) * s.user_scale);
}

/// Helper to render skeleton clock gears.
fn draw_clock_gear(
    c: &mut GpuCanvas,
    s: &radial_common::RadialSetup,
    cx: f32,
    cy: f32,
    r: f32,
    teeth: usize,
    rot: f32,
    col: Color,
) {
    let tooth_h = 6.0 * s.user_scale;
    let tooth_w = (TAU / teeth as f32) * 0.35 * r;

    // Gear Rim
    c.set_stroke(Fill::Solid(col.with_alpha(0.65)));
    c.set_line_width(1.5 * s.user_scale);
    c.stroke_circle(cx, cy, r);

    // Teeth
    for t in 0..teeth {
        let a = (t as f32 / teeth as f32) * TAU + rot;
        let (sin_a, cos_a) = a.sin_cos();
        let (px, py) = (-sin_a, cos_a);

        let x0 = cx + cos_a * r + px * tooth_w;
        let y0 = cy + sin_a * r + py * tooth_w;
        let x1 = cx + cos_a * (r + tooth_h);
        let y1 = cy + sin_a * (r + tooth_h);

        c.set_stroke(Fill::Solid(col.with_alpha(0.85)));
        c.set_line_width(1.2 * s.user_scale);
        c.stroke_line(x0, y0, x1, y1);
    }
}
