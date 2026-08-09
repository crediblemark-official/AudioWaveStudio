//! Hologram Stage style renderer (`hologramStage`) — 3D Luxury Hologram Arena Stage Engine.
//!
//! Masterpiece 100% luxury & detailed redesign:
//! Renders a high-end 3D futuristic hologram concert arena stage featuring:
//! - Multi-tier 3D luxury octagonal podium platform with gold/cyan metallic bevels & glowing LED edge strip
//! - 360° High-density 3D equalizer emitter pillars rising around the podium perimeter
//! - 8 Vertical holographic laser pillars forming a 3D translucent laser curtain wall
//! - Floating 3D holographic reticle wheel with 4 cardinal crosshairs & spectrum wave ring
//! - Luminous central hologram nucleus orb with dual 3D drone satellite motes
//! - Receding 3D perspective floor grid & floating 3D cyber stardust swarm
//! - Dynamic 3D camera pitching & yaw orbiting with full UI settings integration (Scale, Position X & Y, Sensitivity, Bass Boost, Bar Count).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
  theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;

  let p = theme_primary(theme);
  let s = theme_secondary(theme);
  let accent = theme_accent(theme);
  let glow = theme_glow(theme);

  // Settings integration
  let sensitivity = ctx.config.reactivity.sensitivity;
  let user_scale = ctx.config.scale.clamp(0.1, 5.0);
  let pos_offset_x = ctx.config.position_x * width * 0.5;
  let pos_offset_y = -ctx.config.position_y * height * 0.5;
  let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let frame_time = ctx.frame_time;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5 + pos_offset_x;
  let center_y = height * 0.60 - pos_offset_y;

  let base_r = ((width.min(height) * 0.28).clamp(90.0, 320.0) * user_scale).clamp(50.0, width * 0.44);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. DEEP LUXURY ATMOSPHERIC BACKDROP & RADIAL HOLOGRAPHIC AURA
  // -------------------------------------------------------------------------
  let bg_haze = Fill::radial_gradient(
    center_x,
    center_y,
    0.0,
    center_x,
    center_y,
    base_r * 2.2,
    &[
      (0.0, glow.with_alpha(0.26 + be * 0.18)),
      (0.35, p.with_alpha(0.14)),
      (0.70, Color::rgba(0.04, 0.02, 0.08, 0.06)),
      (1.0, Color::TRANSPARENT),
    ],
  );
  c.set_fill(bg_haze);
//   c.fill_rect(0.0, 0.0, width, height);

  // -------------------------------------------------------------------------
  // 2. CAMERA CONFIGURATION FOR NATIVE 3D SCENE (Scene3D)
  // -------------------------------------------------------------------------
  let scene = &mut ctx.scene3d;
  scene.cam_yaw = (rot * 0.06).sin() * (0.35 + be * 0.08);
  scene.cam_pitch = -0.38 - (frame_time * 0.02).sin() * 0.03 - be * 0.04;
  scene.cam_zoom = (1.15 - be * 0.05) / user_scale;
  scene.target_x = pos_offset_x;
  scene.target_y = pos_offset_y;

  let world_cy = height * 0.5 - center_y;
  let world_floor = world_cy - height * 0.16;
  let stage_h = (height * 0.30).clamp(70.0, 220.0);
  let top_y = world_floor + stage_h;

  let gold_col = Color::rgba(0.95, 0.75, 0.30, 0.95);
  let cyan_col = Color::rgba(0.0, 0.90, 1.0, 0.95);

  // -------------------------------------------------------------------------
  // 3. RECEDING 3D PERSPECTIVE FLOOR GRID LINES
  // -------------------------------------------------------------------------
  let half_w = width * 0.88;
  let z_max = -580.0f32;
  let grid_col = mix(p, s, 0.5).with_alpha(0.30);

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
  // 4. MULTI-TIER 3D LUXURY OCTAGONAL PODIUM PLATFORM (BASE & TIER 2)
  // -------------------------------------------------------------------------
  let podium_r1 = base_r * 1.30;
  let podium_r2 = base_r * 1.05;

  // Base Tier 1 Box Platform
  scene.add_box(0.0, world_floor + 4.0, 0.0, podium_r1 * 1.85, 8.0, podium_r1 * 1.85, Color::rgba(0.08, 0.09, 0.13, 0.98));

  // Raised Tier 2 Box Platform
  scene.add_box(0.0, world_floor + 12.0, 0.0, podium_r2 * 1.80, 8.0, podium_r2 * 1.80, Color::rgba(0.12, 0.14, 0.19, 0.98));

  // Gold Bevel Edge Highlight on Tier 1
  let dash_slots = 96usize;
  let mut floor_radii = vec![0.0f32; dash_slots];
  for (i, r_val) in floor_radii.iter_mut().enumerate() {
    *r_val = if (i % 12) < 7 { podium_r1 } else { podium_r1 * 0.94 };
  }

  scene.push();
  scene.translate(0.0, world_floor + 8.5, 0.0);
  scene.rotate_x(std::f32::consts::FRAC_PI_2);
  scene.add_band(0.0, 0.0, 0.0, podium_r1 * 0.94, podium_r1, &floor_radii, 3.0, gold_col);
  scene.pop();

  // Cyan Glowing Edge Strip on Tier 2
  scene.push();
  scene.translate(0.0, world_floor + 16.5, 0.0);
  scene.rotate_x(std::f32::consts::FRAC_PI_2);
  scene.rotate_z(-rot * 0.4);
  scene.add_band(0.0, 0.0, 0.0, podium_r2 * 0.95, podium_r2, &floor_radii, 2.5, cyan_col);
  scene.pop();

  // -------------------------------------------------------------------------
  // 5. 360° 3D EQUALIZER EMITTER PILLARS (PODIUM PERIMETER)
  // -------------------------------------------------------------------------
  let step_f = (freq.len() / bar_count).max(1);
  let max_pillar_h = height * 0.18 * sensitivity;

  for i in 0..bar_count {
    let angle = (i as f32 / bar_count as f32) * TAU + rot * 0.5;
    let k = (i * step_f).min(freq.len().saturating_sub(1));
    let fv = freq[k] as f32 / 255.0;
    let pillar_h = (fv * max_pillar_h + 8.0 + be * 14.0).clamp(8.0, (max_pillar_h * 1.5).max(8.0));

    let (s_a, c_a) = angle.sin_cos();
    let px = c_a * (podium_r1 * 0.96);
    let pz = s_a * (podium_r1 * 0.96);

    let pillar_col = mix(p, s, i as f32 / bar_count as f32);
    let top_col = if fv > 0.60 || bs > 0.40 { Color::WHITE } else { mix(pillar_col, accent, 0.5) };

    // Pillar body
    scene.add_box(px, world_floor + 16.0 + pillar_h * 0.5, pz, 3.8, pillar_h, 3.8, pillar_col);
    // Glowing top cap
    scene.add_box(px, world_floor + 16.0 + pillar_h + 1.0, pz, 4.5, 2.5, 4.5, top_col);
  }

  // -------------------------------------------------------------------------
  // 6. 8 VERTICAL HOLOGRAPHIC LASER PILLARS & TRANSLUCENT CURTAIN WALL
  // -------------------------------------------------------------------------
  let laser_pillar_count = 8usize;
  let laser_r = base_r * 0.70;

  for l_i in 0..laser_pillar_count {
    let l_angle = (l_i as f32 / laser_pillar_count as f32) * TAU + rot * 0.3;
    let lx = l_angle.cos() * laser_r;
    let lz = l_angle.sin() * laser_r;

    // Vertical Laser Pillar
    scene.add_box(lx, (world_floor + top_y) * 0.5, lz, 3.0, stage_h, 3.0, cyan_col.with_alpha(0.85));

    // Connect adjacent laser pillars with translucent laser curtain quad
    let l_next_angle = ((l_i + 1) as f32 / laser_pillar_count as f32) * TAU + rot * 0.3;
    let n_lx = l_next_angle.cos() * laser_r;
    let n_lz = l_next_angle.sin() * laser_r;

    scene.quad(
      [lx, world_floor + 16.0, lz],
      [n_lx, world_floor + 16.0, n_lz],
      [n_lx, top_y, n_lz],
      [lx, top_y, lz],
      p.with_alpha(0.08 + be * 0.06),
    );
  }

  // Inner White-Hot Laser Core Column
  scene.add_box(0.0, (world_floor + top_y) * 0.5, 0.0, laser_r * 0.40, stage_h, laser_r * 0.40, Color::rgba(1.0, 1.0, 1.0, 0.18));

  // -------------------------------------------------------------------------
  // 7. FLOATING 3D HOLOGRAPHIC RETICLE WHEEL & SPECTRUM WAVE (STAGE TOP)
  // -------------------------------------------------------------------------
  let r_wave = base_r * 0.65;
  let wave_slots = 64usize;
  let mut wave_radii = Vec::with_capacity(wave_slots);
  let wave_step = (freq.len() / wave_slots).max(1);

  for k in 0..wave_slots {
    let bin = (k * wave_step).min(freq.len().saturating_sub(1));
    let fv = freq[bin] as f32 / 255.0;
    wave_radii.push(r_wave + fv * 18.0 * sensitivity);
  }

  scene.push();
  scene.translate(0.0, top_y, 0.0);
  scene.rotate_x(std::f32::consts::FRAC_PI_2);
  scene.rotate_z(-rot * 0.8);
  scene.add_band(0.0, 0.0, 0.0, r_wave * 0.92, r_wave, &wave_radii, 4.0, p.with_alpha(0.95));
  scene.pop();

  // 4 Cardinal Crosshair Brackets on Top Stage Reticle
  for c_i in 0..4 {
    let ca = rot * 0.4 + (c_i as f32 / 4.0) * TAU;
    let cx = ca.cos() * (r_wave + 6.0);
    let cz = ca.sin() * (r_wave + 6.0);
    scene.add_box(cx, top_y, cz, 6.0, 6.0, 4.0, Color::WHITE);
  }

  // -------------------------------------------------------------------------
  // 8. HYPER-REALISTIC 3D VOLUMETRIC HOLOGRAPHIC MUSIC NOTE (♫) & LASERS
  // -------------------------------------------------------------------------
  let float_y = (frame_time * 2.2).sin() * 6.0 + be * 8.0;
  let orb_y = top_y + height * 0.08 + float_y;
  let note_sz = (28.0 + be * 14.0 * sensitivity).clamp(22.0, 60.0);
  let cyan_laser = Color::hex("#00f0ff");
  let gold_metallic = mix(accent, gold_col, 0.7);

  // Translucent holographic light pillars shooting UP from stage reticle to under the Music Note
  for laser_i in 0..4 {
    let la = (laser_i as f32 / 4.0) * TAU + rot * 0.4;
    let lx = la.cos() * (r_wave * 0.35);
    let lz = la.sin() * (r_wave * 0.35);
    let laser_col = mix(accent, cyan_laser, laser_i as f32 / 3.0)
      .with_alpha(0.18 + (laser_i as f32 * 0.7 + frame_time * 3.0).sin().abs() * 0.22);

    let pillar_bottom = top_y;
    let pillar_top = orb_y - note_sz * 0.6;
    let pillar_cy = (pillar_bottom + pillar_top) * 0.5;
    let pillar_h = (pillar_top - pillar_bottom).max(1.0);

    scene.add_box(lx, pillar_cy, lz, 1.2, pillar_h, 1.2, laser_col);
  }

  scene.push();
  scene.translate(0.0, orb_y, 0.0);
  scene.rotate_y(rot * 0.8); // Smooth 3D Y-axis spinning in holographic space!

  // 3D Spinning Holographic Ring surrounding the 3D Music Note Emblem
  let ring_r = note_sz * 1.15;
  scene.push();
  scene.rotate_x(0.35);
  scene.rotate_z(rot * 1.2);
  scene.add_ring(0.0, 0.0, 0.0, ring_r, ring_r * 0.94, 2.5, 36, cyan_laser.with_alpha(0.70));
  for i in 0..4 {
    let a = (i as f32 / 4.0) * TAU;
    let (nx, ny) = (a.cos() * ring_r, a.sin() * ring_r);
    scene.add_box(nx, ny, 0.0, 3.5, 3.5, 3.5, Color::WHITE);
  }
  scene.pop();

  // Authentic Beamed Double Eighth Note (♫) Geometry Parameters
  let rx = note_sz * 0.34;
  let ry = note_sz * 0.22;
  let tilt_rad = 28.0f32.to_radians();
  let depth = note_sz * 0.22;
  let (_sin_t, cos_t) = tilt_rad.sin_cos();

  // Left & Right notehead center positions (Right notehead is elevated slightly)
  let left_head_x = -note_sz * 0.38;
  let left_head_y = -note_sz * 0.24;

  let right_head_x = note_sz * 0.38;
  let right_head_y = -note_sz * 0.08;

  let note_body_col = Color::rgba(0.96, 0.96, 1.0, 0.96);
  let note_core_col = cyan_laser;

  // 1. Render Left & Right Tilted 3D Extruded Noteheads
  draw_3d_tilted_notehead(
    scene,
    left_head_x,
    left_head_y,
    0.0,
    rx,
    ry,
    tilt_rad,
    depth,
    24,
    gold_metallic,
    note_core_col,
  );

  draw_3d_tilted_notehead(
    scene,
    right_head_x,
    right_head_y,
    0.0,
    rx,
    ry,
    tilt_rad,
    depth,
    24,
    gold_metallic,
    note_core_col,
  );

  // 2. Compute precise stem attachment points on right-outer edge of tilted noteheads
  let stem_w = note_sz * 0.09;
  let stem_d = note_sz * 0.16;

  let left_stem_x = left_head_x + rx * 0.85 * cos_t;
  let right_stem_x = right_head_x + rx * 0.85 * cos_t;

  let stem_h = note_sz * 1.05;
  let left_stem_top_y = left_head_y + stem_h;
  let right_stem_top_y = right_head_y + stem_h;

  // Render 3D Volumetric Stems
  let left_stem_cy = (left_head_y + left_stem_top_y) * 0.5;
  let left_stem_sy = left_stem_top_y - left_head_y;
  scene.add_box(left_stem_x, left_stem_cy, 0.0, stem_w, left_stem_sy, stem_d, note_body_col);
  scene.add_box(left_stem_x, left_stem_cy, stem_d * 0.5 + 0.1, stem_w * 0.45, left_stem_sy * 0.98, 0.2, cyan_laser);

  let right_stem_cy = (right_head_y + right_stem_top_y) * 0.5;
  let right_stem_sy = right_stem_top_y - right_head_y;
  scene.add_box(right_stem_x, right_stem_cy, 0.0, stem_w, right_stem_sy, stem_d, note_body_col);
  scene.add_box(right_stem_x, right_stem_cy, stem_d * 0.5 + 0.1, stem_w * 0.45, right_stem_sy * 0.98, 0.2, cyan_laser);

  // 3. Slanted Primary Top 3D Connecting Beam
  let beam_thickness = note_sz * 0.16;
  draw_3d_slanted_beam(
    scene,
    left_stem_x - stem_w * 0.5,
    left_stem_top_y - beam_thickness * 0.5,
    right_stem_x + stem_w * 0.5,
    right_stem_top_y - beam_thickness * 0.5,
    0.0,
    beam_thickness,
    stem_d * 1.1,
    note_body_col,
    cyan_laser,
  );

  // 4. Slanted Secondary 3D Connecting Beam (Double Eighth Note ♫)
  let beam2_offset = note_sz * 0.22;
  draw_3d_slanted_beam(
    scene,
    left_stem_x - stem_w * 0.5,
    left_stem_top_y - beam_thickness * 0.5 - beam2_offset,
    right_stem_x + stem_w * 0.5,
    right_stem_top_y - beam_thickness * 0.5 - beam2_offset,
    0.0,
    beam_thickness * 0.85,
    stem_d * 1.0,
    gold_metallic,
    cyan_laser,
  );

  scene.pop();

  // Dual 3D Crystal Drone Satellites Orbiting the Holographic Music Note
  for sat_i in 0..2 {
    let sa = rot * 2.2 + sat_i as f32 * std::f32::consts::PI;
    let sat_dist = note_sz * 1.4 + 18.0 + be * 6.0;
    let sx = sa.cos() * sat_dist;
    let sz = sa.sin() * sat_dist;
    let sy = orb_y + (sa * 0.5).sin() * 8.0;

    scene.add_disc(sx, sy, sz, 5.0 + bs * 2.0, 12, cyan_laser);
  }

  // -------------------------------------------------------------------------
  // 9. FLOATING 3D CYBER STARDUST PARTICLES (45+ MOTES)
  // -------------------------------------------------------------------------
  let mote_count = (22.0 + be * 24.0 * sensitivity).clamp(16.0, 52.0) as usize;
  for m_i in 0..mote_count {
    let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
    let mx = (m_i as f32 * 37.0).sin() * (base_r * 1.45);
    let my = world_floor + m_t * (height * 0.55);
    let mz = (m_i as f32 * 23.0).cos() * (base_r * 1.45);

    let m_sz = (2.5 * (1.0 - m_t) + 1.0).clamp(1.0, 4.5);
    let m_col = mix(glow, Color::WHITE, m_t).with_alpha((1.0 - m_t).clamp(0.1, 0.95));
    scene.add_disc(mx, my, mz, m_sz, 6, m_col);
  }

  c.set_global_alpha(1.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.restore();
}

// ---------------------------------------------------------------------------
// 3D GEOMETRY HELPER FUNCTIONS FOR MUSIC NOTE EMBLEM
// ---------------------------------------------------------------------------

/// Render a 3D tilted extruded elliptical notehead with volumetric depth & glowing core.
fn draw_3d_tilted_notehead(
  scene: &mut crate::gpu2d::scene3d::Scene3D,
  cx: f32,
  cy: f32,
  cz: f32,
  rx: f32,
  ry: f32,
  tilt_rad: f32,
  depth: f32,
  segments: u32,
  color_body: Color,
  color_core: Color,
) {
  let half_d = depth * 0.5;
  let seg = segments.max(12);
  let (sin_t, cos_t) = tilt_rad.sin_cos();

  let mut pts = Vec::with_capacity(seg as usize);
  for i in 0..seg {
    let a = (i as f32 / seg as f32) * TAU;
    let ex = a.cos() * rx;
    let ey = a.sin() * ry;
    let x = ex * cos_t - ey * sin_t;
    let y = ex * sin_t + ey * cos_t;
    pts.push([x, y]);
  }

  let center_front = [cx, cy, cz + half_d];
  let center_back = [cx, cy, cz - half_d];

  // 1. Front face polygon (+Z)
  for i in 0..seg {
    let j = (i + 1) % seg;
    let p0 = [cx + pts[i as usize][0], cy + pts[i as usize][1], cz + half_d];
    let p1 = [cx + pts[j as usize][0], cy + pts[j as usize][1], cz + half_d];
    scene.quad(center_front, p0, p1, center_front, color_body);
  }

  // 2. Back face polygon (-Z)
  for i in 0..seg {
    let j = (i + 1) % seg;
    let p0 = [cx + pts[i as usize][0], cy + pts[i as usize][1], cz - half_d];
    let p1 = [cx + pts[j as usize][0], cy + pts[j as usize][1], cz - half_d];
    scene.quad(center_back, p1, p0, center_back, color_body);
  }

  // 3. 3D Volumetric Extrusion Sidewalls
  for i in 0..seg {
    let j = (i + 1) % seg;
    let p0_front = [cx + pts[i as usize][0], cy + pts[i as usize][1], cz + half_d];
    let p1_front = [cx + pts[j as usize][0], cy + pts[j as usize][1], cz + half_d];
    let p0_back = [cx + pts[i as usize][0], cy + pts[i as usize][1], cz - half_d];
    let p1_back = [cx + pts[j as usize][0], cy + pts[j as usize][1], cz - half_d];

    let ny = (pts[i as usize][1] + pts[j as usize][1]) * 0.5;
    let wall_col = if ny > 0.0 {
      mix(color_body, Color::WHITE, 0.28)
    } else {
      color_body
    };

    scene.quad(p0_back, p1_back, p1_front, p0_front, wall_col);
  }

  // 4. Glowing inner core pulse (front surface)
  let center_core = [cx, cy, cz + half_d + 0.2];
  for i in 0..seg {
    let j = (i + 1) % seg;
    let p0 = [cx + pts[i as usize][0] * 0.55, cy + pts[i as usize][1] * 0.55, cz + half_d + 0.2];
    let p1 = [cx + pts[j as usize][0] * 0.55, cy + pts[j as usize][1] * 0.55, cz + half_d + 0.2];
    scene.quad(center_core, p0, p1, center_core, color_core);
  }
}

/// Render a 3D slanted beam connecting stem endpoints with volumetric depth & neon edge stripe.
fn draw_3d_slanted_beam(
  scene: &mut crate::gpu2d::scene3d::Scene3D,
  x1: f32,
  y1: f32,
  x2: f32,
  y2: f32,
  cz: f32,
  thickness: f32,
  depth: f32,
  color_body: Color,
  color_neon: Color,
) {
  let half_d = depth * 0.5;
  let dx = x2 - x1;
  let dy = y2 - y1;
  let len = (dx * dx + dy * dy).sqrt();
  if len < 0.001 {
    return;
  }
  let angle = dy.atan2(dx);
  let (sin_a, cos_a) = (angle.sin(), angle.cos());

  let nx = -sin_a * thickness * 0.5;
  let ny = cos_a * thickness * 0.5;

  let p0 = [x1 + nx, y1 + ny];
  let p1 = [x2 + nx, y2 + ny];
  let p2 = [x2 - nx, y2 - ny];
  let p3 = [x1 - nx, y1 - ny];

  // Front face (+Z)
  scene.quad(
    [p0[0], p0[1], cz + half_d],
    [p1[0], p1[1], cz + half_d],
    [p2[0], p2[1], cz + half_d],
    [p3[0], p3[1], cz + half_d],
    color_body,
  );

  // Back face (-Z)
  scene.quad(
    [p0[0], p0[1], cz - half_d],
    [p3[0], p3[1], cz - half_d],
    [p2[0], p2[1], cz - half_d],
    [p1[0], p1[1], cz - half_d],
    color_body,
  );

  // Top edge (+N)
  scene.quad(
    [p0[0], p0[1], cz - half_d],
    [p1[0], p1[1], cz - half_d],
    [p1[0], p1[1], cz + half_d],
    [p0[0], p0[1], cz + half_d],
    mix(color_body, Color::WHITE, 0.35),
  );

  // Bottom edge (-N)
  scene.quad(
    [p3[0], p3[1], cz - half_d],
    [p3[0], p3[1], cz + half_d],
    [p2[0], p2[1], cz + half_d],
    [p2[0], p2[1], cz - half_d],
    color_body,
  );

  // Left & Right cap
  scene.quad(
    [p0[0], p0[1], cz - half_d],
    [p0[0], p0[1], cz + half_d],
    [p3[0], p3[1], cz + half_d],
    [p3[0], p3[1], cz - half_d],
    color_body,
  );
  scene.quad(
    [p1[0], p1[1], cz - half_d],
    [p2[0], p2[1], cz - half_d],
    [p2[0], p2[1], cz + half_d],
    [p1[0], p1[1], cz + half_d],
    color_body,
  );

  // Neon accent stripe along front face
  let snx = -sin_a * thickness * 0.20;
  let sny = cos_a * thickness * 0.20;
  scene.quad(
    [x1 + snx, y1 + sny, cz + half_d + 0.2],
    [x2 + snx, y2 + sny, cz + half_d + 0.2],
    [x2 - snx, y2 - sny, cz + half_d + 0.2],
    [x1 - snx, y1 - sny, cz + half_d + 0.2],
    color_neon,
  );
}

