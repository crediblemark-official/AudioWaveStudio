//! Shared state structs and helper functions for complex visualizer styles.

use crate::gpu2d::{Color, GpuCanvas};

// ---------------------------------------------------------------------------
// Shared State Structs
// ---------------------------------------------------------------------------

pub struct FireParticle {
  pub x: f32,
  pub y: f32,
  pub vx: f32,
  pub vy: f32,
  pub size: f32,
  pub alpha: f32,
  pub life: f32,
  pub max_life: f32,
  pub heat: f32,
}

pub struct GalaxyParticle {
  pub angle: f32,
  pub radius: f32,
  pub speed: f32,
  pub size: f32,
  pub arm: u32,
  pub offset: f32,
}

pub struct Spark {
  pub x: f32,
  pub y: f32,
  pub vx: f32,
  pub vy: f32,
  pub life: f32,
  pub max_life: f32,
  pub size: f32,
  pub color: Color,
  pub decay: f32,
  pub trail: Vec<(f32, f32)>,
}

pub struct LightMote {
  pub x: f32,
  pub y: f32,
  pub vx: f32,
  pub vy: f32,
  pub size: f32,
  pub alpha: f32,
  pub phase: f32,
}

pub struct Peak {
  pub x: f32,
  pub y: f32,
  pub alpha: f32,
}

pub struct Ember {
  pub x: f32,
  pub y: f32,
  pub vx: f32,
  pub vy: f32,
  pub size: f32,
  pub life: f32,
  pub max_life: f32,
}

pub struct FloatingNote {
  pub x: f32,
  pub y: f32,
  pub vx: f32,
  pub vy: f32,
  pub symbol: char,
  pub size: f32,
  pub alpha: f32,
  pub rotation: f32,
  pub rot_speed: f32,
}

pub struct SplatterDot {
  pub x: f32,
  pub y: f32,
  pub r: f32,
}

pub struct AdvancedState {
  pub fire: Vec<FireParticle>,
  pub galaxy: Vec<GalaxyParticle>,
  pub galaxy_init: bool,
  pub sparks: Vec<Spark>,
  pub motes: Vec<LightMote>,
  pub peaks: Vec<Peak>,
  pub three_d_prev_beat: f32,
  pub three_d_rot: f32,
  pub embers: Vec<Ember>,
  pub api_time: f32,
  pub frame_history: Vec<Vec<u8>>,
  pub notes: Vec<FloatingNote>,
  pub splatter: Vec<SplatterDot>,
  pub arc_rotation: f32,
  pub galaxy_rotation: f32,
}

impl Default for AdvancedState {
  fn default() -> Self {
    AdvancedState {
      fire: Vec::new(),
      galaxy: Vec::new(),
      galaxy_init: false,
      sparks: Vec::new(),
      motes: Vec::new(),
      peaks: Vec::new(),
      three_d_prev_beat: 0.0,
      three_d_rot: 0.0,
      embers: Vec::new(),
      api_time: 0.0,
      frame_history: Vec::new(),
      notes: Vec::new(),
      splatter: Vec::new(),
      arc_rotation: 0.0,
      galaxy_rotation: 0.0,
    }
  }
}

// ---------------------------------------------------------------------------
// Common Helpers
// ---------------------------------------------------------------------------

pub(crate) fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + (b - a) * t
}

pub(crate) fn mix(p: Color, s: Color, t: f32) -> Color {
  Color::rgba(lerp(p.r, s.r, t), lerp(p.g, s.g, t), lerp(p.b, s.b, t), 1.0)
}

pub(crate) fn bright(c: Color, f: f32) -> Color {
  Color::rgba(
    (c.r * f).clamp(0.0, 1.0),
    (c.g * f).clamp(0.0, 1.0),
    (c.b * f).clamp(0.0, 1.0),
    1.0,
  )
}

pub(crate) fn bin_sum(freq: &[u8], step: usize, idx: usize) -> f32 {
  let mut sum = 0usize;
  let mut n = 0;
  for j in 0..step {
    let k = idx * step + j;
    if k < freq.len() {
      sum += freq[k] as usize;
      n += 1;
    }
  }
  if n == 0 {
    return 0.0;
  }
  sum as f32 / (n as f32 * 255.0)
}

/// Canvas `quadraticCurveTo(prev, mid)` woven polyline.
pub(crate) fn quadratic_wave(raw: &[(f32, f32)], steps: u32) -> Vec<(f32, f32)> {
  if raw.is_empty() {
    return Vec::new();
  }
  let mut out = Vec::with_capacity(raw.len() * (steps as usize + 1));
  let mut prev_end = raw[0];
  out.push(prev_end);
  for i in 1..raw.len() {
    let ctrl = raw[i - 1];
    let end = ((raw[i - 1].0 + raw[i].0) / 2.0, (raw[i - 1].1 + raw[i].1) / 2.0);
    let seg = GpuCanvas::sample_quadratic(prev_end, ctrl, end, steps);
    for p in seg.iter().skip(1) {
      out.push(*p);
    }
    prev_end = end;
  }
  out
}
