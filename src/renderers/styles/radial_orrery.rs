//! Radial Orrery style renderer (`radialOrrery`).
//!
//! A solar-system orrery: a glowing sun core, concentric circular orbits with
//! planets of different speeds/directions, orbital trails that speed up on
//! beats, and audio-reactive orbit radii.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::RenderContext;

use super::radial_common;

const ORBITS: usize = 4;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let step = ((freq.len() as f32) / ORBITS as f32).floor().max(1.0) as usize;

    for o in 0..ORBITS {
        let of = o as f32;
        let audio_v = radial_common::swept_bin(freq, step, o, ORBITS, &s) * s.sensitivity;
        let orbit_r =
            (s.inner_r + 8.0 + of * 30.0 + audio_v * 18.0 + s.be * 10.0) * s.user_scale;

        let orbit_col = mix(s.p_col, s.glow, of / ORBITS as f32);
        c.set_stroke(Fill::Solid(orbit_col.with_alpha(0.25)));
        c.set_line_width(1.0 * s.user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_circle(s.cx, s.cy, orbit_r);

        // Planet motion: opposite directions on even/odd orbits, beat-sped.
        let dir = if o % 2 == 0 { 1.0 } else { -1.0 };
        let speed = (0.25 + of * 0.22) * dir * (1.0 + s.bs * 0.6);
        let planet_a = frame_time * speed + of * 1.7;

        let px = s.cx + planet_a.cos() * orbit_r;
        let py = s.cy + planet_a.sin() * orbit_r;
        let planet_col = mix(s.glow, s.accent, of / ORBITS as f32);

        // Orbital trail behind the planet.
        c.set_stroke(Fill::Solid(planet_col.with_alpha(0.35)));
        c.set_line_width(1.4 * s.user_scale);
        c.set_shadow(planet_col, (6.0 + s.bs * 4.0) * s.user_scale);
        c.stroke_arc(s.cx, s.cy, orbit_r, planet_a - 0.7, planet_a);

        // Planet.
        c.set_fill(Fill::Solid(planet_col));
        c.set_shadow(planet_col, (10.0 + s.bs * 6.0) * s.user_scale);
        c.fill_circle(px, py, (3.5 + of * 0.7 + audio_v * 1.5) * s.user_scale);
    }

    // Sun core.
    let sun_r = (14.0 + s.be * 8.0) * s.user_scale;
    c.set_fill(Fill::radial_gradient(
        s.cx,
        s.cy,
        0.0,
        s.cx,
        s.cy,
        sun_r * 2.6,
        &[
            (0.0, mix(Color::WHITE, s.glow, 0.3)),
            (0.4, s.glow.with_alpha(0.6)),
            (1.0, Color::TRANSPARENT),
        ],
    ));
    c.fill_circle(s.cx, s.cy, sun_r * 2.6);
    c.set_fill(Fill::Solid(mix(Color::WHITE, s.glow, 0.4)));
    c.set_shadow(s.glow, (16.0 + s.bs * 8.0) * s.user_scale);
    c.fill_circle(s.cx, s.cy, sun_r);

    draw_radial_center_image(c, ctx, s.cx, s.cy, sun_r * 0.6);

    // Shared ending minus the black disc: the shared `finish()` would paint an
    // opaque disc (inner_r * 0.96) over the sun, so draw only the glowing ring.
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(s.glow));
    c.set_line_width((3.0 + s.be * 2.0) * s.user_scale);
    c.set_shadow(s.glow, (16.0 + s.bs * 12.0) * s.user_scale);
    c.stroke_circle(s.cx, s.cy, s.inner_r);

    c.set_global_alpha(1.0);
    c.restore();
}
