use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut, draw_line_segment_mut};
use imageproc::rect::Rect;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
  pub style: String,
  pub width: u32,
  pub height: u32,
  pub primary_color: [u8; 4],
  pub secondary_color: [u8; 4],
  pub accent_color: [u8; 4],
  pub bg_color: [u8; 4],
  pub bar_count: usize,
  pub sensitivity: f32,
  pub bass_multiplier: f32,
  pub show_particles: bool,
  pub title_text: Option<String>,
  pub artist_text: Option<String>,
}

pub struct RustRenderer {
  pub peak_data: Vec<f32>,
  pub bass_energy_smooth: f32,
  pub prev_spectrum: Vec<f32>,
}

impl RustRenderer {
  pub fn new() -> Self {
    Self {
      peak_data: Vec::new(),
      bass_energy_smooth: 0.0,
      prev_spectrum: Vec::new(),
    }
  }

  pub fn render_frame(
    &mut self,
    config: &RenderConfig,
    spectrum: &[f32],
    waveform: &[f32],
    bass_energy: f32,
  ) -> RgbaImage {
    let width = config.width;
    let height = config.height;
    let mut img = RgbaImage::from_pixel(width, height, Rgba(config.bg_color));

    let boosted = boost_spectrum(spectrum, config.sensitivity, config.bass_multiplier);

    let target_bass = (bass_energy * config.bass_multiplier).min(1.0);
    self.bass_energy_smooth += (target_bass - self.bass_energy_smooth) * 0.25;

    if config.show_particles {
      render_particles(&mut img, width, height, self.bass_energy_smooth, config.primary_color);
    }

    render_bass_aura(&mut img, width, height, self.bass_energy_smooth, config);

    match config.style.as_str() {
      "radial" => render_radial(&mut img, width, height, &boosted, self.bass_energy_smooth, config),
      "oscilloscope" => render_oscilloscope(&mut img, width, height, waveform, config),
      "equalizer" => render_equalizer(&mut img, width, height, &boosted, config),
      "minimal" => render_minimal(&mut img, width, height, &boosted, config),
      _ => render_spectrum_bars(&mut img, width, height, &boosted, &mut self.peak_data, config),
    }

    self.prev_spectrum = boosted;
    img
  }
}

fn boost_spectrum(spectrum: &[f32], sensitivity: f32, bass_multiplier: f32) -> Vec<f32> {
  let n = spectrum.len();
  let mut out = Vec::with_capacity(n);
  for i in 0..n {
    let raw = spectrum[i];
    let log_val = if raw > 0.0001 {
      let db = 20.0 * raw.log10();
      ((db + 100.0) / 70.0).max(0.0).min(1.0)
    } else {
      0.0
    };
    let bass_boost = if i < 8 {
      1.0 + bass_multiplier * 0.6
    } else if i < 24 {
      1.0 + bass_multiplier * 0.3
    } else {
      1.0
    };
    let boosted = (log_val * sensitivity * bass_boost * 2.5).min(1.0);
    out.push(boosted);
  }
  out
}

fn render_bass_aura(_img: &mut RgbaImage, _width: u32, _height: u32, _bass: f32, _config: &RenderConfig) {
}

fn render_particles(img: &mut RgbaImage, width: u32, height: u32, bass: f32, color: [u8; 4]) {
  let count = 60;
  for i in 0..count {
    let x = ((i * 97 + (bass * 50.0) as usize * (i + 1)) % width as usize) as i32;
    let y = ((i * 131 + (bass * 30.0) as usize * (i + 3)) % height as usize) as i32;
    let radius = (2.0 + bass * 5.0) as i32;
    let alpha = (80.0 + bass * 175.0).min(255.0) as u8;
    draw_filled_circle_mut(img, (x, y), radius, Rgba([color[0], color[1], color[2], alpha]));
  }
}

fn render_spectrum_bars(
  img: &mut RgbaImage,
  width: u32,
  height: u32,
  spectrum: &[f32],
  peak_data: &mut Vec<f32>,
  config: &RenderConfig,
) {
  let bar_count = config.bar_count.min(spectrum.len());
  if bar_count == 0 {
    return;
  }

  if peak_data.len() != bar_count {
    *peak_data = vec![0.0; bar_count];
  }

  let total_width = width as f32 * 0.85;
  let gap = 4.0;
  let bar_w = ((total_width - (gap * (bar_count - 1) as f32)) / bar_count as f32).max(2.0);
  let start_x = ((width as f32 - total_width) / 2.0) as i32;
  let center_y = (height as f32 * 0.55) as i32;
  let max_h = height as f32 * 0.45;

  for i in 0..bar_count {
    let val = spectrum[i].min(1.0);
    let bar_h = (val * max_h) as i32;

    if bar_h as f32 > peak_data[i] {
      peak_data[i] = bar_h as f32;
    } else {
      peak_data[i] = (peak_data[i] - 2.0).max(0.0);
    }

    let bx = start_x + (i as f32 * (bar_w + gap)) as i32;
    let by = center_y - bar_h;

    if bar_h > 0 {
      let rect = Rect::at(bx, by).of_size(bar_w as u32, bar_h as u32);
      let color = interpolate_color(config.secondary_color, config.primary_color, val);
      draw_filled_rect_mut(img, rect, Rgba(color));
    }

    if peak_data[i] > 4.0 {
      let peak_y = center_y - peak_data[i] as i32 - 4;
      let peak_rect = Rect::at(bx, peak_y).of_size(bar_w as u32, 3);
      draw_filled_rect_mut(img, peak_rect, Rgba(config.accent_color));
    }
  }
}

fn render_radial(
  img: &mut RgbaImage,
  width: u32,
  height: u32,
  spectrum: &[f32],
  bass: f32,
  config: &RenderConfig,
) {
  let cx = (width / 2) as f32;
  let cy = height as f32 * 0.48;
  let base_radius = (width.min(height) as f32 * 0.18) + bass * 30.0;
  let bar_count = config.bar_count.min(96).min(spectrum.len());

  draw_filled_circle_mut(
    img,
    (cx as i32, cy as i32),
    base_radius as i32,
    Rgba(config.primary_color),
  );

  let max_spike = width.min(height) as f32 * 0.3;

  for i in 0..bar_count {
    let val = spectrum[i].min(1.0);
    let spike_h = val * max_spike;

    let angle = (i as f32 / bar_count as f32) * PI * 2.0;
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let x1 = cx + cos_a * base_radius;
    let y1 = cy + sin_a * base_radius;
    let x2 = cx + cos_a * (base_radius + spike_h);
    let y2 = cy + sin_a * (base_radius + spike_h);

    let color = interpolate_color(config.primary_color, config.accent_color, val);
    draw_line_segment_mut(img, (x1, y1), (x2, y2), Rgba(color));
  }
}

fn render_oscilloscope(
  img: &mut RgbaImage,
  width: u32,
  height: u32,
  waveform: &[f32],
  config: &RenderConfig,
) {
  let center_y = height as f32 * 0.52;
  let len = waveform.len();
  if len < 2 {
    return;
  }

  let slice_w = width as f32 / (len - 1) as f32;
  let max_amp = height as f32 * 0.35 * config.sensitivity;

  for i in 0..(len - 1) {
    let x1 = i as f32 * slice_w;
    let y1 = center_y + waveform[i] * max_amp;
    let x2 = (i + 1) as f32 * slice_w;
    let y2 = center_y + waveform[i + 1] * max_amp;

    draw_line_segment_mut(img, (x1, y1), (x2, y2), Rgba(config.primary_color));
  }
}

fn render_equalizer(
  img: &mut RgbaImage,
  width: u32,
  height: u32,
  spectrum: &[f32],
  config: &RenderConfig,
) {
  let cols = config.bar_count.min(48).min(spectrum.len());
  let rows = 18;
  let avail_w = width as f32 * 0.8;
  let block_w = (avail_w / cols as f32 - 4.0).max(2.0);
  let block_h = ((height as f32 * 0.35) / rows as f32 - 3.0).max(2.0);
  let start_x = (width as f32 - avail_w) / 2.0;
  let start_y = height as f32 * 0.6;

  for c in 0..cols {
    let val = spectrum[c].min(1.0);
    let active_rows = (val * rows as f32) as usize;

    for r in 0..rows {
      let bx = (start_x + c as f32 * (block_w + 4.0)) as i32;
      let by = (start_y - r as f32 * (block_h + 3.0)) as i32;

      let color = if r < active_rows {
        if r > (rows as f32 * 0.8) as usize {
          config.accent_color
        } else if r > (rows as f32 * 0.5) as usize {
          config.primary_color
        } else {
          config.secondary_color
        }
      } else {
        [40, 40, 50, 50]
      };

      let rect = Rect::at(bx, by).of_size(block_w as u32, block_h as u32);
      draw_filled_rect_mut(img, rect, Rgba(color));
    }
  }
}

fn render_minimal(
  img: &mut RgbaImage,
  width: u32,
  height: u32,
  spectrum: &[f32],
  config: &RenderConfig,
) {
  let count = config.bar_count.min(64).min(spectrum.len());
  let avail_w = width as f32 * 0.7;
  let bar_w = (avail_w / count as f32 - 3.0).max(2.0);
  let start_x = (width as f32 - avail_w) / 2.0;
  let center_y = (height as f32 * 0.55) as i32;

  for i in 0..count {
    let val = spectrum[i].min(1.0);
    let bar_h = ((val * height as f32 * 0.35) as i32).max(4);

    let bx = (start_x + i as f32 * (bar_w + 3.0)) as i32;
    let by = center_y - bar_h / 2;

    let rect = Rect::at(bx, by).of_size(bar_w as u32, bar_h as u32);
    draw_filled_rect_mut(img, rect, Rgba(config.primary_color));
  }
}

fn interpolate_color(c1: [u8; 4], c2: [u8; 4], t: f32) -> [u8; 4] {
  [
    (c1[0] as f32 + (c2[0] as f32 - c1[0] as f32) * t) as u8,
    (c1[1] as f32 + (c2[1] as f32 - c1[1] as f32) * t) as u8,
    (c1[2] as f32 + (c2[2] as f32 - c1[2] as f32) * t) as u8,
    255,
  ]
}
