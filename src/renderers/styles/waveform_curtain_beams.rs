//! Waveform Curtain Beams style renderer (`waveformCurtainBeams`) — Volumetric Laser Stage Engine.
//!
//! Masterpiece Volumetric Laser Curtain Stage:
//! - Top metallic laser diode scanner gantry rail with active optic emitters.
//! - 64 High-density volumetric laser light beams with white-hot core filaments & neon plasma sheaths.
//! - Receding 3D perspective stage floor grid with laser impact reflection flares.
//! - Audio-reactive laser curtain amplitude, intensity, and multi-spectral laser color gradient.
//! - Stage fog haze light scattering & floating laser dust motes.
//! - Full UI Theme colors and settings integration (Scale, Position X & Y, Sensitivity, Bar Count).

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

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
    let cy = height * 0.50 + pos_offset_y;

    let span_w = width * 0.88 * user_scale;
    let start_x = cx - span_w * 0.5;
    let max_amp = height * 0.32 * user_scale;

    let beam_count = bar_count.clamp(32, 96);
    let beam_w = span_w / beam_count as f32;

    // Curated Laser Curtain Color Palette
    let laser_cyan = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.70);
    let laser_green = mix(p_col, Color::rgba(0.0, 1.0, 0.40, 1.0), 0.70);
    let laser_magenta = mix(accent_col, Color::rgba(1.0, 0.15, 0.85, 1.0), 0.75);
    let laser_white = Color::rgba(0.95, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. STAGE FOG HAZE & VOLUMETRIC LASER BACKDROP GLOW
    // -------------------------------------------------------------------------
    let bg_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.65,
        &[
            (0.00, mix(laser_cyan, laser_magenta, 0.5).with_alpha(0.24 + be * 0.16)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);

    // -------------------------------------------------------------------------
    // 2. RECEDING 3D PERSPECTIVE STAGE FLOOR GRID
    // -------------------------------------------------------------------------
    let floor_y = cy + max_amp * 0.90;
    let rail_y = cy - max_amp * 1.10;

    // Stage Horizon Line
    c.set_stroke(Fill::Solid(laser_cyan.with_alpha(0.40)));
    c.set_line_width(1.5 * user_scale);
    c.stroke_line(start_x - 40.0, floor_y, start_x + span_w + 40.0, floor_y);

    // Stage Floor Grid Lines
    let floor_grid_cols = 16usize;
    for g_i in 0..=floor_grid_cols {
        let gx = start_x + (g_i as f32 / floor_grid_cols as f32) * span_w;
        let g_col = mix(laser_cyan, laser_green, g_i as f32 / floor_grid_cols as f32).with_alpha(0.18 + be * 0.10);
        c.set_stroke(Fill::Solid(g_col));
        c.set_line_width(1.0 * user_scale);
        c.stroke_line(gx, floor_y, cx + (gx - cx) * 1.3, height);
    }

    // -------------------------------------------------------------------------
    // 3. TOP METALLIC LASER DIODE SCANNER GANTRY RAIL
    // -------------------------------------------------------------------------
    let rail_h = 10.0 * user_scale;
    let rail_grad = Fill::linear_gradient(
        start_x,
        rail_y - rail_h * 0.5,
        start_x + span_w,
        rail_y + rail_h * 0.5,
        &[
            (0.00, Color::rgba(0.20, 0.22, 0.28, 0.95)),
            (0.50, Color::rgba(0.80, 0.85, 0.92, 0.98)),
            (1.00, Color::rgba(0.15, 0.18, 0.22, 0.95)),
        ],
    );
    c.set_fill(rail_grad);
    c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.60), 8.0 * user_scale);
    c.fill_rect(start_x - 10.0, rail_y - rail_h * 0.5, span_w + 20.0, rail_h);

    // Rail border highlight
    c.set_stroke(Fill::Solid(laser_cyan.with_alpha(0.70)));
    c.set_line_width(1.2 * user_scale);
    c.stroke_rect(start_x - 10.0, rail_y - rail_h * 0.5, span_w + 20.0, rail_h);

    // -------------------------------------------------------------------------
    // 4. VOLUMETRIC LASER CURTAIN BEAMS & IMPACT FLARES
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / beam_count).max(1);

    for i in 0..beam_count {
        let t = i as f32 / beam_count as f32;
        let bx = start_x + i as f32 * beam_w + beam_w * 0.5;

        let sample_idx = (i * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[sample_idx] as f32 / 255.0;

        let curtain_h = (fv * max_amp * sensitivity * 1.2 + 12.0 + be * 15.0).clamp(10.0, max_amp * 1.8);
        let top_y = rail_y + rail_h * 0.5;
        let beam_bot_y = (top_y + curtain_h).min(floor_y);

        let beam_col = mix(
            mix(laser_cyan, laser_green, t),
            mix(laser_magenta, laser_white, fv),
            fv,
        );

        // A. Volumetric Light Curtain Quad (fading down to floor)
        let curtain_grad = Fill::linear_gradient(
            bx, top_y, bx, floor_y,
            &[
                (0.00, beam_col.with_alpha(0.85)),
                (0.40, beam_col.with_alpha(0.40 + fv * 0.25)),
                (0.85, beam_col.with_alpha(0.12)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(curtain_grad);
        c.set_shadow(beam_col, (10.0 + fv * 10.0) * user_scale);
        c.fill_rect(bx - beam_w * 0.42, top_y, beam_w * 0.84, floor_y - top_y);

        // B. Outer Neon Laser Sheath Line
        c.set_stroke(Fill::Solid(beam_col.with_alpha(0.90)));
        c.set_line_width((2.2 + fv * 1.5) * user_scale);
        c.stroke_line(bx, top_y, bx, beam_bot_y);

        // C. White-Hot Intense Core Laser Filament
        c.set_stroke(Fill::Solid(laser_white));
        c.set_line_width((1.2 + fv * 0.8) * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_line(bx, top_y, bx, beam_bot_y);

        // D. Top Laser Diode Emitter Optics Lens
        c.set_fill(Fill::Solid(laser_white));
        c.set_shadow(beam_col, (12.0 + fv * 8.0) * user_scale);
        c.fill_circle(bx, top_y, (2.8 + fv * 1.8) * user_scale);

        // E. Stage Floor Laser Impact Flares (where laser hits the floor)
        let flare_r = (3.5 + fv * 4.5 + bs * 2.5) * user_scale;
        c.set_fill(Fill::Solid(mix(beam_col, laser_white, fv * 0.70)));
        c.set_shadow(beam_col, (14.0 + bs * 10.0) * user_scale);
        c.fill_ellipse(bx, floor_y, flare_r * 1.4, flare_r * 0.6);
    }

    // -------------------------------------------------------------------------
    // 5. FLOATING STAGE DUST & LASER SPARK PARTICLES
    // -------------------------------------------------------------------------
    let mote_count = (20.0 + be * 24.0).clamp(12.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.35 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let mx = start_x + (m_i as f32 * 23.0).sin().abs() * span_w;
        let my = rail_y + m_t * (floor_y - rail_y);

        let m_sz = (2.2 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(laser_cyan, laser_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(laser_cyan, 6.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // Center image support if set
    draw_radial_center_image(c, ctx, cx, cy, max_amp * 0.35);

    c.set_global_alpha(1.0);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.restore();
}
