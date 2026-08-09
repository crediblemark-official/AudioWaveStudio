//! Laser Equalizer Wall style renderer (`laserWall`) — Synthwave LED & Wireframe Mesh Wave Engine.
//!
//! Masterpiece replication of synthwave audio visualizer artwork:
//! - Segmented LED equalizer bars with bright golden yellow caps, orange transitions, and magenta bases.
//! - Mirrored inverted equalizer bar reflections extending down into the floor grid.
//! - Perspective 3D floor grid with bright cyan grid lines and sliding neon light bars (cyan & magenta).
//! - 3D Wireframe Mesh Ribbon: A multi-layered flowing audio wave fabric with cross-hatched grid lines and glowing core.
//! - Digital Binary Data Band: Subtle matrix data streams behind the mesh wave.

use std::f32::consts::PI;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

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
    let bar_count    = ctx.config.reactivity.bar_count.clamp(24, 64);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq      = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let step_f = (freq.len() / bar_count).max(1);

    let cx      = width  * 0.5 + pos_offset_x;
    let vp_y    = height * 0.45 + pos_offset_y; // Vanishing point
    let base_y  = height * 0.65 + pos_offset_y; // Equalizer floor baseline
    let max_h   = height * 0.46 * user_scale;

    // Bar layout
    let total_w = width * 0.90 * user_scale;
    let bar_w   = (total_w / bar_count as f32).clamp(3.0, 32.0);
    let gap     = (bar_w * 0.14).clamp(1.0, 4.0);
    let b_w     = bar_w - gap;
    let start_x = cx - total_w * 0.5;

    let seg_count = 18usize;
    let seg_gap   = 1.8 * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. BACKGROUND — Deep Midnight Synthwave Gradient & Blue Horizon Glow
    // -------------------------------------------------------------------------
    let _bg = Fill::linear_gradient(
        cx, 0.0, cx, height,
        &[
            (0.00, Color::hex("#020109")),
            (0.38, mix(Color::hex("#090326"), p_col.with_alpha(1.0), 0.12)),
            (0.65, mix(Color::hex("#12042b"), acc_col.with_alpha(1.0), 0.10)),
            (1.00, Color::hex("#010006")),
        ],
    );
//     c.set_fill(bg);
//     c.fill_rect(0.0, 0.0, width, height);

    // Horizon blue/violet atmospheric bloom
    let hglow = Fill::radial_gradient(
        cx, vp_y, 0.0, cx, vp_y, width * 0.75,
        &[
            (0.00, Color::rgba(0.0, 0.55, 1.0, 0.38 + be * 0.15)),
            (0.35, Color::rgba(0.6, 0.1, 0.9, 0.18 + be * 0.08)),
            (0.70, mix(glow, Color::rgba(0.0, 0.2, 0.6, 0.05), 0.5)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(hglow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 2. DIGITAL DATA MATRIX BAND (Binary / Matrix streams behind bars)
    // -------------------------------------------------------------------------
    let data_cols = 48usize;
    let data_rows = 10usize;
    let data_area_w = total_w * 1.05;
    let data_top = vp_y - 20.0;
    let data_bot = base_y + 15.0;

    for dr in 0..data_rows {
        for dc in 0..data_cols {
            let tx = dc as f32 / (data_cols - 1) as f32;
            let ty = dr as f32 / (data_rows - 1) as f32;
            let dx = cx - data_area_w * 0.5 + tx * data_area_w;
            let dy = data_top + ty * (data_bot - data_top);

            // Pseudo-random binary character indicator ('1' or '0' dash)
            let val_seed = ((dc * 17 + dr * 31 + (frame_time * 2.0) as usize) % 5) == 0;
            if val_seed {
                let col_dist = (tx - 0.5).abs() * 2.0;
                let alpha = (0.28 - col_dist * 0.18).clamp(0.03, 0.28);
                let char_col = Color::rgba(0.9, 0.3, 0.9, alpha);
                c.set_fill(Fill::Solid(char_col));
                c.set_shadow(Color::TRANSPARENT, 0.0);
                c.fill_rect(dx, dy, 2.5 * user_scale, 4.0 * user_scale);
            }
        }
    }

    // -------------------------------------------------------------------------
    // 3. PERSPECTIVE FLOOR GRID & SLIDING NEON LASER BARS
    // -------------------------------------------------------------------------
    let floor_top_y  = base_y;
    let floor_bot_y  = height + 10.0;
    let floor_half_w = width * 0.64;

    // Longitudinal grid lines (perspective converging to horizon)
    let lng_count = 18usize;
    for i in 0..=lng_count {
        let t = i as f32 / lng_count as f32;
        let bot_x = cx - floor_half_w + t * floor_half_w * 2.0;
        let lc = mix(Color::rgba(0.0, 0.80, 1.0, 0.7), Color::rgba(0.7, 0.1, 1.0, 0.7), t)
            .with_alpha(0.32 + be * 0.12);
        c.set_stroke(Fill::Solid(lc));
        c.set_line_width((1.0 + be * 0.5) * user_scale);
        c.set_shadow(lc, (4.0 + be * 3.0) * user_scale);
        c.stroke_line(cx, floor_top_y, bot_x, floor_bot_y);
    }

    // Latitudinal grid lines (horizontal perspective scaling)
    let lat_count = 12usize;
    for j in 0..=lat_count {
        let t = j as f32 / lat_count as f32;
        let fy = floor_top_y + t * (floor_bot_y - floor_top_y);
        let hw = floor_half_w * (0.02 + t * 0.98);
        let alpha = (0.38 - t * 0.22).max(0.05);
        let lc = mix(Color::rgba(0.0, 0.70, 1.0, 0.8), s_col, t).with_alpha(alpha + be * 0.05);
        c.set_stroke(Fill::Solid(lc));
        c.set_line_width((0.9 + t * 0.6) * user_scale);
        c.set_shadow(lc, (3.0 + t * 3.0) * user_scale);
        c.stroke_line(cx - hw, fy, cx + hw, fy);
    }

    // Sliding Neon Laser Bars on Floor Grid (Cyan & Red/Pink Rectangular Dashes)
    for s in 0..10usize {
        let sf = s as f32;
        let speed = 0.30 + (s % 4) as f32 * 0.14;
        let t_pos = (frame_time * speed + sf * 0.23) % 1.0;
        let fy = floor_top_y + t_pos * (floor_bot_y - floor_top_y);
        let hw = floor_half_w * (0.02 + t_pos * 0.98);

        let streak_len = (35.0 + (s % 4) as f32 * 20.0) * (0.35 + t_pos * 0.85) * user_scale;
        let streak_x = cx - hw + ((sf * 0.31).fract() * 2.0 * hw).clamp(0.0, (2.0 * hw - streak_len).max(1.0));

        let is_cyan = s % 2 == 0;
        let streak_col = if is_cyan {
            Color::rgba(0.0, 0.95, 1.0, 0.90) // Bright Cyan Laser Bar
        } else {
            Color::rgba(1.0, 0.10, 0.50, 0.90) // Bright Pink/Red Laser Bar
        };

        // Render laser bar with thick line and core
        c.set_stroke(Fill::Solid(streak_col));
        c.set_line_width((3.0 + t_pos * 2.0) * user_scale);
        c.set_shadow(streak_col, (10.0 + t_pos * 6.0) * user_scale);
        c.stroke_line(streak_x, fy, streak_x + streak_len, fy);

        // Core highlight
        c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.90)));
        c.set_line_width(1.0 * user_scale);
        c.stroke_line(streak_x, fy, streak_x + streak_len, fy);
    }

    // -------------------------------------------------------------------------
    // 4. SEGMENTED LED EQUALIZER BARS & MIRRORED REFLECTIONS
    // -------------------------------------------------------------------------
    let mut bar_heights: Vec<f32> = Vec::with_capacity(bar_count);

    for i in 0..bar_count {
        let bin_k = (i * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;
        let h = (fv * max_h * sensitivity + be * max_h * 0.08)
            .clamp(4.0 * user_scale, max_h);
        bar_heights.push(h);

        let x = start_x + i as f32 * bar_w;
        let bar_t = i as f32 / (bar_count - 1).max(1) as f32;

        let seg_unit  = h / seg_count as f32;
        let seg_inner = (seg_unit - seg_gap).max(1.5 * user_scale);

        let active_segs = (fv * seg_count as f32 * sensitivity + be * 2.0)
            .clamp(0.0, seg_count as f32) as usize;
        let peak_seg = active_segs.saturating_sub(1);

        for seg in 0..seg_count {
            let sy = base_y - (seg as f32 + 1.0) * seg_unit;
            let seg_t = seg as f32 / (seg_count - 1) as f32; // 0=bottom, 1=top

            let lit = seg < active_segs;
            let is_peak = seg == peak_seg && lit;

            if lit {
                // Precise synthwave color gradient:
                // Top caps (seg_t > 0.75): Bright Golden Yellow (#FFE000)
                // Mid body (0.35..0.75): Bright Orange -> Pink/Magenta
                // Base (0.0..0.35): Deep Crimson Red / Magenta (#D00060)
                let seg_col = if is_peak || seg_t > 0.78 {
                    Color::rgba(1.0, 0.92, 0.10, 1.0) // Golden Yellow Top
                } else if seg_t > 0.50 {
                    mix(
                        Color::rgba(1.0, 0.30, 0.60, 1.0),
                        Color::rgba(1.0, 0.75, 0.05, 1.0),
                        (seg_t - 0.50) / 0.28,
                    )
                } else if seg_t > 0.25 {
                    mix(
                        Color::rgba(0.85, 0.10, 0.50, 1.0),
                        Color::rgba(1.0, 0.30, 0.60, 1.0),
                        (seg_t - 0.25) / 0.25,
                    )
                } else {
                    mix(
                        mix(p_col, Color::rgba(0.70, 0.05, 0.40, 1.0), 0.6),
                        Color::rgba(0.85, 0.10, 0.50, 1.0),
                        seg_t / 0.25,
                    )
                };

                c.set_fill(Fill::Solid(seg_col));
                let shadow_r = if is_peak { 14.0 + fv * 8.0 } else { 4.0 + seg_t * 6.0 };
                c.set_shadow(seg_col, shadow_r * user_scale);
                c.fill_rect(x, sy, b_w, seg_inner);
            } else {
                // Dim inactive segment slot
                c.set_fill(Fill::Solid(Color::rgba(0.15, 0.05, 0.28, 0.40)));
                c.set_shadow(Color::TRANSPARENT, 0.0);
                c.fill_rect(x, sy, b_w, seg_inner);
            }
        }

        // --- MIRRORED INVERTED EQUALIZER REFLECTION BELOW BASELINE ---
        let refl_h = h * 0.50;
        let refl_active_segs = (active_segs as f32 * 0.60) as usize;
        let refl_seg_unit = refl_h / seg_count as f32;
        let refl_seg_inner = (refl_seg_unit - seg_gap * 0.5).max(1.0 * user_scale);

        for seg in 0..refl_active_segs {
            let ry = base_y + seg as f32 * refl_seg_unit + 3.0;
            let alpha = (0.50 - seg as f32 / seg_count as f32 * 0.42).clamp(0.04, 0.50);

            let refl_col = mix(
                Color::rgba(0.90, 0.10, 0.50, alpha),
                Color::rgba(0.20, 0.60, 1.0, alpha),
                bar_t,
            );
            c.set_fill(Fill::Solid(refl_col));
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.fill_rect(x, ry, b_w, refl_seg_inner);
        }
    }

    // -------------------------------------------------------------------------
    // 5. 3D WIREFRAME MESH AUDIO WAVE SHROUD / RIBBON (DENSE NET FABRIC)
    // -------------------------------------------------------------------------
    let num_sub_waves = 6usize; // 6 stacked longitudinal strands forming a mesh
    let mut mesh_grid: Vec<Vec<(f32, f32)>> = Vec::with_capacity(num_sub_waves);

    for w_idx in 0..num_sub_waves {
        let w_t = w_idx as f32 / (num_sub_waves - 1) as f32; // 0..1 top to bottom of mesh
        let v_offset = (w_t - 0.5) * 28.0 * user_scale;

        let mut strand: Vec<(f32, f32)> = Vec::with_capacity(bar_count + 1);

        for i in 0..=bar_count {
            let t = i as f32 / bar_count as f32;
            let x = start_x + t * total_w;

            let bin_k = (i.min(bar_count - 1) * step_f).min(freq.len().saturating_sub(1));
            let fv = freq[bin_k] as f32 / 255.0;

            let wave_phase = frame_time * 1.6 + t * PI * 4.0 + w_t * 0.8;
            let bar_h_curr = bar_heights.get(i.min(bar_count - 1)).copied().unwrap_or(0.0);

            let wave_amp = bar_h_curr * 0.35 + (wave_phase * 0.7).cos() * 18.0 * user_scale;
            let mid_y = base_y - bar_h_curr * (0.42 + fv * 0.18);

            let y = mid_y + wave_phase.sin() * wave_amp + v_offset;
            strand.push((x, y));
        }
        mesh_grid.push(strand);
    }

    // A. Draw Horizontal Mesh Strands (Longitudinal Wavy Lines)
    for (w_idx, strand) in mesh_grid.iter().enumerate() {
        let w_t = w_idx as f32 / (num_sub_waves - 1) as f32;
        let is_center = w_idx == num_sub_waves / 2;

        let mesh_col = mix(
            Color::rgba(1.0, 0.20, 0.80, 0.90), // Bright Pink
            Color::rgba(0.9, 0.60, 1.0, 0.90),  // Light Violet
            w_t,
        );

        if is_center {
            // Main glowing core strand
            c.set_stroke(Fill::Solid(mesh_col.with_alpha(0.30)));
            c.set_line_width(14.0 * user_scale);
            c.set_shadow(mesh_col, 20.0 * user_scale);
            c.stroke_polyline(strand);

            c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.98)));
            c.set_line_width(2.0 * user_scale);
            c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.90), 8.0 * user_scale);
            c.stroke_polyline(strand);
        } else {
            // Auxiliary mesh strands
            let alpha = 0.45 - (w_t - 0.5).abs() * 0.5;
            c.set_stroke(Fill::Solid(mesh_col.with_alpha(alpha.max(0.12))));
            c.set_line_width(1.0 * user_scale);
            c.set_shadow(mesh_col, 6.0 * user_scale);
            c.stroke_polyline(strand);
        }
    }

    // B. Draw Vertical/Cross-Hatching Grid Lines & Matrix Nodes connecting sub-waves
    c.set_shadow(Color::TRANSPARENT, 0.0);
    let num_pts = mesh_grid[0].len();

    for k in (0..num_pts).step_by(2) {
        let top_pt = mesh_grid[0][k];
        let bot_pt = mesh_grid[num_sub_waves - 1][k];

        let cross_col = Color::rgba(1.0, 0.35, 0.90, 0.45);
        c.set_stroke(Fill::Solid(cross_col));
        c.set_line_width(0.8 * user_scale);
        c.stroke_line(top_pt.0, top_pt.1, bot_pt.0, bot_pt.1);

        // Matrix dots at grid intersections
        for w_idx in 0..num_sub_waves {
            let (node_x, node_y) = mesh_grid[w_idx][k];
            c.set_fill(Fill::Solid(Color::rgba(1.0, 0.85, 1.0, 0.70)));
            c.fill_circle(node_x, node_y, 1.3 * user_scale);
        }
    }

    // -------------------------------------------------------------------------
    // 6. BASELINE NEON HORIZON STRIP
    // -------------------------------------------------------------------------
    let bl_col = Color::rgba(0.0, 0.90, 1.0, 1.0); // Bright Cyan Horizon Line
    let bl_grad = Fill::linear_gradient(
        start_x, base_y, start_x + total_w, base_y,
        &[
            (0.00, Color::TRANSPARENT),
            (0.08, bl_col.with_alpha(0.70 + be * 0.20)),
            (0.50, Color::rgba(1.0, 1.0, 1.0, 0.98)),
            (0.92, bl_col.with_alpha(0.70 + be * 0.20)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_stroke(bl_grad);
    c.set_line_width((2.2 + be * 1.5) * user_scale);
    c.set_shadow(bl_col, (14.0 + be * 8.0) * user_scale);
    c.stroke_line(start_x, base_y, start_x + total_w, base_y);

    // -------------------------------------------------------------------------
    // 7. BEAT FLASH
    // -------------------------------------------------------------------------
    if bs > 0.62 {
        let fa = (bs - 0.62) * 0.25;
        c.set_fill(Fill::Solid(mix(p_col, Color::rgba(1.0, 1.0, 1.0, fa), 0.4).with_alpha(fa)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
//         c.fill_rect(0.0, 0.0, width, height);
    }

    let _ = (s_col, glow, PI);

    c.set_global_alpha(1.0);
    c.restore();
}
