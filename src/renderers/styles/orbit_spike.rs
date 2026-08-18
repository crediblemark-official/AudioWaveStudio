//! Orbit Spike 3D visualizer style (`orbitSpike`).
//!
//! Tilted 3D audio rainbow spoke disc with vertical frequency fins, smooth depth-sorting,
//! and glowing rounded spoke tip motes (eliminating blunt/flat line caps).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const SPOKES: usize = 64;

fn rainbow_color(t: f32, alpha: f32) -> Color {
    let h = t * 6.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    Color::rgba(r, g, b, alpha)
}

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col  = theme_primary(theme);
    let s_col  = theme_secondary(theme);
    let _accent = theme_accent(theme);
    let glow   = theme_glow(theme);

    let sensitivity   = ctx.config.reactivity.sensitivity;
    let _bass_mult    = ctx.config.reactivity.bass_multiplier;
    let user_scale    = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;
    let bar_count    = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width  * 0.5 + pos_offset_x;
    let cy = height * 0.52 + pos_offset_y;

    // Disc radius & tilt factor
    let base_size  = width.min(height) * 0.42 * user_scale;
    let r_inner    = base_size * 0.22;
    let r_outer    = base_size * 0.95;
    let r_mid      = base_size * 0.58; // radius where frequency spike rises
    let tilt_y     = 0.35 + be * 0.04;  // Y-axis perspective compression (perspective tilt)
    let max_spike_h = height * 0.28 * user_scale;

    let rot_speed  = frame_time * 0.15; // Slow rotation of the rainbow wheel
    let step_f     = (freq.len() / bar_count.max(1)).max(1);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Ambient radial glow
    let bg_glow = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, base_size * 1.5,
        &[
            (0.00, mix(glow, Color::rgba(0.5, 0.2, 1.0, 0.3), 0.5).with_alpha(0.22 + be * 0.15)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08 + be * 0.04)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);

    // -------------------------------------------------------------------------
    // PRECOMPUTE ALL SPOKES (Coordinates & Audio Heights)
    // -------------------------------------------------------------------------
    struct SpokeData {
        _idx: usize,
        sin_val: f32, // for depth sorting
        _cos_val: f32,
        color: Color,
        x_in: f32,
        y_in: f32,
        x_out: f32,
        y_out: f32,
        x_mid: f32,
        y_mid: f32,
        spike_y: f32, // Y coordinate of vertical spike peak
        fv: f32,
    }

    let mut spokes: Vec<SpokeData> = Vec::with_capacity(SPOKES);

    for i in 0..SPOKES {
        let t = i as f32 / SPOKES as f32; // 0..1
        let angle = t * TAU + rot_speed;

        let sin_a = angle.sin();
        let cos_a = angle.cos();

        let mirrored_t = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
        let bin_k = ((mirrored_t * bar_count as f32 * 0.5) as usize * step_f)
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let spike_h = (fv * max_spike_h * sensitivity + be * max_spike_h * 0.12)
            .clamp(2.0 * user_scale, max_spike_h);

        // 3D Tilted Plane projections
        let x_in  = cx + r_inner * cos_a;
        let y_in  = cy + r_inner * sin_a * tilt_y;

        let x_mid = cx + r_mid * cos_a;
        let y_mid = cy + r_mid * sin_a * tilt_y;

        let x_out = cx + r_outer * cos_a;
        let y_out = cy + r_outer * sin_a * tilt_y;

        let spike_y = y_mid - spike_h;

        let color = rainbow_color(t, 0.95);

        spokes.push(SpokeData {
            _idx: i,
            sin_val: sin_a,
            _cos_val: cos_a,
            color,
            x_in,
            y_in,
            x_out,
            y_out,
            x_mid,
            y_mid,
            spike_y,
            fv,
        });
    }

    // Sort spokes back-to-front (sin_val ascending -> back spokes rendered first)
    let mut render_order: Vec<usize> = (0..SPOKES).collect();
    render_order.sort_by(|&a, &b| spokes[a].sin_val.partial_cmp(&spokes[b].sin_val).unwrap());

    // -------------------------------------------------------------------------
    // RENDER DISC & FREQUENCY SPIKES (PAINTER'S ALGORITHM)
    // -------------------------------------------------------------------------
    let draw_spoke = |c: &mut GpuCanvas, sp: &SpokeData, next_sp: &SpokeData| {
        let depth_t = (sp.sin_val * 0.5 + 0.5).clamp(0.0, 1.0);
        let alpha = 0.55 + depth_t * 0.42;
        let spoke_col = sp.color.with_alpha(alpha);

        // 1. Spoke Base Line
        c.set_stroke(Fill::Solid(spoke_col));
        c.set_line_width((1.2 + depth_t * 1.0 + sp.fv * 1.5) * user_scale);
        c.set_shadow(spoke_col, (4.0 + sp.fv * 8.0) * user_scale);
        c.stroke_line(sp.x_in, sp.y_in, sp.x_out, sp.y_out);

        // Rounded tip motes at outer spoke tips (removes flat "ujung buntung")
        c.set_fill(Fill::Solid(mix(spoke_col, Color::WHITE, 0.6)));
        c.set_shadow(spoke_col, 6.0 * user_scale);
        c.fill_circle(sp.x_out, sp.y_out, (1.8 + sp.fv * 1.5) * user_scale);

        // 2. Vertical Audio Frequency Fin / Spike
        let spike_poly = vec![
            (sp.x_mid, sp.y_mid),
            (sp.x_mid, sp.spike_y),
            (next_sp.x_mid, next_sp.spike_y),
            (next_sp.x_mid, next_sp.y_mid),
        ];

        let spike_fill = Fill::linear_gradient(
            sp.x_mid, sp.spike_y,
            sp.x_mid, sp.y_mid,
            &[
                (0.00, Color::rgba(1.0, 1.0, 1.0, 0.95)),
                (0.30, spoke_col.with_alpha(0.90)),
                (1.00, spoke_col.with_alpha(0.60)),
            ],
        );
        c.set_fill(spike_fill);
        c.set_shadow(spoke_col, (8.0 + sp.fv * 12.0) * user_scale);
        c.fill_polygon(&spike_poly);

        // Bright top ridge line
        c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.92)));
        c.set_line_width((1.5 + sp.fv * 1.2) * user_scale);
        c.set_shadow(spoke_col, 6.0 * user_scale);
        c.stroke_line(sp.x_mid, sp.spike_y, next_sp.x_mid, next_sp.spike_y);

        // Glowing peak dot at spike vertex
        c.set_fill(Fill::Solid(Color::WHITE));
        c.set_shadow(glow, 10.0 * user_scale);
        c.fill_circle(sp.x_mid, sp.spike_y, (2.2 + sp.fv * 1.8) * user_scale);
    };

    // Render back spokes
    for &idx in &render_order {
        if spokes[idx].sin_val < 0.0 {
            let next_idx = (idx + 1) % SPOKES;
            draw_spoke(c, &spokes[idx], &spokes[next_idx]);
        }
    }

    let stroke_ellipse_poly = |c: &mut GpuCanvas, rx: f32, ry: f32| {
        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(65);
        for k in 0..=64 {
            let a = k as f32 / 64.0 * TAU;
            pts.push((cx + a.cos() * rx, cy + a.sin() * ry));
        }
        c.stroke_polyline(&pts);
    };

    // Inner glowing ring
    c.set_stroke(Fill::Solid(glow));
    c.set_line_width((2.5 + be * 2.0) * user_scale);
    c.set_shadow(glow, (14.0 + bs * 10.0) * user_scale);
    stroke_ellipse_poly(c, r_inner, r_inner * tilt_y);

    // Render front spokes
    for &idx in &render_order {
        if spokes[idx].sin_val >= 0.0 {
            let next_idx = (idx + 1) % SPOKES;
            draw_spoke(c, &spokes[idx], &spokes[next_idx]);
        }
    }

    // Outer perimeter neon ring
    c.set_stroke(Fill::Solid(p_col.with_alpha(0.65)));
    c.set_line_width(1.5 * user_scale);
    c.set_shadow(p_col, 8.0 * user_scale);
    stroke_ellipse_poly(c, r_outer, r_outer * tilt_y);

    c.set_global_alpha(1.0);
    c.restore();

}
