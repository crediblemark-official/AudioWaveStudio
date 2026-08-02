//! Background overlay effects (Phase 6): grid, aurora, noise, bokeh,
//! starfield, nebula, psychedelic. Mirrors `src/services/renderers/background/*`.
//! Canvas `screen` compositing is approximated with alpha blending.

use super::{hsl_to_color, RenderContext};
use crate::config::{MusicNoteStyle, ParticleStyle};
use crate::gpu2d::{Color, Fill, GpuCanvas, NOISE_LAYER};

const STAR_COUNT: usize = 300;

pub struct Star {
  pub x: f32,
  pub y: f32,
  pub size: f32,
  pub phase: f32,
  pub speed: f32,
}

/// Floating accent particle (mirrors TS `Particle`).
pub struct Particle {
  pub x: f32,
  pub y: f32,
  pub radius: f32,
  pub vx: f32,
  pub vy: f32,
  pub alpha: f32,
  pub phase: f32,
}

/// Music note particle (mirrors TS `MusicNote`).
pub struct MusicNote {
  pub x: f32,
  pub y: f32,
  pub vx: f32,
  pub vy: f32,
  pub size: f32,
  pub alpha: f32,
  pub rotation: f32,
  pub symbol: u32,
  pub life: f32,
  pub max_life: f32,
  pub base_x: f32,
  pub phase: f32,
}

const NOTE_SYMBOLS: [&str; 4] = ["\u{2669}", "\u{266A}", "\u{266B}", "\u{266C}"];

fn make_particle(rng: &mut super::Rng, edge_spawn: bool) -> Particle {
  let angle = rng.next() * std::f32::consts::TAU;
  let speed = 0.0004 + rng.next() * 0.0004;
  let (x, y) = if edge_spawn {
    let side = (rng.next() * 4.0) as u32;
    match side {
      0 => (0.05, 0.05 + rng.next() * 0.9),
      1 => (0.95, 0.05 + rng.next() * 0.9),
      2 => (0.05 + rng.next() * 0.9, 0.05),
      _ => (0.05 + rng.next() * 0.9, 0.95),
    }
  } else {
    (0.05 + rng.next() * 0.9, 0.05 + rng.next() * 0.9)
  };
  Particle {
    x,
    y,
    radius: rng.next() * 3.0 + 1.0,
    vx: angle.cos() * speed,
    vy: angle.sin() * speed,
    alpha: rng.next() * 0.5 + 0.3,
    phase: rng.next() * std::f32::consts::TAU,
  }
}

pub fn init_particles(rng: &mut super::Rng) -> Vec<Particle> {
  (0..60).map(|_| make_particle(rng, false)).collect()
}

/// Mirrors `src/services/renderers/background/particles.ts`.
pub fn render_particles(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let bg = &ctx.config.background;
  let color = Color::hex(if bg.particle_color.is_empty() {
    ctx.config.theme.accent_color.as_str()
  } else {
    bg.particle_color.as_str()
  });
  let style = bg.particle_style.clone().unwrap_or(ParticleStyle::Float);
  let speed = bg.particle_speed.unwrap_or(1.0).max(0.1);
  let size = bg.particle_size.unwrap_or(4.0).max(1.0);
  let target_count = bg.particle_count.unwrap_or(60).max(5) as usize;

  let particles = &mut ctx.state.particles;
  while particles.len() < target_count {
    particles.push(make_particle(&mut ctx.state.rng, false));
  }
  particles.truncate(target_count);

  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let kick_bins = 12.min(freq.len());
  let mut kick_sum = 0usize;
  for k in 0..kick_bins {
    kick_sum += freq[k] as usize;
  }
  let kick_energy = if kick_bins > 0 { kick_sum as f32 / (kick_bins as f32 * 255.0) } else { 0.0 };
  let is_percussive = bs > 0.12 || (kick_energy > 0.4 && bs > 0.08);
  let impact = if is_percussive { (bs * 1.2).max(kick_energy * 0.8) } else { 0.0 };
  let kaget = impact * 0.025 * speed;
  let scatter = impact * 0.03 * speed;

  for (i, p) in particles.iter_mut().enumerate() {
    p.phase += 0.012 * speed;
    let roll_angle = p.phase + i as f32 * 0.3;
    let base_roll_x = roll_angle.cos() * 0.0003 * speed;
    let base_roll_y = roll_angle.sin() * 0.0003 * speed;
    let drum_vib_x = if is_percussive { (ctx.state.rng.next() - 0.5) * impact * 0.018 * speed } else { 0.0 };
    let drum_vib_y = if is_percussive { (ctx.state.rng.next() - 0.5) * impact * 0.018 * speed } else { 0.0 };

    match &style {
      ParticleStyle::Confined => {
        p.vx += base_roll_x + drum_vib_x;
        p.vy += base_roll_y + drum_vib_y;
        if is_percussive {
          let (dx, dy) = (p.x - 0.5, p.y - 0.5);
          let dist = dx.hypot(dy).max(0.0001);
          let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
          p.vx += (dx / dist) * scatter + (ctx.state.rng.next() - 0.5) * kaget * dir;
          p.vy += (dy / dist) * scatter + (ctx.state.rng.next() - 0.5) * kaget * dir;
        }
        p.vx *= 0.94;
        p.vy *= 0.94;
        if p.x < 0.03 { p.x = 0.03; p.vx = p.vx.abs() * 0.9; }
        if p.x > 0.97 { p.x = 0.97; p.vx = -p.vx.abs() * 0.9; }
        if p.y < 0.03 { p.y = 0.03; p.vy = p.vy.abs() * 0.9; }
        if p.y > 0.97 { p.y = 0.97; p.vy = -p.vy.abs() * 0.9; }
      }
      ParticleStyle::Bounce => {
        p.vx += base_roll_x * 0.6 + drum_vib_x;
        p.vy += base_roll_y * 0.6 + drum_vib_y;
        if is_percussive {
          let sa = ctx.state.rng.next() * std::f32::consts::TAU;
          p.vx += sa.cos() * scatter;
          p.vy += sa.sin() * scatter;
        }
        p.vx *= 0.95;
        p.vy *= 0.95;
        if p.x < 0.03 { p.x = 0.03; p.vx = p.vx.abs() * 0.9; }
        if p.x > 0.97 { p.x = 0.97; p.vx = -p.vx.abs() * 0.9; }
        if p.y < 0.03 { p.y = 0.03; p.vy = p.vy.abs() * 0.9; }
        if p.y > 0.97 { p.y = 0.97; p.vy = -p.vy.abs() * 0.9; }
      }
      ParticleStyle::Wave => {
        let wave_y = -0.0005 * speed + p.phase.sin() * 0.0004 * speed;
        let wave_x = (p.phase * 0.7).cos() * 0.0006 * speed;
        p.vx += (wave_x - p.vx) * 0.1 + drum_vib_x;
        p.vy += (wave_y - p.vy) * 0.1 + drum_vib_y;
        if is_percussive {
          p.vy -= kaget * 1.5;
          p.vx += (ctx.state.rng.next() - 0.5) * scatter * 1.5;
        }
      }
      ParticleStyle::Static => {
        let hover_x = p.phase.cos() * 0.00025 * speed;
        let hover_y = p.phase.sin() * 0.00025 * speed;
        p.vx = hover_x + drum_vib_x;
        p.vy = hover_y + drum_vib_y;
        if is_percussive {
          p.vx += (ctx.state.rng.next() - 0.5) * kaget * 1.2;
          p.vy += (ctx.state.rng.next() - 0.5) * kaget * 1.2;
        }
      }
      ParticleStyle::Float => {
        let float_up = -0.0006 * speed;
        let float_sway = p.phase.sin() * 0.0004 * speed;
        p.vy += (float_up - p.vy) * 0.08 + drum_vib_y;
        p.vx += (float_sway - p.vx) * 0.08 + drum_vib_x;
        if is_percussive {
          p.vy -= kaget * 1.8;
          p.vx += (ctx.state.rng.next() - 0.5) * scatter * 1.8;
        }
      }
    }

    let min_roll = 0.0003 * speed;
    let max_roll = 0.018 * speed;
    let cur = p.vx.hypot(p.vy);
    if cur < min_roll {
      let a = ctx.state.rng.next() * std::f32::consts::TAU;
      p.vx += a.cos() * min_roll;
      p.vy += a.sin() * min_roll;
    } else if cur > max_roll {
      p.vx = (p.vx / cur) * max_roll;
      p.vy = (p.vy / cur) * max_roll;
    }

    p.x += p.vx;
    p.y += p.vy;

    if matches!(style, ParticleStyle::Confined | ParticleStyle::Bounce) {
      if p.x < 0.03 { p.x = 0.03; p.vx = p.vx.abs() * 0.9; }
      if p.x > 0.97 { p.x = 0.97; p.vx = -p.vx.abs() * 0.9; }
      if p.y < 0.03 { p.y = 0.03; p.vy = p.vy.abs() * 0.9; }
      if p.y > 0.97 { p.y = 0.97; p.vy = -p.vy.abs() * 0.9; }
    } else {
      if p.y < -0.05 { p.y = 1.05; p.x = ctx.state.rng.next(); p.vy = -ctx.state.rng.next() * 0.0008 - 0.0003; }
      if p.y > 1.05 { p.y = -0.05; p.x = ctx.state.rng.next(); }
      if p.x < -0.05 { p.x = 1.05; }
      if p.x > 1.05 { p.x = -0.05; }
    }
  }

  for p in particles.iter() {
    let bx = p.x * ctx.width;
    let by = p.y * ctx.height;
    let base_radius = (p.radius * 0.5 + 1.2) * (size / 4.0);
    let beat_pulse = if is_percussive { impact * 10.0 * (size / 4.0) } else { 0.0 };
    let r = (base_radius + beat_pulse).max(0.5);
    let alpha = (p.alpha + if is_percussive { impact * 0.6 } else { 0.0 }).min(1.0);
    c.set_fill(Fill::Solid(color.with_alpha(alpha)));
    c.fill_circle(bx, by, r);
  }
}

/// Mirrors `src/services/renderers/background/musicNotes.ts`.
pub fn render_music_notes(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let bg = &ctx.config.background;
  let density = bg.music_note_density.unwrap_or(1.0);
  let note_size = bg.music_note_size.unwrap_or(60.0);
  let max_notes = bg.music_note_count.unwrap_or(80).min(80) as usize;
  let sensitivity = bg.music_note_sensitivity.unwrap_or(1.0);
  let color = Color::hex(
    bg.music_note_color
      .as_deref()
      .unwrap_or(ctx.config.theme.accent_color.as_str()),
  );
  let style = bg.music_note_style.clone().unwrap_or(MusicNoteStyle::Float);

  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let high_bins = 64.min(freq.len());
  let mut high_sum = 0usize;
  for i in 24..high_bins {
    high_sum += freq[i] as usize;
  }
  let high_energy = high_sum as f32 / ((high_bins.saturating_sub(24)).max(1) as f32 * 255.0);
  let wobble_amp = high_energy * 3.0;
  let is_confined = matches!(style, MusicNoteStyle::Confined);

  let notes = &mut ctx.state.music_notes;
  if bs > 0.05 && ctx.state.rng.next() < (density * 0.5 + bs * 0.5).min(1.0) {
    let count = (1.0 + bs * 3.0).floor() as usize;
    let count = count.min(max_notes.saturating_sub(notes.len())).min(if is_confined { 1 } else { 3 });
    let phase_step = std::f32::consts::TAU / count.max(1) as f32;
    let base_vy = -(3.0 + ctx.state.rng.next() * 3.0 + bs * 12.0);
    for n in 0..count {
      notes.push(MusicNote {
        x: if is_confined { ctx.state.rng.next() * ctx.width } else { ctx.state.rng.next() * ctx.width },
        y: if is_confined { ctx.state.rng.next() * ctx.height } else { ctx.height },
        vx: if is_confined { (ctx.state.rng.next() - 0.5) * 3.0 } else { (ctx.state.rng.next() - 0.5) * 2.0 },
        vy: if is_confined { (ctx.state.rng.next() - 0.5) * 3.0 } else { base_vy * (1.0 + ctx.state.rng.next() * 0.3) },
        size: note_size * (0.5 + ctx.state.rng.next() * 0.7 + bs * 0.5),
        alpha: 0.5 + ctx.state.rng.next() * 0.5,
        rotation: (ctx.state.rng.next() - 0.5) * 0.3,
        symbol: (ctx.state.rng.next() * 4.0) as u32 % 4,
        life: 0.0,
        max_life: if is_confined { 100.0 + ctx.state.rng.next() * 50.0 } else { 60.0 + ctx.state.rng.next() * 30.0 },
        base_x: ctx.state.rng.next() * ctx.width,
        phase: n as f32 * phase_step,
      });
    }
  }

  let speed_boost = 1.5 + bs * 4.0 * sensitivity;

  let mut alive: Vec<MusicNote> = Vec::with_capacity(notes.len());
  for mut n in notes.drain(..) {
    n.life += 1.0;
    if n.life >= n.max_life {
      continue;
    }
    let t = n.life / n.max_life;
    let fade_out = (1.0 - n.life / n.max_life).min(1.0).min((n.max_life - n.life) / 10.0);
    let alpha = n.alpha * fade_out;

    match &style {
      MusicNoteStyle::Bounce => {
        n.y += n.vy * speed_boost * 0.6;
        n.x += (n.life * 0.06 + n.phase).sin() * wobble_amp * 0.5;
        n.vy += 0.3;
      }
      MusicNoteStyle::Spiral => {
        let r = t * ctx.width.min(ctx.height) * 0.4;
        let a = n.life * 0.04 + n.phase;
        n.x = n.base_x + a.cos() * r * 0.3;
        n.y = ctx.height * 0.5 - t * ctx.height * 0.3 + a.sin() * r * 0.1;
      }
      MusicNoteStyle::Wave => {
        n.y += n.vy * speed_boost * 0.5;
        n.x = n.base_x + (n.life * 0.05 + n.phase).sin() * ctx.width * 0.2;
      }
      MusicNoteStyle::Burst => {
        n.x += (n.vx + (n.life * 0.1).sin() * 0.5) * speed_boost;
        n.y += n.vy * speed_boost + 0.5;
      }
      MusicNoteStyle::Confined => {
        n.vx += (ctx.state.rng.next() - 0.5) * 0.2 + (ctx.state.rng.next() - 0.5) * bs * 2.0;
        n.vy += (ctx.state.rng.next() - 0.5) * 0.2 + (ctx.state.rng.next() - 0.5) * bs * 2.0;
        n.vx = n.vx.clamp(-4.0, 4.0);
        n.vy = n.vy.clamp(-4.0, 4.0);
        n.x += n.vx * speed_boost;
        n.y += n.vy * speed_boost;
        if n.x < 0.0 { n.x = 0.0; n.vx = n.vx.abs(); }
        else if n.x > ctx.width { n.x = ctx.width; n.vx = -n.vx.abs(); }
        if n.y < 0.0 { n.y = 0.0; n.vy = n.vy.abs(); }
        else if n.y > ctx.height { n.y = ctx.height; n.vy = -n.vy.abs(); }
      }
      MusicNoteStyle::Float => {
        n.x += n.vx * speed_boost;
        n.y += n.vy * speed_boost;
      }
    }

    n.rotation += 0.02;
    let pulse = 1.0 + (n.life * 0.1).sin() * 0.1;
    let sz = n.size * pulse;

    c.save();
    c.translate(n.x, n.y);
    c.rotate(n.rotation);
    c.set_fill(Fill::Solid(color));
    c.draw_text(
      NOTE_SYMBOLS[n.symbol as usize],
      0.0,
      sz * 0.35,
      sz,
      "serif",
      400.0,
      crate::gpu2d::text::TextAlign::Center,
      Fill::Solid(color),
      alpha.clamp(0.0, 1.0),
      &crate::gpu2d::text::TextOpts::default(),
    );
    c.restore();

    alive.push(n);
  }
  *notes = alive;
}

fn hash(n: f32) -> f32 {
  let x = (n * 12.9898 + 78.233).sin() * 43758.5453;
  x - x.floor()
}

pub fn build_stars() -> Vec<Star> {
  (0..STAR_COUNT)
    .map(|i| {
      let i = i as f32;
      Star {
        x: hash(i + 1.0),
        y: hash(i + 1000.0),
        size: 1.2 + hash(i + 2000.0) * 2.8,
        phase: hash(i + 3000.0) * std::f32::consts::TAU,
        speed: 0.01 + hash(i + 4000.0) * 0.03,
      }
    })
    .collect()
}

pub fn render_grid(c: &mut GpuCanvas, ctx: &RenderContext) {
  let bg = &ctx.config.background;
  let color = Color::hex(bg.grid_color.as_deref().unwrap_or("#ffffff")).with_alpha(0.25);
  let grid_size = bg.grid_size.unwrap_or(40.0).max(2.0);
  let line_width = bg.grid_line_width.unwrap_or(1.0).max(0.5);
  c.set_stroke(Fill::Solid(color));
  c.set_line_width(line_width);
  let mut x = 0.0;
  while x <= ctx.width {
    c.stroke_line(x, 0.0, x, ctx.height);
    x += grid_size;
  }
  let mut y = 0.0;
  while y <= ctx.height {
    c.stroke_line(0.0, y, ctx.width, y);
    y += grid_size;
  }
}

pub fn render_aurora(c: &mut GpuCanvas, ctx: &RenderContext) {
  let bg = &ctx.config.background;
  let speed_mult = bg.aurora_speed.unwrap_or(1.0);
  let base_amp = bg.aurora_amplitude.unwrap_or(50.0);
  let base_opacity = bg.aurora_opacity.unwrap_or(0.25);
  let t = ctx.frame_time * speed_mult;
  let speed = (0.3 + ctx.bass_energy * 0.6) * speed_mult;
  let amp = base_amp + ctx.beat_strength * 60.0;
  for i in 0..4 {
    let hue = (i as f32 * 60.0 + t * 25.0) % 360.0;
    let alpha = (base_opacity * 0.6 + ctx.bass_energy * 0.1).min(1.0);
    c.set_fill(Fill::Solid(hsl_to_color(hue, 0.85, 0.60, alpha)));
    let mut pts: Vec<(f32, f32)> = Vec::new();
    let mut x = 0.0;
    while x <= ctx.width {
      let y = ctx.height * 0.45
        + (x * 0.006 + t * speed + i as f32 * 1.5).sin() * amp
        + (x * 0.012 + t * speed * 0.7 + i as f32 * 2.0).sin() * (amp * 0.5);
      pts.push((x, y));
      x += 6.0;
    }
    pts.push((ctx.width, ctx.height));
    pts.push((0.0, ctx.height));
    c.fill_polygon(&pts);
  }
}

pub fn render_noise(c: &mut GpuCanvas, ctx: &RenderContext) {
  let bg = &ctx.config.background;
  let base_opacity = bg.grain_opacity.unwrap_or(0.08);
  let alpha = (base_opacity + ctx.bass_energy * 0.08 + ctx.beat_strength * 0.06).min(1.0);
  let seed = (ctx.frame_time * 60.0).floor() as u32;
  let mut rgba = Vec::with_capacity(128 * 128 * 4);
  let mut rng = super::Rng::new(seed.wrapping_add(0x5EED));
  for _ in 0..128 * 128 {
    let v = (rng.next() * 255.0) as u8;
    rgba.extend_from_slice(&[v, v, v, 255]);
  }
  c.push_atlas_layer(NOISE_LAYER, rgba, 128, 128);
  c.push_textured_quad(
    NOISE_LAYER,
    0.0,
    0.0,
    ctx.width,
    ctx.height,
    [0.0, 0.0, 1.0, 1.0],
    Color::rgba(1.0, 1.0, 1.0, alpha),
  );
}

pub fn render_bokeh(c: &mut GpuCanvas, ctx: &RenderContext) {
  let bg = &ctx.config.background;
  let count = bg.bokeh_count.unwrap_or(18) as usize;
  let base_size = bg.bokeh_size.unwrap_or(30.0);
  let base_opacity = bg.bokeh_opacity.unwrap_or(0.3);
  let t = ctx.frame_time / 5.0;
  for i in 0..count {
    let seed = i as f32 * 137.5;
    let x = ((seed + t * (0.2 + i as f32 * 0.03)).sin() * 0.5 + 0.5) * ctx.width;
    let y = ((seed * 0.7 + t * (0.15 + i as f32 * 0.02)).cos() * 0.5 + 0.5) * ctx.height;
    let radius = ((base_size + (seed * 0.3 + t).sin() * (base_size * 0.4) + ctx.beat_strength * 40.0).abs()).max(1.0);
    let hue = (seed + t * 30.0) % 360.0;
    let a1 = (base_opacity + ctx.bass_energy * 0.15).clamp(0.0, 1.0);
    c.set_fill(Fill::Solid(hsl_to_color(hue, 0.80, 0.65, a1)));
    c.fill_circle(x, y, radius);
    let a2 = (base_opacity * 1.3 + ctx.bass_energy * 0.2).clamp(0.0, 1.0);
    c.set_fill(Fill::Solid(hsl_to_color(hue, 0.90, 0.85, a2)));
    c.fill_circle(x - radius * 0.25, y - radius * 0.25, radius * 0.45);
  }
}

pub fn render_starfield(c: &mut GpuCanvas, ctx: &RenderContext, stars: &[Star]) {
  let bg = &ctx.config.background;
  let target = bg.star_count.unwrap_or(160).clamp(20, STAR_COUNT as u32) as usize;
  let speed_mult = bg.star_speed.unwrap_or(1.0);
  let brightness = bg.star_brightness.unwrap_or(1.0);
  let t = ctx.frame_time * speed_mult;
  let pulse = 0.7 + ctx.bass_energy * 0.4;
  for s in &stars[..target] {
    let raw_x = s.x * ctx.width + (t * s.speed + s.phase).sin() * 12.0;
    let raw_y = s.y * ctx.height + (t * s.speed * 0.7 + s.phase).cos() * 12.0;
    let x = raw_x.rem_euclid(ctx.width);
    let y = raw_y.rem_euclid(ctx.height);
    let twinkle = 0.4 + (t * (1.5 + s.speed * 4.0) + s.phase).sin() * 0.6;
    let alpha = (twinkle * pulse * (0.6 + ctx.beat_strength * 0.4) * brightness).min(1.0).max(0.0);
    c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, alpha)));
    c.fill_circle(x, y, s.size * pulse);
  }
}

pub fn render_nebula(c: &mut GpuCanvas, ctx: &RenderContext) {
  let bg = &ctx.config.background;
  let speed_mult = bg.nebula_speed.unwrap_or(1.0);
  let intensity_mult = bg.nebula_intensity.unwrap_or(0.6);
  let t = (ctx.frame_time / 7.0) * speed_mult;
  let intensity = (0.5 + ctx.bass_energy * 0.5) * intensity_mult;
  for i in 0..5 {
    let seed = i as f32 * 73.0;
    let cx = ((seed * 0.1 + t * (0.1 + i as f32 * 0.02)).sin() * 0.5 + 0.5) * ctx.width;
    let cy = ((seed * 0.13 + t * (0.08 + i as f32 * 0.03)).cos() * 0.5 + 0.5) * ctx.height;
    let r = 180.0 + (seed + t * 0.05).sin() * 80.0 + ctx.beat_strength * 100.0;
    let hue = (seed * 0.7 + t * 20.0 + i as f32 * 50.0) % 360.0;
    let g = Fill::radial_gradient(
      cx, cy, 0.0, cx, cy, r,
      &[
        (0.0, hsl_to_color(hue, 0.85, 0.65, 0.50 * intensity)),
        (0.5, hsl_to_color(hue + 30.0, 0.75, 0.45, 0.25 * intensity)),
        (1.0, hsl_to_color(hue + 60.0, 0.65, 0.25, 0.0)),
      ],
    );
    c.set_fill(g);
    c.fill_rect(0.0, 0.0, ctx.width, ctx.height);
  }
}

pub fn render_psychedelic(c: &mut GpuCanvas, ctx: &RenderContext) {
  if ctx.width <= 0.0 || ctx.height <= 0.0 {
    return;
  }
  let bg = &ctx.config.background;
  let speed_mult = bg.psychedelic_speed.unwrap_or(1.0).max(0.01);
  let target_bands = bg.psychedelic_bands.unwrap_or(24).max(1);
  let base_line_width = bg.psychedelic_line_width.unwrap_or(4.0).max(0.5);
  let t = (ctx.frame_time / 2.0) * speed_mult;
  let cx = ctx.width / 2.0;
  let cy = ctx.height / 2.0;
  let max_r = (ctx.width * ctx.width + ctx.height * ctx.height).sqrt() / 2.0;
  let bands = (target_bands + (ctx.beat_strength * 20.0) as u32).max(1);
  if max_r <= 0.0 {
    return;
  }
  for i in 0..bands {
    let r = (i as f32 / bands as f32) * max_r;
    let angle = r * 0.05 + t * (0.3 + ctx.bass_energy * 0.4) + i as f32 * 0.5;
    let hue = ((angle * 40.0 + t * 50.0) % 360.0 + 360.0) % 360.0;
    let alpha = (0.20 + ctx.bass_energy * 0.15).clamp(0.0, 1.0);
    let ring_radius = r + (t * 2.0 + i as f32).sin() * 12.0;
    if ring_radius <= 0.0 {
      continue;
    }
    c.set_stroke(Fill::Solid(hsl_to_color(hue, 0.95, 0.60, alpha)));
    c.set_line_width((base_line_width + ctx.beat_strength * 6.0).max(0.1));
    c.stroke_circle(cx, cy, ring_radius);
  }
}
