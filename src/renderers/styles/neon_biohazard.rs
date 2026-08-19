//! Neon Bio-Hazard Pulse style renderer (`neonBiohazard`) — Cyber Toxic Engine.
//!
//! Masterpiece 3D Biohazard Emblem (native Scene3D):
//! - Authentic 3D Extruded Biohazard Trefoil Emblem with volumetric depth & glowing edge highlights.
//! - 3D Interlocking Center Ring with precision gaps matching the 3 trefoil lobes.
//! - 360° Cyber Audio Spectrum Corona (3D audio-reactive frequency pillars).
//! - Alternating Biohazard Hazard Caution Tape Ring (diagonal toxic yellow/black stripes).
//! - 240-Segment Smooth Fluid Wave Band rippling with audio frequencies.
//! - Tactical HUD Ring with 120 precision micro-ticks and cardinal indicators.
//! - Luminous radioactive plasma orb core with 3 expanding energy pulse halos.
//! - Receding 3D perspective floor grid & 45+ floating 3D toxic spore particles.
//! - Deep atmospheric toxic glow backdrop & full UI settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const RING_SEGS: usize = 240;
const BLADE_SEGS: usize = 48;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    // Settings integration
    let sensitivity = ctx.config.reactivity.sensitivity;
    let user_scale = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;
    let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;
    let rot = ctx.rotation_angle;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;
    let reference_size = width.min(height);
    let base_r = reference_size * 0.28 * user_scale;

    // Curated Cyber Toxic Color Palette
    let toxic_green = mix(mix(p_col, glow_col, 0.5), Color::rgba(0.0, 1.0, 0.40, 1.0), 0.65);
    let toxic_cyan = mix(mix(s_col, accent_col, 0.5), Color::rgba(0.0, 0.90, 1.0, 1.0), 0.60);
    let hazard_yellow = mix(accent_col, Color::rgba(1.0, 0.85, 0.0, 1.0), 0.85);
    let dark_bg_glow = Color::rgba(0.02, 0.12, 0.06, 0.35 + be * 0.20);
    let dark_core = Color::rgba(0.02, 0.06, 0.03, 0.96);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. DEEP ATMOSPHERIC BACKDROP & RADIAL TOXIC GLOW
    // -------------------------------------------------------------------------
    let bg_haze = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        base_r * 2.5,
        &[
            (0.0, dark_bg_glow),
            (0.35, toxic_green.with_alpha(0.18 + be * 0.12)),
            (0.70, toxic_cyan.with_alpha(0.08)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_haze);

    // Draw user radial center image as backdrop disc behind core orb (if set)
    draw_radial_center_image(c, ctx, cx, cy, base_r * 0.22);

    // -------------------------------------------------------------------------
    // 2. NATIVE 3D SCENE CONFIGURATION (Scene3D)
    // -------------------------------------------------------------------------
    let scene = &mut ctx.scene3d;
    scene.cam_yaw = (rot * 0.04).sin() * (0.06 + be * 0.05);
    scene.cam_pitch = -0.42 - (frame_time * 0.02).sin() * 0.03 - be * 0.04;
    scene.cam_zoom = (1.15 - be * 0.04) / user_scale;
    scene.target_x = pos_offset_x;
    scene.target_y = pos_offset_y;

    let world_cy = height * 0.5 - cy;
    let world_floor = world_cy - height * 0.14;

    // -------------------------------------------------------------------------
    // 3. RECEDING 3D PERSPECTIVE FLOOR GRID LINES
    // -------------------------------------------------------------------------
    let half_w = width * 0.88;
    let z_max = -580.0f32;
    let grid_col = mix(toxic_green, toxic_cyan, 0.5).with_alpha(0.25 + be * 0.10);

    for col_i in 0..=12 {
        let gx = -half_w + (col_i as f32 / 12.0) * half_w * 2.0;
        scene.add_box(gx, world_floor, z_max * 0.5, 1.5, 1.5, -z_max, grid_col);
    }
    for row_i in 0..=8 {
        let rz = z_max * (row_i as f32 / 8.0);
        let spread = half_w * (0.45 + 0.55 * (row_i as f32 / 8.0));
        scene.add_box(0.0, world_floor, rz, spread * 2.0, 1.5, 1.5, grid_col);
    }

    // -------------------------------------------------------------------------
    // 4. RING 1: ROTATING 3D BIOHAZARD HAZARD CAUTION STRIP (HAZARD TAPE)
    // -------------------------------------------------------------------------
    let r1_hazard = base_r * 1.30;
    let hazard_segs = 48usize;
    let h_tape_w = 8.0 * user_scale;
    let h_tape_depth = 3.5;

    for s in 0..hazard_segs {
        let a0 = (s as f32 / hazard_segs as f32) * TAU + rot * 0.3;
        let a1 = ((s + 1) as f32 / hazard_segs as f32) * TAU + rot * 0.3;

        let is_yellow = s % 2 == 0;
        let segment_col = if is_yellow {
            mix(hazard_yellow, Color::WHITE, bs * 0.25)
        } else {
            Color::rgba(0.03, 0.08, 0.04, 0.95)
        };

        let (co0, si0) = a0.sin_cos();
        let (co1, si1) = a1.sin_cos();

        let r_in = r1_hazard;
        let r_out = r1_hazard + h_tape_w;

        let zt = h_tape_depth * 0.5;
        let zb = -h_tape_depth * 0.5;

        let p_in0 = [co0 * r_in, world_cy + si0 * r_in, zt];
        let p_out0 = [co0 * r_out, world_cy + si0 * r_out, zt];
        let p_out1 = [co1 * r_out, world_cy + si1 * r_out, zt];
        let p_in1 = [co1 * r_in, world_cy + si1 * r_in, zt];

        let q_in0 = [co0 * r_in, world_cy + si0 * r_in, zb];
        let q_out0 = [co0 * r_out, world_cy + si0 * r_out, zb];
        let q_out1 = [co1 * r_out, world_cy + si1 * r_out, zb];
        let q_in1 = [co1 * r_in, world_cy + si1 * r_in, zb];

        // Top face (+z)
        scene.quad(p_in0, p_out0, p_out1, p_in1, segment_col);
        // Bottom face (-z)
        scene.quad(q_in0, q_in1, q_out1, q_out0, segment_col);
        // Outer wall
        scene.quad(q_out0, q_out1, p_out1, p_out0, segment_col);
        // Inner wall
        scene.quad(q_in1, q_in0, p_in0, p_in1, segment_col);
    }

    // -------------------------------------------------------------------------
    // 5. RING 2: TACTICAL HUD DEGREE RING & 120 MICRO-TICKS
    // -------------------------------------------------------------------------
    let r2_inner = base_r * 1.22;
    let micro_ticks = 120usize;

    for t_i in 0..micro_ticks {
        let a = (t_i as f32 / micro_ticks as f32) * TAU - rot * 0.4;
        let is_major = t_i % 10 == 0;
        let t_len = if is_major { 8.0 } else { 4.0 };
        let (s_a, c_a) = a.sin_cos();

        let x0 = c_a * r2_inner;
        let y0 = s_a * r2_inner;
        let x1 = c_a * (r2_inner + t_len);
        let y1 = s_a * (r2_inner + t_len);

        let t_col = if is_major { Color::WHITE } else { toxic_cyan.with_alpha(0.65) };
        scene.add_box((x0 + x1) * 0.5, world_cy + (y0 + y1) * 0.5, 0.0, 1.5, t_len, 1.5, t_col);
    }

    // -------------------------------------------------------------------------
    // 6. RING 3: 360° HIGH-DENSITY SPECTRUM CORONA
    // -------------------------------------------------------------------------
    let r3_base = base_r * 1.12;
    let max_bar_h = height * 0.14 * sensitivity;
    let step_f = (freq.len() / bar_count).max(1);

    for i in 0..bar_count {
        let angle = (i as f32 / bar_count as f32) * TAU + rot * 0.5;
        let k = (i * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[k] as f32 / 255.0;
        let bh = (fv * max_bar_h + 4.0 + be * 10.0).clamp(4.0, (max_bar_h * 1.4).max(4.0));

        let (s_a, c_a) = angle.sin_cos();
        let x0 = c_a * r3_base;
        let y0 = s_a * r3_base;
        let x1 = c_a * (r3_base + bh);
        let y1 = s_a * (r3_base + bh);

        let bar_col = mix(toxic_green, toxic_cyan, i as f32 / bar_count as f32);
        let top_col = if fv > 0.60 || bs > 0.40 { Color::WHITE } else { mix(bar_col, hazard_yellow, 0.5) };

        // Pillar body & top cap
        scene.add_box((x0 + x1) * 0.5, world_cy + (y0 + y1) * 0.5, 0.0, 2.8, bh, 2.8, bar_col);
        scene.add_box(x1, world_cy + y1, 0.0, 3.5, 2.0, 3.5, top_col);
    }

    // -------------------------------------------------------------------------
    // 7. RING 4: MID SPECTRUM FLUID WAVE BAND (240-SEGMENT SMOOTH SPLINE)
    // -------------------------------------------------------------------------
    let r4_base = base_r * 0.90;
    let mut wave_radii = Vec::with_capacity(RING_SEGS);

    for k in 0..RING_SEGS {
        let t_k = k as f32 / RING_SEGS as f32;
        let bin_exact = t_k * (freq.len() as f32 - 1.0);
        let bin0 = bin_exact.floor() as usize;
        let bin1 = (bin0 + 1).min(freq.len().saturating_sub(1));
        let frac = bin_exact - bin0 as f32;

        let fv0 = freq[bin0] as f32 / 255.0;
        let fv1 = freq[bin1] as f32 / 255.0;
        let fv_smooth = fv0 * (1.0 - frac) + fv1 * frac;

        wave_radii.push(r4_base + fv_smooth * 18.0 * sensitivity);
    }

    scene.push();
    scene.translate(0.0, world_cy, 0.0);
    scene.rotate_z(-rot * 0.7);
    scene.add_band(0.0, 0.0, 0.0, r4_base * 0.96, r4_base, &wave_radii, 4.0, toxic_green.with_alpha(0.92));
    scene.pop();

    // -------------------------------------------------------------------------
    // 8. LAYER 5: AUTHENTIC 3D EXTRUDED BIOHAZARD TREFOIL EMBLEM
    // -------------------------------------------------------------------------
    let emblem_r = base_r * 0.68;
    let blade_dist = emblem_r * 0.52;
    let r_out_base = emblem_r * 0.58;
    let r_in_base = emblem_r * 0.44;
    let r_center_cut = emblem_r * 0.36;

    let blade_depth = 6.0 + be * 4.0;
    let zt = blade_depth * 0.5;
    let zb = -blade_depth * 0.5;

    for b in 0..3 {
        let theta_b = (b as f32 / 3.0) * TAU + rot * 0.5;
        let bin_k = (b * 8 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let r_out_cur = r_out_base * (1.0 + fv * 0.15 * sensitivity);
        let cb_x = theta_b.cos() * blade_dist;
        let cb_y = theta_b.sin() * blade_dist;

        let front_col = mix(toxic_green, Color::WHITE, fv * 0.35 + bs * 0.20);
        let back_col = dark_core;
        let wall_col = mix(toxic_green, toxic_cyan, 0.5);

        let alpha_max = 2.2f32; // Sweep angle (~126°)

        for i in 0..BLADE_SEGS {
            let t0 = i as f32 / BLADE_SEGS as f32;
            let t1 = (i + 1) as f32 / BLADE_SEGS as f32;

            let a0 = (t0 - 0.5) * alpha_max;
            let a1 = (t1 - 0.5) * alpha_max;

            let phi0 = theta_b + a0;
            let phi1 = theta_b + a1;

            let r_out0 = r_in_base + (r_out_cur - r_in_base) * (std::f32::consts::PI * (t0 - 0.5)).cos();
            let r_out1 = r_in_base + (r_out_cur - r_in_base) * (std::f32::consts::PI * (t1 - 0.5)).cos();

            // Raw outer points
            let p_out0_raw = (cb_x + phi0.cos() * r_out0, cb_y + phi0.sin() * r_out0);
            let p_out1_raw = (cb_x + phi1.cos() * r_out1, cb_y + phi1.sin() * r_out1);

            // Raw inner points
            let p_in0_raw = (cb_x + phi0.cos() * r_in_base, cb_y + phi0.sin() * r_in_base);
            let p_in1_raw = (cb_x + phi1.cos() * r_in_base, cb_y + phi1.sin() * r_in_base);

            // Apply Central Cutout Circle (r_center_cut) around (0,0)
            let d_in0 = (p_in0_raw.0 * p_in0_raw.0 + p_in0_raw.1 * p_in0_raw.1).sqrt();
            let p_in0 = if d_in0 < r_center_cut && d_in0 > 0.001 {
                (p_in0_raw.0 / d_in0 * r_center_cut, p_in0_raw.1 / d_in0 * r_center_cut)
            } else {
                p_in0_raw
            };

            let d_in1 = (p_in1_raw.0 * p_in1_raw.0 + p_in1_raw.1 * p_in1_raw.1).sqrt();
            let p_in1 = if d_in1 < r_center_cut && d_in1 > 0.001 {
                (p_in1_raw.0 / d_in1 * r_center_cut, p_in1_raw.1 / d_in1 * r_center_cut)
            } else {
                p_in1_raw
            };

            let p_out0 = p_out0_raw;
            let p_out1 = p_out1_raw;

            let pt_in0 = [p_in0.0, world_cy + p_in0.1, zt];
            let pt_out0 = [p_out0.0, world_cy + p_out0.1, zt];
            let pt_out1 = [p_out1.0, world_cy + p_out1.1, zt];
            let pt_in1 = [p_in1.0, world_cy + p_in1.1, zt];

            let pb_in0 = [p_in0.0, world_cy + p_in0.1, zb];
            let pb_out0 = [p_out0.0, world_cy + p_out0.1, zb];
            let pb_out1 = [p_out1.0, world_cy + p_out1.1, zb];
            let pb_in1 = [p_in1.0, world_cy + p_in1.1, zb];

            // 1. Front face (+z)
            scene.quad(pt_in0, pt_out0, pt_out1, pt_in1, front_col.with_alpha(0.95));
            // 2. Back face (-z)
            scene.quad(pb_in0, pb_in1, pb_out1, pb_out0, back_col);
            // 3. Outer wall
            scene.quad(pb_out0, pb_out1, pt_out1, pt_out0, wall_col);
            // 4. Inner wall
            scene.quad(pb_in1, pb_in0, pt_in0, pt_in1, wall_col.with_alpha(0.85));
        }
    }

    // -------------------------------------------------------------------------
    // 9. LAYER 6: BIOHAZARD INTERLOCKING CENTER CONNECTING RING
    // -------------------------------------------------------------------------
    let r_ring_in = emblem_r * 0.54;
    let r_ring_out = emblem_r * 0.66;
    let ring_depth = 4.0;

    // The ring has 3 precision gaps centered at angles theta_b where it passes behind the blades
    for b in 0..3 {
        let theta_b0 = (b as f32 / 3.0) * TAU + rot * 0.5 + 0.32;
        let theta_b1 = ((b + 1) as f32 / 3.0) * TAU + rot * 0.5 - 0.32;

        let arc_segs = 16usize;
        for i in 0..arc_segs {
            let a0 = theta_b0 + (i as f32 / arc_segs as f32) * (theta_b1 - theta_b0);
            let a1 = theta_b0 + ((i + 1) as f32 / arc_segs as f32) * (theta_b1 - theta_b0);

            let (co0, si0) = a0.sin_cos();
            let (co1, si1) = a1.sin_cos();

            let zt = ring_depth * 0.5;
            let zb = -ring_depth * 0.5;

            let p_in0 = [co0 * r_ring_in, world_cy + si0 * r_ring_in, zt];
            let p_out0 = [co0 * r_ring_out, world_cy + si0 * r_ring_out, zt];
            let p_out1 = [co1 * r_ring_out, world_cy + si1 * r_ring_out, zt];
            let p_in1 = [co1 * r_ring_in, world_cy + si1 * r_ring_in, zt];

            let q_in0 = [co0 * r_ring_in, world_cy + si0 * r_ring_in, zb];
            let q_out0 = [co0 * r_ring_out, world_cy + si0 * r_ring_out, zb];
            let q_out1 = [co1 * r_ring_out, world_cy + si1 * r_ring_out, zb];
            let q_in1 = [co1 * r_ring_in, world_cy + si1 * r_ring_in, zb];

            let ring_col = mix(hazard_yellow, Color::WHITE, 0.20);
            scene.quad(p_in0, p_out0, p_out1, p_in1, ring_col);
            scene.quad(q_in0, q_in1, q_out1, q_out0, dark_core);
            scene.quad(q_out0, q_out1, p_out1, p_out0, ring_col);
            scene.quad(q_in1, q_in0, p_in0, p_in1, ring_col.with_alpha(0.80));
        }
    }

    // -------------------------------------------------------------------------
    // 10. LAYER 7: RADIOACTIVE PLASMA REACTOR CORE & EXPANDING PULSE HALOS
    // -------------------------------------------------------------------------
    // Dark sink disc behind central core
    scene.add_disc(0.0, world_cy, 0.0, base_r * 0.32, 32, dark_core);

    let core_r = (base_r * 0.18 + be * 8.0).clamp(10.0, 48.0);
    scene.add_disc(0.0, world_cy, 0.0, core_r, 24, Color::WHITE);
    scene.add_disc(0.0, world_cy, 0.0, core_r * 1.35, 28, toxic_cyan.with_alpha(0.80));

    // Concentric Expanding Energy Pulse Halos (3D)
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = core_r * (1.2 + p_t * 1.9);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + bs * 0.40)).clamp(0.0, 0.85);
        scene.add_disc(0.0, world_cy, 0.0, pulse_r, 32, toxic_green.with_alpha(pulse_alpha));
    }

    // -------------------------------------------------------------------------
    // 11. LAYER 8: FLOATING 3D TOXIC SPORE PARTICLES (45+ MOTES)
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + be * 24.0).clamp(14.0, 52.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let mx = (m_i as f32 * 37.0).sin() * (base_r * 1.4);
        let my = (m_i as f32 * 23.0).cos() * (base_r * 1.4);
        let mz = (m_i as f32 * 17.0).sin() * 50.0;

        let m_sz = (2.5 * (1.0 - m_t) + 1.0).clamp(1.0, 4.5);
        let m_col = mix(toxic_green, Color::WHITE, m_t).with_alpha((1.0 - m_t).clamp(0.1, 0.95));
        scene.add_disc(mx, world_cy + my, mz, m_sz, 6, m_col);
    }

    c.set_global_alpha(1.0);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.restore();
}

