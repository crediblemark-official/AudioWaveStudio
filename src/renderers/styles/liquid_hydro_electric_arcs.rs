//! Liquid Hydro-Electric Arc Reactor style renderer (`liquidHydroElectricArcs`).
//!
//! Visual Concept:
//! - Futuristic Sci-Fi Stark Arc Reactor Liquid Core inside a 3D Glass Cylinder Grid.
//! - High-voltage purple/cyan zig-zag plasma bolts snapping across anode/cathode node pins.
//! - Audio-reactive high-voltage electric discharges & floating energy arc motes.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const REACTOR_PINS: usize = 16;

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

    let be = ctx.bass_energy.clamp(0.0, 3.0);
    let bs = ctx.beat_strength.clamp(0.0, 3.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08 + bs * 0.04);
    let max_r = base_r * 1.55;

    // Palette: Arc Reactor Plasma
    let arc_cyan = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.85);
    let arc_purple = mix(accent_col, Color::rgba(0.70, 0.0, 1.0, 1.0), 0.85);
    let spark_white = Color::WHITE;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Ambient Arc Reactor Backdrop
    let bg_reactor = Fill::radial_gradient(
        cx, cy, inner_r,
        cx, cy, max_r * 1.8,
        &[
            (0.0, mix(arc_cyan, arc_purple, 0.5).with_alpha(0.38 + be * 0.20)),
            (0.55, s_col.with_alpha(0.15)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_reactor);
    c.fill_rect(cx - max_r * 1.8, cy - max_r * 1.8, max_r * 3.6, max_r * 3.6);

    // 2. 3D Glass Cylinder Anode/Cathode Containment Ring
    c.set_stroke(Fill::Solid(arc_cyan.with_alpha(0.60)));
    c.set_line_width(2.2 * user_scale);
    c.set_shadow(arc_cyan, (14.0 + bs * 8.0) * user_scale);
    c.stroke_circle(cx, cy, max_r);

    // 3. 16 Reactor Pin Nodes & Zig-Zag Electric Bolts
    let step = (freq.len() / REACTOR_PINS).max(1);

    for p_i in 0..REACTOR_PINS {
        let pin_angle = (p_i as f32 / REACTOR_PINS as f32) * TAU;
        let (sin_a, cos_a) = pin_angle.sin_cos();

        let fv = super::radial_common::full_random_scattered_bin(freq, step, p_i, REACTOR_PINS, ctx.beat_count);
        let val = (fv * sensitivity * 1.1 + be * 0.35 + bs * 0.20).clamp(0.10, 2.2);

        let pin_x = cx + cos_a * max_r;
        let pin_y = cy + sin_a * max_r;

        // Draw Anode/Cathode Node Pin
        c.set_fill(Fill::Solid(spark_white));
        c.set_shadow(arc_cyan, (12.0 + val * 8.0) * user_scale);
        c.fill_circle(pin_x, pin_y, (4.0 + val * 2.5) * user_scale);

        // High-Voltage Zig-Zag Plasma Bolt (Inner Core -> Outer Pin)
        let mid_x = (cx + pin_x) * 0.5 + (frame_time * 30.0 + p_i as f32 * 5.0).sin() * (12.0 * user_scale);
        let mid_y = (cy + pin_y) * 0.5 + (frame_time * 30.0 + p_i as f32 * 5.0).cos() * (12.0 * user_scale);

        let bolt_col = mix(arc_cyan, arc_purple, p_i as f32 / REACTOR_PINS as f32);

        // Outer Glow Bolt
        c.set_stroke(Fill::Solid(bolt_col.with_alpha(0.85)));
        c.set_line_width((2.8 + val * 1.8) * user_scale);
        c.set_shadow(bolt_col, (16.0 + val * 10.0) * user_scale);
        c.stroke_line(cx + cos_a * inner_r, cy + sin_a * inner_r, mid_x, mid_y);
        c.stroke_line(mid_x, mid_y, pin_x, pin_y);

        // Core White Filament
        c.set_stroke(Fill::Solid(spark_white));
        c.set_line_width(1.2 * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_line(cx + cos_a * inner_r, cy + sin_a * inner_r, mid_x, mid_y);
        c.stroke_line(mid_x, mid_y, pin_x, pin_y);
    }

    // 4. Central Core Disc
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(arc_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(arc_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    let _ = p_col;
    c.set_global_alpha(1.0);
    c.restore();
}
