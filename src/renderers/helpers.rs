//! Shared state structs and helper functions for visualizer style renderers.

use crate::gpu2d::{Color, GpuCanvas};

// ---------------------------------------------------------------------------
// Shared State Structs for Styles
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

/// A single expanding firework burst (spawned on beats by `radialFireworksBurst`).
pub struct Firework {
  pub angle: f32,
  pub start_time: f32,
  pub speed: f32,
  pub color_phase: f32,
  pub sparks: usize,
}

pub struct AdvancedState {
  pub fire: Vec<FireParticle>,
  pub galaxy: Vec<GalaxyParticle>,
  pub galaxy_init: bool,
  pub sparks: Vec<Spark>,
  pub motes: Vec<LightMote>,
  pub peaks: Vec<Peak>,
  pub embers: Vec<Ember>,
  pub api_time: f32,
  pub frame_history: Vec<Vec<u8>>,
  pub notes: Vec<FloatingNote>,
  pub splatter: Vec<SplatterDot>,
  pub arc_rotation: f32,
  pub galaxy_rotation: f32,
  pub fireworks: Vec<Firework>,
  pub last_firework_beat: u64,
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
      embers: Vec::new(),
      api_time: 0.0,
      frame_history: Vec::new(),
      notes: Vec::new(),
      splatter: Vec::new(),
      arc_rotation: 0.0,
      galaxy_rotation: 0.0,
      fireworks: Vec::new(),
      last_firework_beat: 0,
    }
  }
}

// ---------------------------------------------------------------------------
// Common Helpers
// ---------------------------------------------------------------------------

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + (b - a) * t
}

pub fn mix(p: Color, s: Color, t: f32) -> Color {
  Color::rgba(lerp(p.r, s.r, t), lerp(p.g, s.g, t), lerp(p.b, s.b, t), 1.0)
}

pub fn bright(c: Color, f: f32) -> Color {
  Color::rgba(
    (c.r * f).clamp(0.0, 1.0),
    (c.g * f).clamp(0.0, 1.0),
    (c.b * f).clamp(0.0, 1.0),
    1.0,
  )
}

pub fn bin_sum(freq: &[u8], step: usize, idx: usize) -> f32 {
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
pub fn quadratic_wave(raw: &[(f32, f32)], steps: u32) -> Vec<(f32, f32)> {
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

/// Draws the user's Radial Center Image (uploaded to a persistent atlas layer)
/// as a circular texture centered at (cx, cy) with radius `r`. Returns `true`
/// when an image was drawn — callers fall back to their own disc otherwise.
pub fn draw_radial_center_image(
  c: &mut GpuCanvas,
  ctx: &crate::renderers::RenderContext,
  cx: f32,
  cy: f32,
  r: f32,
) -> bool {
  if let Some(img) = &ctx.state.radial_center_image {
    c.push_circular_textured_quad(img.layer, cx, cy, r, [0.0, 0.0, 1.0, 1.0], Color::WHITE);
    true
  } else {
    false
  }
}

/// Draws a **tapered radial bar** — wide at the base (`r_start`) narrowing to a
/// sharp point at the tip (`r_end`) — plus a soft glowing dot at the tip for the
/// "lancip blur" (pointed + blur) visual effect.
///
/// Parameters:
/// - `angle`: bar direction in radians
/// - `r_start` / `r_end`: inner and outer radii
/// - `base_half_width`: half-width (in radians) at the inner edge
/// - `bar_col`: fill color for the taper polygon
/// - `glow_col`: color of the tip glow dot
/// - `glow_radius`: radius of the tip glow dot
/// - `glow_sigma`: shadow blur sigma of the glow dot
/// - `line_width`: stroke width of the bright edge contour line (0 = no stroke)
#[allow(clippy::too_many_arguments)]
pub fn draw_tapered_bar(
  c: &mut GpuCanvas,
  cx: f32, cy: f32,
  angle: f32,
  r_start: f32, r_end: f32,
  base_half_width: f32,
  bar_col: Color,
  glow_col: Color,
  glow_radius: f32,
  glow_sigma: f32,
  line_width: f32,
) {
  use crate::gpu2d::Fill;

  let (cos_a, sin_a) = angle.sin_cos();
  let (cos_l, sin_l) = (angle - base_half_width).sin_cos();
  let (cos_r, sin_r) = (angle + base_half_width).sin_cos();

  // Triangle: two base corners + one tip point
  let base_l = (cx + cos_l * r_start, cy + sin_l * r_start);
  let base_r = (cx + cos_r * r_start, cy + sin_r * r_start);
  let tip    = (cx + cos_a * r_end,   cy + sin_a * r_end);

  // 1. Filled tapered triangle
  c.set_fill(Fill::Solid(bar_col));
  c.set_shadow(bar_col, glow_sigma * 0.7);
  c.fill_polygon(&[base_l, tip, base_r]);

  // 2. Optional bright edge stroke
  if line_width > 0.0 {
    c.set_stroke(Fill::Solid(mix(bar_col, Color::WHITE, 0.5)));
    c.set_line_width(line_width);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    let outline = [base_l, tip, base_r];
    c.stroke_polyline(&outline);
  }

  // 3. Soft glow dot at the tip — creates the "lancip blur" pointed-glow effect
  if glow_radius > 0.0 {
    c.set_fill(Fill::Solid(glow_col.with_alpha(0.55)));
    c.set_shadow(glow_col, glow_sigma);
    c.fill_circle(tip.0, tip.1, glow_radius);
  }
}

/// Fills a closed radial wave polygon around center `(cx, cy)` by fanning
/// individual triangles from `(cx, cy)` to `pts[i]` and `pts[i+1]`.
///
/// This completely eliminates the fan-overflow bug of naive `fill_polygon(&pts)`
/// (which fans from `pts[0]`), ensuring zero triangle slices ('bulu-bulu'/bristles)
/// spill outside the wave boundary.
pub fn fill_radial_polygon(c: &mut GpuCanvas, cx: f32, cy: f32, pts: &[(f32, f32)]) {
  if pts.len() < 2 {
    return;
  }
  for i in 0..pts.len() - 1 {
    c.fill_polygon(&[(cx, cy), pts[i], pts[i + 1]]);
  }
}

