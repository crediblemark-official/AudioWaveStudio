//! Text rendering via ab_glyph.
//!
//! Each call bakes a whole text run (including wave offsets, letter spacing
//! and outline) into a single RGBA atlas and returns one textured quad that
//! maps the run's ink bounding box to the full atlas. Colors are baked into
//! the atlas at glyph position, so gradient/outline text needs no per-vertex
//! color machinery and glyph overlap cannot double-blend.

use std::sync::OnceLock;

use ab_glyph::{Font, FontArc, GlyphId, OutlinedGlyph, PxScale, ScaleFont};

use super::scene::{Color, Fill};

/// Number of atlas layers reserved for text (images use layers 8+).
pub const TEXT_LAYERS: u32 = 20;
const MAX_ATLAS: u32 = 1024;
const PAD: f32 = 2.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextAlign {
  Left,
  Center,
  Right,
}

#[derive(Clone, Debug)]
pub struct TextOpts {
  pub letter_spacing: f32,
  pub outline: bool,
  pub outline_color: Color,
  pub outline_width: f32,
  pub wave: bool,
  pub wave_time: f32,
  pub wave_amp: f32,
  pub char_index_start: usize,
  pub bass: f32,
}

impl Default for TextOpts {
  fn default() -> Self {
    TextOpts {
      letter_spacing: 0.0,
      outline: false,
      outline_color: Color::BLACK,
      outline_width: 1.0,
      wave: false,
      wave_time: 0.0,
      wave_amp: 0.0,
      char_index_start: 0,
      bass: 0.0,
    }
  }
}

pub struct TextAtlas {
  pub rgba: Vec<u8>,
  pub atlas_w: u32,
  pub atlas_h: u32,
  pub left: f32,
  pub top: f32,
  pub width: f32,
  pub height: f32,
  pub ascent: f32,
  pub advance: f32,
}

// ---------------------------------------------------------------------------
// System font discovery (cached for the process lifetime).
// ---------------------------------------------------------------------------

struct FontSet {
  regular: FontArc,
  bold: FontArc,
  mono: FontArc,
  serif: FontArc,
}

static FONTS: OnceLock<Option<FontSet>> = OnceLock::new();

fn load_any(candidates: &[&str]) -> Option<FontArc> {
  for p in candidates {
    if let Ok(bytes) = std::fs::read(p) {
      if let Ok(f) = FontArc::try_from_vec(bytes) {
        return Some(f);
      }
    }
  }
  None
}

fn fc_match(hint: &str) -> Option<FontArc> {
  let out = std::process::Command::new("fc-match")
    .args(["-f", "%{file}", hint])
    .output()
    .ok()?;
  if !out.status.success() {
    return None;
  }
  let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
  if path.is_empty() || !std::path::Path::new(&path).exists() {
    return None;
  }
  let bytes = std::fs::read(&path).ok()?;
  FontArc::try_from_vec(bytes).ok()
}

const REGULAR_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
  "/usr/share/fonts/dejavu/DejaVuSans.ttf",
  "/usr/share/fonts/TTF/DejaVuSans.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
  "/usr/share/fonts/liberation-sans/LiberationSans-Regular.ttf",
  "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
  "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
  "C:\\Windows\\Fonts\\segoeui.ttf",
  "C:\\Windows\\Fonts\\arial.ttf",
  "/System/Library/Fonts/Supplemental/Arial.ttf",
  "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
];

const BOLD_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
  "C:\\Windows\\Fonts\\segoeuib.ttf",
  "C:\\Windows\\Fonts\\arialbd.ttf",
  "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
];

const MONO_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
  "C:\\Windows\\Fonts\\consola.ttf",
  "/System/Library/Fonts/Supplemental/Courier New.ttf",
];

const SERIF_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf",
  "C:\\Windows\\Fonts\\times.ttf",
  "/System/Library/Fonts/Supplemental/Times New Roman.ttf",
];

fn font_set() -> Option<&'static FontSet> {
  FONTS
    .get_or_init(|| {
      let regular = load_any(REGULAR_CANDIDATES).or_else(|| fc_match("sans-serif"))?;
      let reg = regular.clone();
      let bold = load_any(BOLD_CANDIDATES).or_else(|| fc_match("sans-serif:bold"));
      let mono = load_any(MONO_CANDIDATES).or_else(|| fc_match("monospace"));
      let serif = load_any(SERIF_CANDIDATES).or_else(|| fc_match("serif"));
      Some(FontSet {
        regular,
        bold: bold.unwrap_or_else(|| reg.clone()),
        mono: mono.unwrap_or_else(|| reg.clone()),
        serif: serif.unwrap_or_else(|| reg.clone()),
      })
    })
    .as_ref()
}

/// Pick a cached font for a family name + weight (600+ = bold).
pub fn select_font(family: &str, weight: f32) -> Option<&'static FontArc> {
  let set = font_set()?;
  let f = family.to_ascii_lowercase();
  if f.contains("mono") || f.contains("courier") || f.contains("consol") {
    Some(&set.mono)
  } else if f.contains("serif") && !f.contains("sans") {
    Some(&set.serif)
  } else if weight >= 600.0 {
    Some(&set.bold)
  } else {
    Some(&set.regular)
  }
}

/// Total advance (pen width incl. letter spacing) of a text run in px.
pub fn measure(font: &FontArc, text: &str, font_size: f32, letter_spacing: f32) -> f32 {
  let scale = PxScale::from(font_size);
  let scaled = font.as_scaled(scale);
  let mut pen = 0.0f32;
  let mut max_x = 0.0f32;
  for ch in text.chars() {
    let gid = font.glyph_id(ch);
    let adv = scaled.h_advance(gid);
    max_x = max_x.max(pen + adv);
    pen += adv + letter_spacing;
  }
  max_x.max(0.0)
}

/// Rasterize a text run into an RGBA atlas (color baked in, alpha = coverage).
pub fn rasterize(
  font: &FontArc,
  text: &str,
  font_size: f32,
  fill: &Fill,
  opts: &TextOpts,
) -> Option<TextAtlas> {
  if text.is_empty() {
    return None;
  }
  let scale = PxScale::from(font_size);
  let scaled = font.as_scaled(scale);
  let ascent = scaled.ascent();
  let line_h = (ascent - scaled.descent()).max(1.0);

  let chars: Vec<char> = text.chars().collect();

  struct Placed {
    gid: GlyphId,
    x: f32,
    y: f32,
  }
  let mut placed: Vec<Placed> = Vec::with_capacity(chars.len());
  let mut pen = 0.0f32;
  let mut max_x = 0.0f32;
  for (i, &ch) in chars.iter().enumerate() {
    let gid = font.glyph_id(ch);
    let adv = scaled.h_advance(gid);
    let mut y = 0.0f32;
    if opts.wave {
      let t = opts.wave_time * 5.0 + (opts.char_index_start + i) as f32 * 0.6;
      y = opts.wave_amp * (0.4 + opts.bass * 0.6) * t.sin();
    }
    placed.push(Placed { gid, x: pen, y });
    max_x = max_x.max(pen + adv);
    pen += adv + opts.letter_spacing;
  }
  let advance = max_x.max(0.0);
  if advance <= 0.0 {
    return None;
  }

  let atlas_w = (advance + PAD * 2.0).ceil() as u32;
  let atlas_h = (line_h + PAD * 2.0).ceil() as u32;
  if atlas_w > MAX_ATLAS || atlas_h > MAX_ATLAS {
    return None;
  }

  let mut buf = vec![0u8; (atlas_w as usize) * (atlas_h as usize) * 4];

  let mut min_x = f32::MAX;
  let mut min_y = f32::MAX;
  let mut max_ix = f32::MIN;
  let mut max_iy = f32::MIN;

  for p in &placed {
    let glyph = p.gid.with_scale(scale);
    let Some(og) = font.outline_glyph(glyph) else { continue };
    let bounds = og.px_bounds();
    let bx = PAD + p.x;
    let by = PAD + ascent + p.y;

    if opts.outline && opts.outline_width > 0.0 {
      let ow = opts.outline_width;
      let dirs = [
        (-ow, 0.0),
        (ow, 0.0),
        (0.0, -ow),
        (0.0, ow),
        (-ow, -ow),
        (ow, -ow),
        (-ow, ow),
        (ow, ow),
      ];
      for (dx, dy) in dirs {
        draw_outline(&mut buf, atlas_w, atlas_h, &og, bx + dx, by + dy, opts.outline_color);
      }
    }
    draw_fill(&mut buf, atlas_w, atlas_h, &og, bx, by, fill);

    min_x = min_x.min(bx + bounds.min.x);
    min_y = min_y.min(by + bounds.min.y);
    max_ix = max_ix.max(bx + bounds.max.x);
    max_iy = max_iy.max(by + bounds.max.y);
  }

  if min_x == f32::MAX {
    return None;
  }

  Some(TextAtlas {
    rgba: buf,
    atlas_w,
    atlas_h,
    left: min_x,
    top: min_y,
    width: (max_ix - min_x).max(1.0),
    height: (max_iy - min_y).max(1.0),
    ascent,
    advance,
  })
}

fn blend_px(buf: &mut [u8], atlas_w: u32, atlas_h: u32, px: f32, py: f32, color: Color, cov: f32) {
  let x = px.floor() as i32;
  let y = py.floor() as i32;
  if x < 0 || y < 0 || x >= atlas_w as i32 || y >= atlas_h as i32 {
    return;
  }
  let idx = (y as usize * atlas_w as usize + x as usize) * 4;
  let a = (cov * 255.0) as u8;
  if a > buf[idx + 3] {
    buf[idx] = (color.r * 255.0) as u8;
    buf[idx + 1] = (color.g * 255.0) as u8;
    buf[idx + 2] = (color.b * 255.0) as u8;
    buf[idx + 3] = a;
  }
}

fn draw_outline(buf: &mut [u8], w: u32, h: u32, og: &OutlinedGlyph, bx: f32, by: f32, color: Color) {
  og.draw(|u, v, cov| {
    blend_px(buf, w, h, bx + u as f32, by + v as f32, color, cov);
  });
}

fn draw_fill(buf: &mut [u8], w: u32, h: u32, og: &OutlinedGlyph, bx: f32, by: f32, fill: &Fill) {
  og.draw(|u, v, cov| {
    let px = bx + u as f32;
    let py = by + v as f32;
    let color = match fill {
      Fill::Solid(c) => *c,
      Fill::Gradient(g) => g.sample(px, py),
    };
    blend_px(buf, w, h, px, py, color, cov);
  });
}
