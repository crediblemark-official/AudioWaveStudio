//! Radial Fireworks Burst style renderer (`radialFireworksBurst`).
//!
//! Fireworks: on every beat a burst of sparks explodes outward, each spark a
//! bright head with a tapering glowing trail. Bursts fade out over ~1.6s.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::hsl_to_color;
use crate::renderers::RenderContext;

use super::radial_common;

const LIFETIME: f32 = 1.6;

/// Draws one spark as a bright head + tapered trail of `segments` stubs, each
/// thinner and dimmer toward the tail so it looks like a real rocket trail.
fn spark(
    c: &mut GpuCanvas,
    s: &radial_common::RadialSetup,
    head_x: f32,
    head_y: f32,
    tail_x: f32,
    tail_y: f32,
    col: Color,
    alpha: f32,
    width: f32,
) {
    let segments = 6usize;
    for k in (1..=segments).rev() {
        let t0 = k as f32 / segments as f32;
        let t1 = (k - 1) as f32 / segments as f32;
        let fade = t1 * alpha;
        let w = (width * t1).max(0.3);
        let x0 = tail_x + (head_x - tail_x) * t0;
        let y0 = tail_y + (head_y - tail_y) * t0;
        let x1 = tail_x + (head_x - tail_x) * t1;
        let y1 = tail_y + (head_y - tail_y) * t1;
        if k < segments {
            c.set_stroke(Fill::Solid(col.with_alpha(fade)));
            c.set_line_width(w * s.user_scale);
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.stroke_line(x0, y0, x1, y1);
        }
    }

    // Bright head.
    c.set_fill(Fill::Solid(col.with_alpha(alpha)));
    c.set_shadow(col, 10.0 * s.user_scale);
    c.fill_circle(head_x, head_y, width * 0.9);
}

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.08, 0.0);
    let frame_time = ctx.frame_time;

    let st = &mut ctx.state.advanced;
    st.fireworks
        .retain(|f| frame_time - f.start_time < LIFETIME);

    if ctx.beat_count != st.last_firework_beat {
        st.last_firework_beat = ctx.beat_count;
        for k in 0..2 {
            let seed = ctx.beat_count.wrapping_add(k as u64 * 0x9E37_79B9);
            st.fireworks.push(crate::renderers::helpers::Firework {
                angle: radial_common::sweep_angle(seed),
                start_time: frame_time,
                speed: (80.0 + (seed as u32 % 60) as f32) * s.user_scale,
                color_phase: (seed as u32 % 360) as f32 / 360.0,
                sparks: 18 + (seed as u32 % 10) as usize,
            });
        }
    }

    c.set_shadow(Color::TRANSPARENT, 0.0);
    for fw in &st.fireworks {
        let age = frame_time - fw.start_time;
        let prog = (age / LIFETIME).clamp(0.0, 1.0);
        let alpha = (1.0 - prog) * 0.95;
        let col = hsl_to_color(fw.color_phase * 360.0, 0.85, 0.62, 1.0);

        // Central burst flash.
        c.set_fill(Fill::Solid(col.with_alpha(alpha * 0.7)));
        c.set_shadow(col, (16.0 * (1.0 - prog) + 2.0) * s.user_scale);
        c.fill_circle(s.cx, s.cy, (2.0 + 7.0 * (1.0 - prog)) * s.user_scale);

        for k in 0..fw.sparks {
            let kf = k as f32;
            // Jittered direction so sparks scatter, plus slight tangential
            // spread so the burst looks spherical rather than spoked.
            let sa = fw.angle
                + (kf / fw.sparks as f32) * TAU
                + (kf * 2.399_963).sin() * 0.22
                + (kf * 0.791_9).sin() * 0.12;
            let len = (s.inner_r + 8.0 + fw.speed * age) * (0.55 + (kf * 7.13).sin().abs() * 0.45);
            let (sin_a, cos_a) = sa.sin_cos();
            // Trail is anchored inward (toward the burst centre).
            let tail_frac = 0.55 - (kf * 0.618_03).sin().abs() * 0.15;
            let hx = s.cx + cos_a * len;
            let hy = s.cy + sin_a * len;
            let tx = s.cx + cos_a * len * tail_frac;
            let ty = s.cy + sin_a * len * tail_frac;
            let w = (1.0 + (1.0 - prog) * 1.4) * s.user_scale;

            spark(c, &s, hx, hy, tx, ty, col, alpha, w);
        }
    }

    c.set_global_alpha(1.0);

    // Centre image support (kept above the bursts, without the shared black
    // disc so the fireworks stay visible around it).
    crate::renderers::helpers::draw_radial_center_image(c, ctx, s.cx, s.cy, s.inner_r * 0.90);

    c.restore();
}
