//! Text rendering via ab_glyph (outlines) + rustybuzz (HarfBuzz shaping).
//!
//! Each call bakes a whole text run (including wave offsets, letter spacing
//! and outline) into a single RGBA atlas and returns one textured quad that
//! maps the run's ink bounding box to the full atlas. Colors are baked into
//! the atlas at glyph position, so gradient/outline text needs no per-vertex
//! color machinery and glyph overlap cannot double-blend.
//!
//! Arabic (RTL) runs are shaped with rustybuzz/HarfBuzz so letters join into
//! connected forms and are laid out right-to-left; the shaped glyphs are then
//! rasterized with ab_glyph outlines. Non-Arabic runs keep the simple
//! char-by-char advance path. Shaped runs also drive `measure` so wrapping
//! and alignment stay consistent with what gets drawn.

use std::sync::OnceLock;

use ab_glyph::{Font as _, FontArc, GlyphId, OutlinedGlyph, PxScale, ScaleFont};

use super::scene::{Color, Fill};

/// Number of atlas layers reserved for text (images use layers 8+).
pub const TEXT_LAYERS: u32 = 20;
const MAX_ATLAS: u32 = 1024;
/// Atlas padding (px) around the ink/pen bounds. Also the atlas-local
/// position of the run's pen start (linear path) — `gradient_fill` uses it to
/// cancel the padding when anchoring gradients to the visual line center.
pub const PAD: f32 = 2.0;

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
  /// Atlas-x of the run's pen start (linear: `PAD`; shaped: `ox - pen_min`).
  /// `draw_text` aligns this to `x + alignOffset`, mirroring canvas `fillText`
  /// textAlign (which positions the pen, not the ink box).
  pub pen_x: f32,
  /// Atlas-y of the run's baseline (linear: `PAD + ascent`; shaped: `oy`).
  /// `draw_text` places this exactly at the `y` passed in, like `fillText`.
  pub baseline: f32,
}

// ---------------------------------------------------------------------------
// System font discovery (cached for the process lifetime).
// ---------------------------------------------------------------------------

/// A loaded font: `arc` provides glyph outlines/advances (ab_glyph) and
/// `bytes` backs the rustybuzz/HarfBuzz shaping face. Bytes are leaked into
/// `'static` because the font set lives for the whole process.
#[derive(Clone)]
pub struct Font {
  pub arc: FontArc,
  pub bytes: &'static [u8],
}

impl Font {
  /// A rustybuzz face borrowing this font's bytes. Built per call (the parsed
  /// face is a cheap view over the data; shaping itself dominates).
  pub fn hb_face(&self) -> Option<rustybuzz::Face<'_>> {
    rustybuzz::Face::from_slice(self.bytes, 0)
  }
}

struct FontSet {
  regular: Font,
  bold: Font,
  italic: Font,
  bold_italic: Font,
  mono: Font,
  mono_bold: Font,
  mono_italic: Font,
  mono_bold_italic: Font,
  serif: Font,
  serif_italic: Font,
  arabic: Option<Font>,
}

static FONTS: OnceLock<Option<FontSet>> = OnceLock::new();

fn build_font(bytes: Vec<u8>) -> Option<Font> {
  // ab_glyph takes ownership of its copy; leak another copy for rustybuzz.
  let arc = FontArc::try_from_vec(bytes.clone()).ok()?;
  let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
  Some(Font { arc, bytes: leaked })
}

fn load_any(candidates: &[&str]) -> Option<Font> {
  for p in candidates {
    if let Ok(bytes) = std::fs::read(p) {
      if let Some(f) = build_font(bytes) {
        return Some(f);
      }
    }
  }
  None
}

fn fc_match(hint: &str) -> Option<Font> {
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
  build_font(bytes)
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

const ITALIC_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/dejavu/DejaVuSans-Oblique.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationSans-Italic.ttf",
  "/usr/share/fonts/truetype/freefont/FreeSansOblique.ttf",
  "C:\\Windows\\Fonts\\segoeuii.ttf",
  "C:\\Windows\\Fonts\\ariali.ttf",
  "/System/Library/Fonts/Supplemental/Arial Italic.ttf",
];

const BOLD_ITALIC_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/dejavu/DejaVuSans-BoldOblique.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationSans-BoldItalic.ttf",
  "C:\\Windows\\Fonts\\segoeuiz.ttf",
  "C:\\Windows\\Fonts\\arialbi.ttf",
  "/System/Library/Fonts/Supplemental/Arial Bold Italic.ttf",
];

const MONO_ITALIC_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Oblique.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationMono-Italic.ttf",
  "C:\\Windows\\Fonts\\consolai.ttf",
  "/System/Library/Fonts/Supplemental/Courier New Italic.ttf",
];

const MONO_BOLD_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationMono-Bold.ttf",
  "C:\\Windows\\Fonts\\consolab.ttf",
  "/System/Library/Fonts/Supplemental/Courier New Bold.ttf",
];

const MONO_BOLD_ITALIC_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-BoldOblique.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationMono-BoldItalic.ttf",
  "C:\\Windows\\Fonts\\consolaz.ttf",
  "/System/Library/Fonts/Supplemental/Courier New Bold Italic.ttf",
];

const SERIF_ITALIC_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/dejavu/DejaVuSerif-Italic.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationSerif-Italic.ttf",
  "C:\\Windows\\Fonts\\timesi.ttf",
  "/System/Library/Fonts/Supplemental/Times New Roman Italic.ttf",
];

const ARABIC_CANDIDATES: &[&str] = &[
  "/usr/share/fonts/truetype/noto/NotoSansArabic-Regular.ttf",
  "/usr/share/fonts/noto/NotoSansArabic-Regular.ttf",
  "/usr/share/fonts/truetype/noto/NotoNaskhArabic-Regular.ttf",
  "/usr/share/fonts/google-noto/NotoSansArabic-Regular.ttf",
  "C:\\Windows\\Fonts\\seguiemj.ttf",
  "C:\\Windows\\Fonts\\arial.ttf",
];

fn font_set() -> Option<&'static FontSet> {
  FONTS
    .get_or_init(|| {
      let regular = load_any(REGULAR_CANDIDATES).or_else(|| fc_match("sans-serif"))?;
      let reg = regular.clone();
      let bold = load_any(BOLD_CANDIDATES).or_else(|| fc_match("sans-serif:bold"));
      let italic = load_any(ITALIC_CANDIDATES).or_else(|| fc_match("sans-serif:italic"));
      let bold_italic = load_any(BOLD_ITALIC_CANDIDATES)
        .or_else(|| fc_match("sans-serif:bold:italic"))
        .or_else(|| fc_match("sans-serif:italic:bold"));
      let mono = load_any(MONO_CANDIDATES).or_else(|| fc_match("monospace"));
      let mono_bold = load_any(MONO_BOLD_CANDIDATES)
        .or_else(|| fc_match("monospace:bold"))
        .or_else(|| fc_match("monospace:bold:italic"));
      let mono_italic = load_any(MONO_ITALIC_CANDIDATES).or_else(|| fc_match("monospace:italic"));
      let mono_bold_italic = load_any(MONO_BOLD_ITALIC_CANDIDATES)
        .or_else(|| fc_match("monospace:bold:italic"))
        .or_else(|| fc_match("monospace:italic:bold"));
      let serif = load_any(SERIF_CANDIDATES).or_else(|| fc_match("serif"));
      let serif_italic = load_any(SERIF_ITALIC_CANDIDATES).or_else(|| fc_match("serif:italic"));
      let arabic = load_any(ARABIC_CANDIDATES).or_else(|| fc_match("arabic")).or_else(|| fc_match("Noto Sans Arabic"));
      Some(FontSet {
        regular,
        bold: bold.clone().unwrap_or_else(|| reg.clone()),
        italic: italic.unwrap_or_else(|| reg.clone()),
        bold_italic: bold_italic.unwrap_or_else(|| bold.clone().unwrap_or_else(|| reg.clone())),
        mono: mono.clone().unwrap_or_else(|| reg.clone()),
        mono_bold: mono_bold
          .clone()
          .unwrap_or_else(|| mono.clone().unwrap_or_else(|| reg.clone())),
        mono_italic: mono_italic.unwrap_or_else(|| mono.clone().unwrap_or_else(|| reg.clone())),
        mono_bold_italic: mono_bold_italic
          .unwrap_or_else(|| mono_bold.clone().unwrap_or_else(|| mono.clone().unwrap_or_else(|| reg.clone()))),
        serif: serif.clone().unwrap_or_else(|| reg.clone()),
        serif_italic: serif_italic.unwrap_or_else(|| serif.clone().unwrap_or_else(|| reg.clone())),
        arabic,
      })
    })
    .as_ref()
}

pub fn is_arabic_text(text: &str) -> bool {
  let has_arabic = text.chars().any(|ch| matches!(ch, '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}' | '\u{08A0}'..='\u{08FF}'));
  let has_ascii_alnum = text.chars().any(|ch| ch.is_ascii_alphanumeric());
  has_arabic && !has_ascii_alnum
}

/// Pick a cached font for a family name + weight (600+ = bold).
pub fn select_font(family: &str, weight: f32) -> Option<&'static Font> {
  select_font_for_text_style(family, weight, false, "")
}

/// Pick a cached font for a family name + weight + italic style, falling back
/// to the nearest non-italic style when no italic face is installed.
pub fn select_font_for_text(family: &str, weight: f32, text: &str) -> Option<&'static Font> {
  select_font_for_text_style(family, weight, false, text)
}

/// Like `select_font_for_text`, but honors the italic style. Mirrors the TS
/// renderer, which sets `c.font = `${block.italic ? 'italic ' : ''}${weight}px ${family}``.
pub fn select_font_for_text_style(
  family: &str,
  weight: f32,
  italic: bool,
  text: &str,
) -> Option<&'static Font> {
  let set = font_set()?;
  if is_arabic_text(text) {
    if let Some(ref arabic_font) = set.arabic {
      return Some(arabic_font);
    }
  }
  let f = family.to_ascii_lowercase();
  if f.contains("mono") || f.contains("courier") || f.contains("consol") {
    // Mirror canvas: `700px monospace` resolves to the BOLD mono face, so the
    // weight must be honored here too (previously always regular -> thin text).
    Some(if weight >= 600.0 {
      if italic { &set.mono_bold_italic } else { &set.mono_bold }
    } else if italic {
      &set.mono_italic
    } else {
      &set.mono
    })
  } else if f.contains("serif") && !f.contains("sans") {
    Some(if italic { &set.serif_italic } else { &set.serif })
  } else if weight >= 600.0 {
    Some(if italic { &set.bold_italic } else { &set.bold })
  } else {
    Some(if italic { &set.italic } else { &set.regular })
  }
}

// ---------------------------------------------------------------------------
// HarfBuzz shaping (Arabic / RTL runs)
// ---------------------------------------------------------------------------

/// A shaped glyph with pixel-space placement data.
struct ShapedGlyph {
  gid: GlyphId,
  /// Pen advance in px, letter spacing already folded in.
  x_advance: f32,
  /// Horizontal offset in px (HarfBuzz x is right-positive).
  x_offset: f32,
  /// Vertical offset in px, down-positive (HarfBuzz y is up-positive).
  y_offset: f32,
}

/// Shape a run with HarfBuzz: applies GSUB (Arabic letter joining) and GPOS.
///
/// NOTE (verified empirically on rustybuzz 0.20): RTL runs are returned in
/// VISUAL order (first glyph = the string's last character = leftmost) with
/// POSITIVE advances, unlike classic HarfBuzz (logical order + negative
/// advances). Placing each glyph at the running pen therefore renders the
/// line left-to-right correctly.
///
/// `letter_spacing` is folded into the advance like the linear path so
/// `measure` and `rasterize` always agree.
///
/// LIMITATION: `guess_segment_properties` picks ONE direction from the first
/// strong character, so a mixed "Arabic + Latin" line is shaped as a single
/// run in one direction (no bidi). Proper mixed-script rendering needs a bidi
/// algorithm + per-direction runs (future work).
fn shape_run(font: &Font, text: &str, font_size: f32, letter_spacing: f32) -> Option<Vec<ShapedGlyph>> {
  if text.is_empty() {
    return Some(Vec::new());
  }
  let face = font.hb_face()?;
  let mut buffer = rustybuzz::UnicodeBuffer::new();
  buffer.push_str(text);
  buffer.guess_segment_properties();
  let out = rustybuzz::shape(&face, &[], buffer);
  let scale = font_size / face.units_per_em() as f32;
  let infos = out.glyph_infos();
  let positions = out.glyph_positions();
  let mut glyphs = Vec::with_capacity(infos.len());
  for (info, pos) in infos.iter().zip(positions.iter()) {
    glyphs.push(ShapedGlyph {
      gid: GlyphId(info.glyph_id as u16),
      x_advance: pos.x_advance as f32 * scale + letter_spacing,
      x_offset: pos.x_offset as f32 * scale,
      y_offset: -(pos.y_offset as f32 * scale),
    });
  }
  Some(glyphs)
}

/// Visual width (px) of a shaped run: pen extent, positive for both LTR and
/// RTL (RTL advances are negative, so the pen spans [sum, 0]).
fn shaped_width(glyphs: &[ShapedGlyph]) -> f32 {
  let mut pen = 0.0f32;
  let mut min_p = 0.0f32;
  let mut max_p = 0.0f32;
  for g in glyphs {
    pen += g.x_advance;
    min_p = min_p.min(pen);
    max_p = max_p.max(pen);
  }
  (max_p - min_p).max(0.0)
}

// ---------------------------------------------------------------------------
// Measurement & rasterization
// ---------------------------------------------------------------------------

/// Total advance (pen width incl. letter spacing) of a text run in px.
pub fn measure(font: &Font, text: &str, font_size: f32, letter_spacing: f32) -> f32 {
  if is_arabic_text(text) {
    return match shape_run(font, text, font_size, letter_spacing) {
      Some(glyphs) => shaped_width(&glyphs),
      None => 0.0,
    };
  }
  let scale = px_scale(font, font_size);
  let scaled = font.arc.as_scaled(scale);
  let mut pen = 0.0f32;
  let mut max_x = 0.0f32;
  for ch in text.chars() {
    let gid = font.arc.glyph_id(ch);
    let adv = scaled.h_advance(gid);
    max_x = max_x.max(pen + adv);
    pen += adv + letter_spacing;
  }
  max_x.max(0.0)
}

/// A `PxScale` that maps the font's units-per-em to `font_size` pixels, the
/// way Canvas/browser text does.
///
/// NOTE: plain `PxScale::from(font_size)` does NOT do this — ab_glyph's
/// `h_scale_factor()` divides by `height_unscaled` (ascent − descent), while
/// browsers/Skia divide by `units_per_em`. For fonts where the two differ
/// (DejaVuSansMono: height 2384 vs upm 2048) every glyph advance and outline
/// is silently squashed by ~14%, making Rust text visibly narrower than the
/// TS reference. Scaling `font_size` by `height_unscaled / upm` restores the
/// canvas behaviour (`h_scale_factor` then yields `font_size / upm`).
fn px_scale(font: &Font, font_size: f32) -> PxScale {
  let upm = font.arc.units_per_em().unwrap_or_else(|| font.arc.height_unscaled());
  PxScale::from(font_size * font.arc.height_unscaled() / upm)
}

/// The scaled ascent (px) of `font` at `font_size`. `rasterize_linear` places
/// the run's baseline at atlas-y `PAD + ascent`, so text renderers use this to
/// anchor gradients/shadows to the baseline (mirroring canvas `fillText`, where
/// the baseline is the y passed in).
pub fn ascent(font: &Font, font_size: f32) -> f32 {
  font.arc.as_scaled(px_scale(font, font_size)).ascent()
}

/// Rasterize a text run into an RGBA atlas (color baked in, alpha = coverage).
pub fn rasterize(
  font: &Font,
  text: &str,
  font_size: f32,
  fill: &Fill,
  opts: &TextOpts,
) -> Option<TextAtlas> {
  if text.is_empty() {
    return None;
  }
  if is_arabic_text(text) {
    return rasterize_shaped(font, text, font_size, fill, opts);
  }
  rasterize_linear(font, text, font_size, fill, opts)
}

/// Non-Arabic path: char-by-char advance, one glyph per char.
fn rasterize_linear(
  font: &Font,
  text: &str,
  font_size: f32,
  fill: &Fill,
  opts: &TextOpts,
) -> Option<TextAtlas> {
  let scale = px_scale(font, font_size);
  let scaled = font.arc.as_scaled(scale);
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
    let gid = font.arc.glyph_id(ch);
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
    if p.gid.0 == 0 { continue; }
    let glyph = p.gid.with_scale(scale);
    let Some(og) = font.arc.outline_glyph(glyph) else { continue };
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
    pen_x: PAD,
    baseline: PAD + ascent,
  })
}

/// Arabic/RTL path: glyphs come from HarfBuzz shaping (joined forms, visual
/// order), positioned with the shaped advances/offsets.
fn rasterize_shaped(
  font: &Font,
  text: &str,
  font_size: f32,
  fill: &Fill,
  opts: &TextOpts,
) -> Option<TextAtlas> {
  let glyphs = shape_run(font, text, font_size, opts.letter_spacing)?;
  if glyphs.is_empty() {
    return None;
  }
  let scale = px_scale(font, font_size);
  let scaled = font.arc.as_scaled(scale);
  let ascent = scaled.ascent();
  let advance = shaped_width(&glyphs);

  // Pen positions; for RTL the pen walks negative, so normalize by pen_min so
  // the leftmost glyph lands at x = 0.
  let mut pens = Vec::with_capacity(glyphs.len() + 1);
  pens.push(0.0f32);
  for g in &glyphs {
    pens.push(pens.last().unwrap() + g.x_advance);
  }
  let pen_min = pens.iter().cloned().fold(f32::MAX, f32::min);

  // Pass 1: compute placed ink bounds so the atlas fits exactly (marks can
  // overhang the pen extents).
  let mut placed: Vec<(f32, f32, GlyphId)> = Vec::with_capacity(glyphs.len());
  let mut min_x = f32::MAX;
  let mut min_y = f32::MAX;
  let mut max_ix = f32::MIN;
  let mut max_iy = f32::MIN;
  for (i, g) in glyphs.iter().enumerate() {
    if g.gid.0 == 0 { continue; }
    let mut y = g.y_offset;
    if opts.wave {
      let t = opts.wave_time * 5.0 + (opts.char_index_start + i) as f32 * 0.6;
      y += opts.wave_amp * (0.4 + opts.bass * 0.6) * t.sin();
    }
    let x = pens[i] - pen_min + g.x_offset;
    let glyph = g.gid.with_scale(scale);
    let Some(og) = font.arc.outline_glyph(glyph) else { continue };
    let b = og.px_bounds();
    min_x = min_x.min(x + b.min.x);
    min_y = min_y.min(y + b.min.y);
    max_ix = max_ix.max(x + b.max.x);
    max_iy = max_iy.max(y + b.max.y);
    if opts.outline && opts.outline_width > 0.0 {
      let ow = opts.outline_width;
      min_x = min_x.min(x + b.min.x - ow);
      min_y = min_y.min(y + b.min.y - ow);
      max_ix = max_ix.max(x + b.max.x + ow);
      max_iy = max_iy.max(y + b.max.y + ow);
    }
    placed.push((x, y, g.gid));
  }
  if placed.is_empty() || min_x == f32::MAX {
    return None;
  }

  let width = (max_ix - min_x).max(1.0);
  let height = (max_iy - min_y).max(1.0);
  let atlas_w = (width + PAD * 2.0).ceil() as u32;
  let atlas_h = (height + PAD * 2.0).ceil() as u32;
  if atlas_w > MAX_ATLAS || atlas_h > MAX_ATLAS {
    return None;
  }
  // Shift so the ink bounding box maps to (PAD, PAD) inside the atlas.
  let ox = PAD - min_x;
  let oy = PAD - min_y;

  let mut buf = vec![0u8; (atlas_w as usize) * (atlas_h as usize) * 4];

  for (x, y, gid) in placed {
    let glyph = gid.with_scale(scale);
    let Some(og) = font.arc.outline_glyph(glyph) else { continue };
    let bx = x + ox;
    let by = y + oy;
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
  }

  Some(TextAtlas {
    rgba: buf,
    atlas_w,
    atlas_h,
    left: PAD,
    top: PAD,
    width,
    height,
    ascent,
    advance,
    // Pen start: normalized pen 0 (pens[0] - pen_min) shifted by ox into the
    // atlas. Baseline: glyphs sit at `y_offset + oy`, so the baseline
    // (y_offset = 0) is at atlas-y `oy`.
    pen_x: ox - pen_min,
    baseline: oy,
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
  // ab_glyph's `draw()` closure emits RASTER-LOCAL pixel coordinates: (0,0) is
  // the glyph's px_bounds top-left, NOT its baseline origin. The bounds min
  // must be added back to reach glyph-space (baseline-relative) coordinates
  // before offsetting by the atlas anchor (bx, by). Without this, a glyph's
  // TOP lands on the baseline — shifting the whole run ~ascent px too low and
  // clipping it at the bottom of the atlas (visible as a dim flat band instead
  // of text, since the quad then samples empty atlas space).
  let b = og.px_bounds();
  og.draw(|u, v, cov| {
    blend_px(buf, w, h, bx + b.min.x + u as f32, by + b.min.y + v as f32, color, cov);
  });
}

fn draw_fill(buf: &mut [u8], w: u32, h: u32, og: &OutlinedGlyph, bx: f32, by: f32, fill: &Fill) {
  // Same raster-local -> glyph-space correction as draw_outline.
  let b = og.px_bounds();
  og.draw(|u, v, cov| {
    let px = bx + b.min.x + u as f32;
    let py = by + b.min.y + v as f32;
    let color = match fill {
      Fill::Solid(c) => *c,
      Fill::Gradient(g) => g.sample(px, py),
    };
    blend_px(buf, w, h, px, py, color, cov);
  });
}
