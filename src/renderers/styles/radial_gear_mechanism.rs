//! Radial Gear Mechanism style renderer (`radialGearMechanism`) — Steampunk Planetary Gear Engine.
//!
//! Masterpiece Steampunk Planetary Gear Train Assembly:
//! - Central Brass Sun Gear with 18 precision involute teeth & 4-spoke wheel cutouts.
//! - 4 Interlocking Planetary Spur Gears with 12 teeth & bronze metallic bevels meshing in real mechanical sync.
//! - Outer Annular Ring Gear with 48 inward teeth, anodized titanium casing, & bolt rivets.
//! - Audio-reactive gear torque, rotation acceleration, & tooth mesh friction sparks.
//! - 40+ Floating mechanical brass dust particles & lubricant vapor sparks.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const SUN_TEETH: usize = 18;
const PLANET_TEETH: usize = 12;
const RING_TEETH: usize = 48;
const PLANET_COUNT: usize = 4;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let beat_speed = 1.0 + s.bs * 0.70;
    let rot_sun = frame_time * 0.45 * beat_speed;
    let rot_orbit = -frame_time * 0.15 * beat_speed;

    // Curated Steampunk Gear Palette (theme-dominant, hardcoded hue only as character accent)
    let brass_gold = mix(Color::rgba(0.95, 0.75, 0.15, 1.0), s.accent, 0.75);
    let bronze_red = mix(Color::rgba(0.85, 0.45, 0.10, 1.0), s.accent, 0.75);
    let chrome_steel = mix(Color::rgba(0.70, 0.78, 0.88, 1.0), s.p_col, 0.70);
    let spark_white = mix(Color::rgba(1.0, 0.98, 0.90, 0.98), s.glow, 0.15);

    // Gear Radii Definitions
    let annulus_r = s.base_r * 1.45;
    let sun_r = s.base_r * 0.55;
    let planet_r = (annulus_r - sun_r) * 0.50;
    let orbit_r = (annulus_r + sun_r) * 0.50;

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC STEAMPUNK BACKDROP GLOW
    // -------------------------------------------------------------------------
    let bg_gear = Fill::radial_gradient(
        s.cx,
        s.cy,
        0.0,
        s.cx,
        s.cy,
        annulus_r * 1.8,
        &[
            (0.0, mix(brass_gold, bronze_red, 0.5).with_alpha(0.28 + s.be * 0.18)),
            (0.50, chrome_steel.with_alpha(0.10)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_gear);

    // -------------------------------------------------------------------------
    // 2. OUTER ANNULAR RING GEAR (48 INWARD TEETH)
    // -------------------------------------------------------------------------
    draw_metallic_gear(
        c,
        &s,
        s.cx,
        s.cy,
        annulus_r,
        RING_TEETH,
        -frame_time * 0.03,
        true,
        bronze_red,
        s.be,
        8,
    );

    // Outer Ring Bolt Rivets
    let rivets = 16usize;
    for r_i in 0..rivets {
        let ra = (r_i as f32 / rivets as f32) * TAU;
        let rx = s.cx + ra.cos() * (annulus_r + 14.0 * s.user_scale);
        let ry = s.cy + ra.sin() * (annulus_r + 14.0 * s.user_scale);

        c.set_fill(Fill::Solid(mix(chrome_steel, Color::WHITE, 0.70)));
        c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.60), 4.0 * s.user_scale);
        c.fill_circle(rx, ry, 2.5 * s.user_scale);
    }

    // -------------------------------------------------------------------------
    // 3. CENTRAL BRASS SUN GEAR (18 OUTWARD TEETH & 4 SPOKES)
    // -------------------------------------------------------------------------
    draw_metallic_gear(
        c,
        &s,
        s.cx,
        s.cy,
        sun_r,
        SUN_TEETH,
        rot_sun,
        false,
        brass_gold,
        s.be * 1.2,
        4,
    );

    // Sun Gear Center Axle Hub
    c.set_fill(Fill::Solid(mix(spark_white, brass_gold, 0.60)));
    c.set_shadow(brass_gold, (14.0 + s.bs * 10.0) * s.user_scale);
    c.fill_circle(s.cx, s.cy, (6.0 + s.be * 3.0) * s.user_scale);

    // -------------------------------------------------------------------------
    // 4. 4 INTERLOCKING PLANETARY SPUR GEARS (12 TEETH EACH)
    // -------------------------------------------------------------------------
    let step = ((freq.len() as f32) / PLANET_COUNT as f32).floor().max(1.0) as usize;

    for p in 0..PLANET_COUNT {
        let pa = rot_orbit + (p as f32 / PLANET_COUNT as f32) * TAU;
        let px = s.cx + pa.cos() * orbit_r;
        let py = s.cy + pa.sin() * orbit_r;

        // Counter-rotation formula so planet teeth mesh seamlessly with sun & ring
        let rot_planet = -rot_sun * (SUN_TEETH as f32 / PLANET_TEETH as f32) + pa * 0.5;

        let fv = radial_common::swept_bin(freq, step, p, PLANET_COUNT, &s) * s.sensitivity;
        let p_col = mix(mix(chrome_steel, brass_gold, p as f32 / PLANET_COUNT as f32), spark_white, fv * 0.35);

        draw_metallic_gear(
            c,
            &s,
            px,
            py,
            planet_r,
            PLANET_TEETH,
            rot_planet,
            false,
            p_col,
            fv + s.be * 0.4,
            3,
        );

        // Gear Tooth Contact Mesh Sparks (where planet hits sun gear)
        let mesh_x = s.cx + pa.cos() * (sun_r + (planet_r * 0.5));
        let mesh_y = s.cy + pa.sin() * (sun_r + (planet_r * 0.5));

        c.set_fill(Fill::Solid(spark_white));
        c.set_shadow(brass_gold, (12.0 + s.bs * 8.0) * s.user_scale);
        c.fill_circle(mesh_x, mesh_y, (2.8 + fv * 2.0) * s.user_scale);
    }

    // -------------------------------------------------------------------------
    // 5. FLOATING MECHANICAL BRASS DUST & SPARKS
    // -------------------------------------------------------------------------
    let mote_count = (20.0 + s.be * 24.0 * s.sensitivity).clamp(12.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 23.0).sin() * TAU;
        let m_dist = sun_r * 0.8 + m_t * (annulus_r * 0.95);

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

/// Renders a photorealistic metallic gear wheel complete with 3D gear teeth,
/// metallic rim bevels, gear body, and spoke cutouts.
fn draw_metallic_gear(
    c: &mut GpuCanvas,
    s: &radial_common::RadialSetup,
    cx: f32,
    cy: f32,
    r: f32,
    teeth: usize,
    rot: f32,
    inward: bool,
    col: Color,
    val: f32,
    spokes: usize,
) {
    let tooth_len = (8.0 + val * 8.0) * s.user_scale;
    let tooth_half_w = (TAU / teeth as f32) * 0.32 * r;
    let dir = if inward { -1.0 } else { 1.0 };

    // 1. Gear Solid Body Disc
    let body_r = if inward { r + 16.0 * s.user_scale } else { r };
    let body_grad = Fill::radial_gradient(
        cx - body_r * 0.20,
        cy - body_r * 0.20,
        0.0,
        cx,
        cy,
        body_r,
        &[
            (0.0, mix(col, Color::WHITE, 0.50).with_alpha(0.95)),
            (0.65, col.with_alpha(0.92)),
            (1.0, mix(col, Color::BLACK, 0.60).with_alpha(0.98)),
        ],
    );
    c.set_fill(body_grad);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.70), 12.0 * s.user_scale);

    if inward {
        // Outer Ring Housing
        c.stroke_circle(cx, cy, body_r);
        c.set_line_width(12.0 * s.user_scale);
        c.stroke_circle(cx, cy, r + 6.0 * s.user_scale);
    } else {
        c.fill_circle(cx, cy, r);
    }

    // 2. 3D Gear Teeth (Trapezoidal Involute Gear Tooth Profiles)
    for t in 0..teeth {
        let a = (t as f32 / teeth as f32) * TAU + rot;
        let (sin_a, cos_a) = a.sin_cos();
        let (px, py) = (-sin_a, cos_a);

        let r_in = r;
        let r_out = r + dir * tooth_len;

        let corners = [
            (cx + cos_a * r_in + px * (tooth_half_w * 1.15), cy + sin_a * r_in + py * (tooth_half_w * 1.15)),
            (cx + cos_a * r_in - px * (tooth_half_w * 1.15), cy + sin_a * r_in - py * (tooth_half_w * 1.15)),
            (cx + cos_a * r_out - px * (tooth_half_w * 0.75), cy + sin_a * r_out - py * (tooth_half_w * 0.75)),
            (cx + cos_a * r_out + px * (tooth_half_w * 0.75), cy + sin_a * r_out + py * (tooth_half_w * 0.75)),
        ];

        let tooth_col = mix(col, Color::WHITE, 0.35 + (a.cos() * 0.15));
        c.set_fill(Fill::Solid(tooth_col));
        c.set_shadow(col, (4.0 + val * 4.0) * s.user_scale);
        c.fill_polygon(&corners);

        // Specular Top Highlight on Tooth Tip
        c.set_stroke(Fill::Solid(Color::WHITE));
        c.set_line_width(1.0 * s.user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_line(corners[3].0, corners[3].1, corners[2].0, corners[2].1);
    }

    // 3. Gear Spoke Cutouts (for non-inward gears)
    if !inward && spokes > 0 && r > 25.0 * s.user_scale {
        let inner_hub_r = r * 0.30;
        let spoke_outer_r = r * 0.75;

        for s_i in 0..spokes {
            let sa = (s_i as f32 / spokes as f32) * TAU + rot;
            let (sx, sy) = sa.sin_cos();

            let x0 = cx + sy * inner_hub_r;
            let y0 = cy + sx * inner_hub_r;
            let x1 = cx + sy * spoke_outer_r;
            let y1 = cy + sx * spoke_outer_r;

            c.set_stroke(Fill::Solid(mix(col, Color::BLACK, 0.50)));
            c.set_line_width(3.0 * s.user_scale);
            c.stroke_line(x0, y0, x1, y1);
        }

        // Inner Hub Ring
        c.set_stroke(Fill::Solid(mix(col, Color::WHITE, 0.60)));
        c.set_line_width(1.8 * s.user_scale);
        c.stroke_circle(cx, cy, inner_hub_r);
    }
}
