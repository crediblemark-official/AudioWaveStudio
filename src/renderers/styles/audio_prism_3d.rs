//! Audio Prism 3D style renderer (`audioPrism3D`) — Glass Optical Dispersion Engine.
//!
//! Dark Side of the Moon style glass prism:
//! - Triangular glass prism with realistic transparent shading & specular highlights.
//! - White laser beam enters the left face, disperses as audio-reactive rainbow out the right face.
//! - Each rainbow ray thickness reacts to its matching frequency band.
//! - Caustic light scatter on the floor below the prism.

use std::f32::consts::PI;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const RAINBOW_RAYS: usize = 36;

// Spectral hue table: ROYGBIV across the fan
fn spectral_color(t: f32) -> Color {
    // t: 0=red, 1=violet
    let hue = t * 270.0; // degrees, 0=red → 270=violet
    let h = hue / 60.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = if h < 1.0      { (1.0, x,   0.0) }
                    else if h < 2.0 { (x,   1.0, 0.0) }
                    else if h < 3.0 { (0.0, 1.0, x)   }
                    else if h < 4.0 { (0.0, x,   1.0) }
                    else             { (x,   0.0, 1.0) };
    Color::rgba(r, g, b, 0.92)
}

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

    // Prism centred slightly left of screen centre so the rainbow has room
    let cx = width  * 0.42 + pos_offset_x;
    let cy = height * 0.50 + pos_offset_y;

    // Prism geometry — equilateral-ish triangle
    let prism_size = 110.0 * user_scale;
    let prism_h    = prism_size * 0.866; // height of equilateral triangle

    // Vertices: apex top-centre, bottom-left, bottom-right
    let apex    = (cx,                  cy - prism_h * 0.55);
    let bot_l   = (cx - prism_size * 0.5, cy + prism_h * 0.45);
    let bot_r   = (cx + prism_size * 0.5, cy + prism_h * 0.45);

    // Beam enters horizontal from left, aimed at the midpoint of the left face
    let entry_pt_x = (apex.0 + bot_l.0) * 0.5;
    let entry_pt_y = (apex.1 + bot_l.1) * 0.5;
    // Exit point: midpoint of right face
    let exit_pt_x  = (apex.0 + bot_r.0) * 0.5;
    let exit_pt_y  = (apex.1 + bot_r.1) * 0.5;

    // Slow gentle sway with bass
    let sway = (frame_time * 0.4).sin() * 4.0 * user_scale * be;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. BACKGROUND — deep void + faint prism atmosphere
    // -------------------------------------------------------------------------
//     c.set_fill(Fill::Solid(Color::hex("#04050e")));
//     c.fill_rect(0.0, 0.0, width, height);

    let bg_glow = Fill::radial_gradient(
        cx, cy, 0.0, cx, cy, width * 0.55,
        &[
            (0.00, mix(glow, Color::rgba(0.20, 0.60, 1.0, 0.18 + be * 0.10), 0.5)),
            (0.50, mix(p_col, Color::rgba(0.05, 0.10, 0.28, 0.06), 0.5)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 2. INCOMING WHITE LASER BEAM (left edge → prism entry face)
    // -------------------------------------------------------------------------
    let beam_y = entry_pt_y + sway;

    // Soft outer glow halo
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.25)));
    c.set_line_width((22.0 + be * 10.0) * user_scale);
    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.60), 28.0 * user_scale);
    c.stroke_line(0.0, beam_y, entry_pt_x, beam_y);

    // Medium glow
    c.set_stroke(Fill::Solid(Color::rgba(0.80, 0.90, 1.0, 0.50)));
    c.set_line_width((8.0 + be * 4.0) * user_scale);
    c.set_shadow(Color::rgba(0.80, 0.90, 1.0, 0.80), 14.0 * user_scale);
    c.stroke_line(0.0, beam_y, entry_pt_x, beam_y);

    // Bright white core
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.98)));
    c.set_line_width((2.5 + be * 1.5) * user_scale);
    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 1.0), 8.0 * user_scale);
    c.stroke_line(0.0, beam_y, entry_pt_x, beam_y);

    // -------------------------------------------------------------------------
    // 3. GLASS PRISM — triangular polygon with glass shading
    // -------------------------------------------------------------------------
    let tri: Vec<(f32, f32)> = vec![
        (apex.0,  apex.1),
        (bot_l.0, bot_l.1),
        (bot_r.0, bot_r.1),
    ];

    // Glass body fill: blue-grey semi-transparent
    let glass_fill = Fill::linear_gradient(
        bot_l.0, apex.1,
        bot_r.0, bot_r.1,
        &[
            (0.00, Color::rgba(0.55, 0.72, 0.92, 0.28)),
            (0.40, Color::rgba(0.75, 0.88, 1.00, 0.22)),
            (0.70, Color::rgba(0.40, 0.58, 0.82, 0.32)),
            (1.00, Color::rgba(0.25, 0.40, 0.65, 0.35)),
        ],
    );
    c.set_fill(glass_fill);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_polygon(&tri);

    // Glass edge outline (thin bright rim)
    c.set_stroke(Fill::Solid(Color::rgba(0.88, 0.95, 1.0, 0.75)));
    c.set_line_width(2.0 * user_scale);
    c.set_shadow(Color::rgba(0.70, 0.90, 1.0, 0.80), 8.0 * user_scale);
    // stroke_polyline: close the triangle by repeating the first vertex
    let tri_closed: Vec<(f32, f32)> = vec![
        (apex.0,  apex.1),
        (bot_l.0, bot_l.1),
        (bot_r.0, bot_r.1),
        (apex.0,  apex.1),
    ];
    c.stroke_polyline(&tri_closed);

    // Internal specular highlight — thin bright streak along left face
    let spec_ax = apex.0 * 0.85 + bot_l.0 * 0.15;
    let spec_ay = apex.1 * 0.85 + bot_l.1 * 0.15;
    let spec_bx = apex.0 * 0.50 + bot_l.0 * 0.50;
    let spec_by = apex.1 * 0.50 + bot_l.1 * 0.50;
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.55)));
    c.set_line_width(1.5 * user_scale);
    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.70), 5.0 * user_scale);
    c.stroke_line(spec_ax, spec_ay, spec_bx, spec_by);

    // -------------------------------------------------------------------------
    // 4. RAINBOW DISPERSION FAN (exit right face → right edge of screen)
    // -------------------------------------------------------------------------
    let step_f    = (freq.len() / bar_count).max(1);
    // Fan spreads vertically from the exit point to the right screen edge
    // Top ray (red) exits upward, bottom ray (violet) exits downward
    let fan_top_y = exit_pt_y - height * 0.40 * user_scale;
    let fan_bot_y = exit_pt_y + height * 0.40 * user_scale;
    let fan_end_x = width;

    for r in 0..RAINBOW_RAYS {
        let t = r as f32 / (RAINBOW_RAYS - 1) as f32; // 0=top(red)→1=bot(violet)

        let bin_k = (r * step_f / ((RAINBOW_RAYS / bar_count.max(1)).max(1)))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let end_y = fan_top_y + t * (fan_bot_y - fan_top_y)
            + (t * PI * 2.0).sin() * fv * 22.0 * sensitivity; // subtle audio wave on each ray

        let ray_col = spectral_color(t);
        let ray_w   = (2.5 + fv * 5.0 * sensitivity + be * 2.0) * user_scale;

        // Outer glow
        c.set_stroke(Fill::Solid(ray_col.with_alpha(0.30 + fv * 0.15)));
        c.set_line_width(ray_w * 4.0);
        c.set_shadow(ray_col, (10.0 + fv * 8.0) * user_scale);
        c.stroke_line(exit_pt_x, exit_pt_y, fan_end_x, end_y);

        // Bright core
        c.set_stroke(Fill::Solid(ray_col));
        c.set_line_width(ray_w);
        c.set_shadow(ray_col, (6.0 + fv * 6.0) * user_scale);
        c.stroke_line(exit_pt_x, exit_pt_y, fan_end_x, end_y);
    }

    // -------------------------------------------------------------------------
    // 5. PRISM EXIT FLARE — bright white-hot point where beam exits
    // -------------------------------------------------------------------------
    let flare_r = (10.0 + bs * 16.0 + be * 8.0) * user_scale;
    let flare_grad = Fill::radial_gradient(
        exit_pt_x, exit_pt_y, 0.0,
        exit_pt_x, exit_pt_y, flare_r * 2.5,
        &[
            (0.00, Color::rgba(1.0, 1.0, 1.0, 0.98)),
            (0.30, Color::rgba(0.90, 0.96, 1.0, 0.70 + bs * 0.15)),
            (0.70, Color::rgba(0.50, 0.75, 1.0, 0.25)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(flare_grad);
    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.90), (18.0 + bs * 12.0) * user_scale);
    c.fill_circle(exit_pt_x, exit_pt_y, flare_r);

    // Entry point flare (smaller)
    c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.85)));
    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.80), 10.0 * user_scale);
    c.fill_circle(entry_pt_x, beam_y, (5.0 + be * 4.0) * user_scale);

    // -------------------------------------------------------------------------
    // 6. CAUSTIC FLOOR — faint rainbow patches below prism (refracted light)
    // -------------------------------------------------------------------------
    let floor_y = bot_l.1 + 18.0 * user_scale;
    for r in 0..6usize {
        let t = r as f32 / 5.0;
        let caus_col = spectral_color(t);
        let caus_x = bot_l.0 + t * (bot_r.0 - bot_l.0) + (frame_time * 0.6 + t * 1.2).sin() * 8.0;
        let caus_r = (8.0 + t * 6.0) * user_scale;
        let caus_grad = Fill::radial_gradient(
            caus_x, floor_y, 0.0,
            caus_x, floor_y, caus_r * 2.5,
            &[
                (0.00, caus_col.with_alpha(0.30 + be * 0.12)),
                (0.60, caus_col.with_alpha(0.10)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(caus_grad);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_circle(caus_x, floor_y, caus_r * 2.5);
    }

    // suppress unused
    let _ = (s_col, acc_col, glow, sway);

    c.set_global_alpha(1.0);
    c.restore();
}
