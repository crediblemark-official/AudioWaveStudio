//! Rust port of `src/services/renderers/textOverlay.ts` (export path).

use std::f32::consts::TAU;

use crate::config::{TextAlign, TextBlock, TextTransform};
use crate::gpu2d::text::{self, TextOpts};
use crate::gpu2d::{Color, Fill, GpuCanvas};

use super::RenderContext;

const FADE_S: f32 = 0.8;

/// Mirrors `textOverlay.ts` `fadeFactor`:
///
/// ```ts
/// function fadeFactor(isPlaying: boolean, frameTime: number): number {
///   if (isPlaying && !wasPlaying) playStartFrame = frameTime;
///   wasPlaying = isPlaying;
///   if (!isPlaying) return 1;
///   return Math.min(1, Math.max(0, (frameTime - playStartFrame) / FADE_MS));
/// }
/// ```
///
/// `play_start_frame` / `was_playing` are persistent state (mirroring the TS
/// module-level variables) so the fade restarts exactly when playback begins
/// and the text is fully visible whenever playback is paused.
pub fn fade_factor(
  is_playing: bool,
  frame_time: f32,
  play_start_frame: &mut f32,
  was_playing: &mut bool,
) -> f32 {
  if is_playing && !*was_playing {
    *play_start_frame = frame_time;
  }
  *was_playing = is_playing;
  if !is_playing {
    return 1.0;
  }
  ((frame_time - *play_start_frame) / FADE_S).clamp(0.0, 1.0)
}

fn align_of(align: TextAlign) -> text::TextAlign {
  match align {
    TextAlign::Left => text::TextAlign::Left,
    TextAlign::Center => text::TextAlign::Center,
    TextAlign::Right => text::TextAlign::Right,
  }
}

fn apply_transform(text: &str, transform: &TextTransform) -> String {
  match transform {
    TextTransform::None => text.to_string(),
    TextTransform::Uppercase => text.to_uppercase(),
    TextTransform::Lowercase => text.to_lowercase(),
    TextTransform::Capitalize => {
      let mut out = String::new();
      let mut cap = true;
      for ch in text.chars() {
        if ch.is_alphanumeric() {
          if cap {
            out.extend(ch.to_uppercase());
          } else {
            out.push(ch);
          }
          cap = false;
        } else {
          out.push(ch);
          cap = true;
        }
      }
      out
    }
  }
}

/// Mirrors `textOverlay.ts` `drawBlock` anchor computation:
/// `const anchorX = (block.positionX / 100) * width;`
/// `const anchorY = (block.positionY / 100) * height;`
///
/// There is NO special-casing of (0,0) on the TS side — a block at the
/// origin renders literally at the top-left corner, so the Rust port must
/// behave identically.
pub fn block_anchor(block: &TextBlock, width: f32, height: f32) -> (f32, f32) {
  ((block.position_x / 100.0) * width, (block.position_y / 100.0) * height)
}

/// Build the per-line linear gradient fill.
///
/// `width` is the line width WITHOUT letter spacing (TS measures with
/// `c.measureText(line).width`, spacing is applied per-char at draw time),
/// `ascent` is the selected font's scaled ascent, and `y_delta` is this line's
/// distance below the block anchor (`i * lineHeight`).
///
/// textOverlay.ts anchors the gradient's center at the line's VISUAL center
/// and at the block's `anchorY` for EVERY line:
/// ```ts
/// const centerX = lineStartX + lineWidth / 2; // per-align, always the visual
/// const g = c.createLinearGradient(           // line center (drawLine shifts
///   centerX - (dx * span) / 2, anchorY - (dy * span) / 2,  // the run by the
///   centerX + (dx * span) / 2, anchorY + (dy * span) / 2); // same amount)
/// ```
/// `draw_text` now maps the atlas to canvas 1:1 (UV-cropped to the ink) with
/// the quad placed so canvas_x(pen_x) = anchorX + dx and
/// canvas_y(baseline) = anchorY + i*lineHeight. So an atlas-local point `a`
/// lands on canvas `a + (anchorX + dx - pen_x, anchorY + i*lineHeight -
/// baseline)`. The gradient is baked at glyph positions in ATLAS space, so the
/// atlas-local axis center that reproduces TS (`centerX = lineStartX +
/// lineWidth/2`, `centerY = anchorY` for every line) is:
/// - `cx = width/2 + pen_x` — with `pen_x = PAD` (linear path) this is
///   `width/2 + PAD`, leaving the canvas center at `anchorX + dx + width/2`
///   (the line's visual center; align shifts quad AND gradient by `dx`, so it
///   cancels — the geometry is align-independent like TS);
/// - `cy = baseline - y_delta` — with `baseline = PAD + ascent` (linear) this
///   is `PAD + ascent - y_delta`; subtracting `y_delta` keeps the canvas-space
///   axis fixed at `anchorY` for every line.
///
/// Residual (unavoidable pre-rasterize): for the shaped/Arabic path the atlas
/// pen_x/baseline differ from the linear values used here (`PAD + min_x` /
/// `PAD - min_y` offsets are only known post-rasterize), so Arabic + gradient
/// text shifts by the run's left/descender ink (±3px / ±5px typically). With
/// letter spacing, TS itself anchors gradients by the no-spacing `measureText`
/// width while centering the drawn run by total width, so center/right aligns
/// drift by `spacing*(n-1)/2`; no single center can match all three — this is
/// correct for the common no-spacing case.
pub fn gradient_fill(block: &TextBlock, width: f32, ascent: f32, _y_delta: f32) -> Fill {
  let angle = block.gradient_angle * std::f32::consts::PI / 180.0;
  let (dx, dy) = (angle.cos(), angle.sin());
  let span = width.max(8.0);
  let cx = width / 2.0 + text::PAD;
  let cy = text::PAD + ascent;
  let s = Color::hex(&block.gradient_start);
  let e = Color::hex(&block.gradient_end);
  Fill::linear_gradient(
    cx - dx * span / 2.0,
    cy - dy * span / 2.0,
    cx + dx * span / 2.0,
    cy + dy * span / 2.0,
    &[(0.0, s), (1.0, e)],
  )
}

fn draw_line(
  c: &mut GpuCanvas,
  text: &str,
  anchor_x: f32,
  y: f32,
  align: TextAlign,
  family: &str,
  weight: f32,
  italic: bool,
  font_size: f32,
  fill: Fill,
  opacity: f32,
  char_index_start: usize,
  now: f32,
  bass: f32,
  block: &TextBlock,
) {
  let opts = TextOpts {
    letter_spacing: block.letter_spacing,
    outline: block.outline,
    outline_color: Color::hex(&block.outline_color),
    outline_width: block.outline_width,
    wave: block.wave_effect,
    wave_time: now,
    wave_amp: font_size * 0.12,
    char_index_start,
    bass,
  };

  // Shadow / glow pass (approximation: blurred duplicate copies).
  let glow = if block.shadow {
    // TS: `c.shadowBlur = block.shadowBlur + (block.glowIntensity || 0)` — a
    // negative glowIntensity is truthy, so it SUBTRACTS from the blur. When the
    // sum is <= 0 the canvas clamps shadowBlur to 0, i.e. a SHARP shadow at the
    // offset (the glow <= 0 branch below). Clamping the intensity (old
    // `.max(0.0)`) kept the full blur where TS shrank/zeroed it.
    block.shadow_blur + block.glow_intensity
  } else if block.glow_intensity > 0.0 {
    block.glow_intensity
  } else {
    0.0
  };
  let shadow_color = if block.use_gradient {
    Color::hex(&block.gradient_end)
  } else {
    Color::hex(&block.color)
  };
  if glow > 0.0 {
    let sh_x = if block.shadow { block.shadow_offset_x } else { 0.0 };
    let sh_y = if block.shadow { block.shadow_offset_y } else { 0.0 };
    // Canvas `shadowBlur = b` paints the run's alpha mask convolved with a
    // Gaussian of sigma = b/2 (verified against Skia: the step-edge erfc
    // falloff and thin-bar wash match sigma = b/2 exactly). Approximate that
    // convolution by rasterizing the glow-colored run ONCE (one atlas layer)
    // and drawing the same quad at N Gaussian-sampled offsets, each at
    // alpha = opacity/N. The copy-sum converges to the true blurred wash;
    // with N=64 the glow is smooth and costs just 64 extra quads (the old
    // per-call `draw_text` loop re-rasterized AND consumed one of the 20
    // atlas layers per copy, silently dropping the main fill on copy 21+).
    //
    // Offsets: radial inverse-CDF stratification of the Rayleigh radius with
    // a golden-angle spiral (deterministic, no RNG, covers out to ~3sigma).
    if let Some(font) = text::select_font_for_text_style(family, weight, italic, text) {
      let glow_fill = Fill::Solid(shadow_color);
      if let Some(atl) = text::rasterize(font, text, font_size, &glow_fill, &opts) {
        if let Some(layer) = c.upload_text_atlas(&atl) {
          // sigma = shadowBlur/2 (Skia's Gaussian). Each copy is the whole
          // run quad at a Gaussian-sampled offset with alpha = opacity/N, so
          // the copy-sum converges to the true 2D convolution of the run's
          // alpha mask with that Gaussian (inter-glyph wash ~0.4 at the gap
          // midpoint with sigma=10, matching Skia). N=256 keeps the estimate
          // smooth; the run is rasterized/uploaded only once, so the glow is
          // 256 extra quads — the offsets use inverse-CDF radii (Rayleigh)
          // with a golden-angle spiral (deterministic, covers out to ~3sigma).
          let sigma = (glow / 2.0).max(0.75);
          let n = 256u32;
          let per_copy = (opacity / n as f32).clamp(0.0, 1.0);
          let golden = 0.618_033_988_749_895f32;
          for k in 0..n {
            let u = (k as f32 + 0.5) / n as f32;
            let rho = sigma * (-2.0 * (1.0 - u).ln()).sqrt();
            let theta = TAU * k as f32 * golden;
            c.draw_text_quad(
              layer,
              &atl,
              anchor_x + sh_x + rho * theta.cos(),
              y + sh_y + rho * theta.sin(),
              align_of(align),
              per_copy,
            );
          }
        }
      }
    }
  } else if block.shadow && (block.shadow_offset_x != 0.0 || block.shadow_offset_y != 0.0) {
    // TS with shadowBlur = 0 (blur AND glow both zero — reachable from the UI
    // sliders) still paints a SHARP drop shadow at the offset: canvas
    // `shadowBlur = 0` means no blur, so the offset copy is drawn hard. The
    // 8-copy path above skips `glow == 0`, so emit one full-opacity copy here.
    c.draw_text(
      text,
      anchor_x + block.shadow_offset_x,
      y + block.shadow_offset_y,
      font_size,
      family,
      weight,
      italic,
      align_of(align),
      Fill::Solid(shadow_color),
      opacity,
      &opts,
    );
  }

  // Main fill pass.
  c.draw_text(text, anchor_x, y, font_size, family, weight, italic, align_of(align), fill, opacity, &opts);
}

/// Greedy word-wrap mirroring `textOverlay.ts` `wrapText`, but with the font
/// selected PER LINE (same `select_font_for_text(family, weight, line)` the
/// rasterizer uses in `draw_line`). Measuring with the paragraph-level font
/// would mis-size pure-Latin lines inside mixed Arabic-Latin paragraphs.
pub fn wrap_text(
  text: &str,
  max_width_px: f32,
  family: &str,
  weight: f32,
  italic: bool,
  font_size: f32,
  letter_spacing: f32,
) -> Vec<String> {
  let paragraphs = text.split('\n');
  let mut lines: Vec<String> = Vec::new();
  for paragraph in paragraphs {
    if paragraph.is_empty() {
      lines.push(String::new());
      continue;
    }
    if max_width_px <= 0.0 {
      lines.push(paragraph.to_string());
      continue;
    }
    let words: Vec<&str> = paragraph.split_whitespace().collect();
    let mut current_line = String::new();
    let mut wrapped = true;
    for word in words {
      let candidate = if current_line.is_empty() {
        word.to_string()
      } else {
        format!("{} {}", current_line, word)
      };
      // Font choice mirrors the rasterizer PER LINE (draw_line -> draw_text ->
      // select_font_for_text_style), so mixed Arabic-Latin paragraphs are
      // measured with exactly the same font that will draw the resulting line.
      let Some(font) = text::select_font_for_text_style(family, weight, italic, &candidate) else {
        // No usable system font at all: keep the paragraph as a single line.
        wrapped = false;
        break;
      };
      let width = text::measure(font, &candidate, font_size, letter_spacing);
      if current_line.is_empty() || width <= max_width_px {
        current_line = candidate;
      } else {
        lines.push(current_line);
        current_line = word.to_string();
      }
    }
    if wrapped {
      lines.push(current_line);
    } else {
      lines.push(paragraph.to_string());
    }
  }
  lines
}

fn draw_block(
  c: &mut GpuCanvas,
  width: f32,
  height: f32,
  block: &TextBlock,
  default_family: &str,
  now: f32,
  bass: f32,
  global_fade: f32,
) {
  // Mirrors textOverlay.ts drawBlock: `if (!block.text.trim() || block.opacity <= 0) return;`
  // An opacity of 0 hides the block entirely (slider min is 0) — never force 1.0.
  if block.text.trim().is_empty() || block.opacity <= 0.0 {
    return;
  }
  // TS: `c.globalAlpha = block.opacity * (block.fadeIn ? globalFade : 1)` — the
  // fade can reach exactly 0 at frame 0, so no floor here either.
  let opacity = block.opacity * (if block.fade_in { global_fade } else { 1.0 });
  if opacity <= 0.0 {
    return;
  }
  // TS passes block.fontSize straight into `c.font` — there is NO 48px
  // fallback. A 0px size draws no glyphs (verified in Chrome: 0 pixels), and a
  // negative size is an invalid font shorthand (browser keeps the previous
  // font). So the Rust port must not invent one either: a stale 48px fallback
  // made exports show text the preview never displayed. `react >= 0`, so
  // `font_size <= 0` iff `block.font_size <= 0` — skip the block entirely so
  // ALL rasterize paths (linear AND shaped/Arabic, which lacks an advance<=0
  // guard) draw nothing, matching the TS preview.
  let react = bass.clamp(0.0, 1.0) * block.reactive_scale;
  let font_size = block.font_size * (1.0 + react * 0.5);
  if font_size <= 0.0 {
    return;
  }
  let family = if block.font_family.trim().is_empty() {
    default_family
  } else {
    &block.font_family
  };
  let line_height = font_size * (if block.line_height <= 0.0 { 1.2 } else { block.line_height });
  let max_width_px = if block.max_width > 0.0 {
    (block.max_width / 100.0) * width
  } else {
    0.0
  };

  let text = apply_transform(&block.text, &block.transform);

  let (anchor_x, anchor_y) = block_anchor(block, width, height);

  let lines = wrap_text(&text, max_width_px, family, block.font_weight, block.italic, font_size, block.letter_spacing);

  let mut char_index = 0usize;
  for (i, line) in lines.iter().enumerate() {
    if line.is_empty() {
      char_index += 1;
      continue;
    }
    let y = anchor_y + i as f32 * line_height;
    let fill = if block.use_gradient {
      let Some(font) = text::select_font_for_text_style(family, block.font_weight, block.italic, line) else {
        char_index += line.chars().count();
        continue;
      };
      // TS measures the line WITHOUT letter spacing (measureText) for the
      // gradient geometry; spacing is added per-char during the draw.
      let width = text::measure(font, line, font_size, 0.0);
      let ascent = text::ascent(font, font_size);
      gradient_fill(block, width, ascent, i as f32 * line_height)
    } else {
      let col = if block.color.trim().is_empty() { "#ffffff" } else { &block.color };
      Fill::Solid(Color::hex(col))
    };
    draw_line(
      c, line, anchor_x, y, block.align, family, block.font_weight, block.italic, font_size, fill, opacity,
      char_index, now, bass, block,
    );
    char_index += line.chars().count();
  }
}

pub fn draw_text_overlay(c: &mut GpuCanvas, ctx: &RenderContext, global_fade: f32) {
  let txt = &ctx.config.text;
  let default_family = if txt.font_family.trim().is_empty() {
    "Outfit"
  } else {
    txt.font_family.as_str()
  };

  struct Item<'a> {
    block: &'a TextBlock,
    text: String,
  }
  let mut items: Vec<Item> = Vec::new();
  if txt.show_title {
    let t = if !txt.song_title.trim().is_empty() {
      txt.song_title.as_str()
    } else if !txt.title.text.trim().is_empty() {
      txt.title.text.as_str()
    } else {
      ""
    };
    if !t.trim().is_empty() {
      items.push(Item { block: &txt.title, text: t.to_string() });
    }
  }

  if txt.show_artist {
    let a = if !txt.artist_name.trim().is_empty() {
      txt.artist_name.as_str()
    } else if !txt.artist.text.trim().is_empty() {
      txt.artist.text.as_str()
    } else {
      ""
    };
    if !a.trim().is_empty() {
      items.push(Item { block: &txt.artist, text: a.to_string() });
    }
  }

  for b in &txt.blocks {
    if b.enabled && !b.text.trim().is_empty() {
      items.push(Item { block: b, text: b.text.clone() });
    }
  }
  if items.is_empty() {
    return;
  }

  // `global_fade` is computed by draw_frame via fade_factor (TS parity):
  // paused → 1.0 (fully visible), playing → ramp from the moment playback
  // started. In export the renderer is always "playing", so it ramps from 0.
  let now = ctx.frame_time;
  let bass = ctx.bass_energy;

  for item in &items {
    let mut block = item.block.clone();
    block.text = item.text.clone();
    draw_block(c, ctx.width, ctx.height, &block, default_family, now, bass, global_fade);
  }
}
