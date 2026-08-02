//! Rust port of `src/services/renderers/textOverlay.ts` (export path).

use std::f32::consts::TAU;

use crate::config::{TextAlign, TextBlock, TextTransform};
use crate::gpu2d::text::{self, TextOpts};
use crate::gpu2d::{Color, Fill, GpuCanvas};

use super::RenderContext;

const FADE_S: f32 = 0.8;

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

fn gradient_fill(block: &TextBlock, advance: f32, font_size: f32) -> Fill {
  let angle = block.gradient_angle * std::f32::consts::PI / 180.0;
  let (dx, dy) = (angle.cos(), angle.sin());
  let span = advance.max(8.0);
  let cx = 2.0 + advance / 2.0;
  let cy = font_size * 0.5;
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
    block.shadow_blur + (block.glow_intensity).max(0.0)
  } else if block.glow_intensity > 0.0 {
    block.glow_intensity
  } else {
    0.0
  };
  if glow > 0.0 {
    let sh_x = if block.shadow { block.shadow_offset_x } else { 0.0 };
    let sh_y = if block.shadow { block.shadow_offset_y } else { 0.0 };
    let shadow_color = if block.use_gradient {
      Color::hex(&block.gradient_end)
    } else {
      Color::hex(&block.color)
    };
    let glow_opacity = (opacity * 0.22).min(0.6);
    if block.shadow && (sh_x != 0.0 || sh_y != 0.0) {
      // Offset hard shadow reads as a real drop shadow.
      c.draw_text(
        text,
        anchor_x + sh_x,
        y + sh_y,
        font_size,
        family,
        weight,
        align_of(align),
        Fill::Solid(shadow_color),
        glow_opacity,
        &opts,
      );
    } else {
      // Radial-ish glow from 8 copies around the text.
      let r = glow.max(0.1);
      for i in 0..8u32 {
        let a = TAU * i as f32 / 8.0;
        c.draw_text(
          text,
          anchor_x + a.cos() * r,
          y + a.sin() * r,
          font_size,
          family,
          weight,
          align_of(align),
          Fill::Solid(shadow_color),
          glow_opacity,
          &opts,
        );
      }
    }
  }

  // Main fill pass.
  c.draw_text(text, anchor_x, y, font_size, family, weight, align_of(align), fill, opacity, &opts);
}

fn wrap_text(
  text: &str,
  max_width_px: f32,
  family: &str,
  weight: f32,
  font_size: f32,
  letter_spacing: f32,
) -> Vec<String> {
  let paragraphs = text.split('\n');
  let mut lines: Vec<String> = Vec::new();
  let font = text::select_font(family, weight);
  for paragraph in paragraphs {
    if paragraph.is_empty() {
      lines.push(String::new());
      continue;
    }
    if max_width_px <= 0.0 || font.is_none() {
      lines.push(paragraph.to_string());
      continue;
    }
    let font_ref = font.unwrap();
    let words: Vec<&str> = paragraph.split_whitespace().collect();
    let mut current_line = String::new();
    for word in words {
      let candidate = if current_line.is_empty() {
        word.to_string()
      } else {
        format!("{} {}", current_line, word)
      };
      let width = text::measure(font_ref, &candidate, font_size, letter_spacing);
      if current_line.is_empty() || width <= max_width_px {
        current_line = candidate;
      } else {
        lines.push(current_line);
        current_line = word.to_string();
      }
    }
    lines.push(current_line);
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
  if block.text.trim().is_empty() || block.opacity <= 0.0 {
    return;
  }
  let react = bass.clamp(0.0, 1.0) * block.reactive_scale;
  let font_size = block.font_size * (1.0 + react * 0.5);
  let family = if block.font_family.trim().is_empty() {
    default_family
  } else {
    &block.font_family
  };
  let line_height = font_size * block.line_height.max(0.1);
  let max_width_px = if block.max_width > 0.0 {
    (block.max_width / 100.0) * width
  } else {
    0.0
  };

  let text = apply_transform(&block.text, &block.transform);
  let opacity = block.opacity * (if block.fade_in { global_fade } else { 1.0 });
  if opacity <= 0.0 {
    return;
  }

  let anchor_x = (block.position_x / 100.0) * width;
  let anchor_y = (block.position_y / 100.0) * height;

  let lines = wrap_text(&text, max_width_px, family, block.font_weight, font_size, block.letter_spacing);

  let mut char_index = 0usize;
  for (i, line) in lines.iter().enumerate() {
    if line.is_empty() {
      char_index += 1;
      continue;
    }
    let y = anchor_y + i as f32 * line_height;
    let fill = if block.use_gradient {
      let Some(font) = text::select_font(family, block.font_weight) else {
        char_index += line.chars().count();
        continue;
      };
      let advance = text::measure(font, line, font_size, block.letter_spacing);
      gradient_fill(block, advance, font_size)
    } else {
      Fill::Solid(Color::hex(&block.color))
    };
    draw_line(
      c, line, anchor_x, y, block.align, family, block.font_weight, font_size, fill, opacity,
      char_index, now, bass, block,
    );
    char_index += line.chars().count();
  }
}

pub fn draw_text_overlay(c: &mut GpuCanvas, ctx: &RenderContext) {
  let txt = &ctx.config.text;
  let default_family = if txt.font_family.trim().is_empty() {
    "monospace"
  } else {
    txt.font_family.as_str()
  };

  struct Item<'a> {
    block: &'a TextBlock,
    text: String,
  }
  let mut items: Vec<Item> = Vec::new();
  if txt.show_title && !txt.song_title.trim().is_empty() {
    items.push(Item { block: &txt.title, text: txt.song_title.clone() });
  }
  if txt.show_artist && !txt.artist_name.trim().is_empty() {
    items.push(Item { block: &txt.artist, text: txt.artist_name.clone() });
  }
  for b in &txt.blocks {
    if b.enabled && !b.text.trim().is_empty() {
      items.push(Item { block: b, text: b.text.clone() });
    }
  }
  if items.is_empty() {
    return;
  }

  // In export mode the renderer is always "playing"; fade in from frame 0.
  let global_fade = (ctx.frame_time / FADE_S).clamp(0.0, 1.0);
  let now = ctx.frame_time;
  let bass = ctx.bass_energy;

  for item in &items {
    let mut block = item.block.clone();
    block.text = item.text.clone();
    draw_block(c, ctx.width, ctx.height, &block, default_family, now, bass, global_fade);
  }
}
