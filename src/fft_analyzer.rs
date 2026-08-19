use realfft::RealFftPlanner;
use std::f32::consts::PI;
use std::sync::Mutex;

/// Attack/release smoother that turns raw FFT magnitudes into the exact u8
/// frequency data the live preview feeds to the renderers.
///
/// The preview and export paths MUST share this single implementation:
/// otherwise the exported video visibly diverges from what's on screen — the
/// export path previously skipped the per-bin sensitivity gain and used a
/// symmetric smoothing curve, so exported bars were dimmer and attacked/decayed
/// differently than the preview.
pub struct FreqSmoother {
  prev: Option<Vec<f32>>,
}

impl FreqSmoother {
  pub fn new() -> Self {
    Self { prev: None }
  }

  /// Smooth a fresh spectrum and quantize to u8, applying asymmetric
  /// attack/release smoothing. Sensitivity is NOT applied here — the
  /// renderers apply it directly on the frequency data so that sensitivity
  /// controls visual reactivity (not a double-squared size multiplier).
  ///
  /// - attack: `prev + (scaled - prev) * (alpha * 1.4)` with `alpha =
  ///   (1 - smoothing).clamp(0.08, 1.0)`
  /// - release: `prev * 0.90 + scaled * 0.10`
  ///
  /// The internal history starts at zero (mirrors the preview's initial
  /// `_prev_smoothed`) and resets automatically when the spectrum length
  /// changes (e.g. an `fft_size` switch).
  pub fn smooth_u8(&mut self, magnitudes: &[f32], _sensitivity: f32, smoothing: f32) -> Vec<u8> {
    let smooth_factor = smoothing.clamp(0.0, 0.95);
    let alpha = (1.0 - smooth_factor).clamp(0.08, 1.0);
    let attack = (alpha * 1.4).min(1.0);

    let prev = self.prev.get_or_insert_with(|| vec![0.0; magnitudes.len()]);
    if prev.len() != magnitudes.len() {
      *prev = vec![0.0; magnitudes.len()];
    }

    let mut out = Vec::with_capacity(magnitudes.len());
    for (i, &m) in magnitudes.iter().enumerate() {
      let scaled = m.clamp(0.0, 1.0);
      let p = &mut prev[i];
      *p = if scaled > *p {
        *p + (scaled - *p) * attack
      } else {
        (*p * 0.90 + scaled * 0.10).max(0.0)
      };
      out.push((*p * 255.0).min(255.0) as u8);
    }
    out
  }

  /// Drop the smoothing history so the next `smooth_u8` restarts from zero.
  pub fn reset(&mut self) {
    self.prev = None;
  }
}

impl Default for FreqSmoother {
  fn default() -> Self {
    Self::new()
  }
}

/// Quantize mono PCM samples to the u8 time-domain data used by waveform
/// styles. Uses the exact same scaling as the live preview path.
pub fn time_domain_u8(samples: &[f32]) -> Vec<u8> {
  samples
    .iter()
    .map(|s| ((s + 1.0) * 127.5).clamp(0.0, 255.0) as u8)
    .collect()
}

pub struct FftAnalyzer {
  planner: Mutex<RealFftPlanner<f32>>,
  pub fft_size: usize,
  pub hann_window: Vec<f32>,
}

impl FftAnalyzer {
  pub fn new(fft_size: usize) -> Self {
    let mut hann_window = Vec::with_capacity(fft_size);
    for i in 0..fft_size {
      hann_window.push(0.5 * (1.0 - (2.0 * PI * i as f32 / fft_size as f32).cos()));
    }
    Self {
      planner: Mutex::new(RealFftPlanner::new()),
      fft_size,
      hann_window,
    }
  }

  pub fn compute_spectrum(&self, samples: &[f32], target_bins: usize) -> Result<(Vec<f32>, f32), String> {
    let n = self.fft_size;
    let mut input = vec![0.0f32; n];

    for i in 0..n.min(samples.len()) {
      input[i] = samples[i] * self.hann_window[i];
    }

    let mut planner = self.planner.lock().map_err(|_| "Mutex poisoned".to_string())?;
    let fft = planner.plan_fft_forward(n);
    drop(planner);

    let mut output = fft.make_output_vec();
    if fft.process(&mut input, &mut output).is_err() {
      return Ok((vec![0.0; target_bins], 0.0));
    }

    let num_freq_bins = output.len();
    let magnitudes: Vec<f32> = output
      .iter()
      .map(|c| {
        let norm = c.norm() / (n as f32 / 2.0);
        let db = 20.0 * norm.max(1e-5).log10();
        ((db + 100.0) / 70.0).clamp(0.0, 1.0)
      })
      .collect();

    let bin_size = (num_freq_bins / target_bins).max(1);
    let mut binned = Vec::with_capacity(target_bins);

    for i in 0..target_bins {
      let start = i * bin_size;
      let end = (start + bin_size).min(num_freq_bins);
      if start < num_freq_bins {
        let sum: f32 = magnitudes[start..end].iter().sum();
        let count = (end - start).max(1) as f32;
        binned.push((sum / count).min(1.0));
      } else {
        binned.push(0.0);
      }
    }

    let bass_bins = 8.min(num_freq_bins);
    let bass_energy: f32 = if bass_bins > 0 {
      magnitudes[0..bass_bins].iter().sum::<f32>() / bass_bins as f32
    } else {
      0.0
    };

    Ok((binned, bass_energy))
  }

  pub fn compute_full_spectrum(&self, samples: &[f32]) -> Result<(Vec<f32>, f32), String> {
    let n = self.fft_size;
    let mut input = vec![0.0f32; n];

    for i in 0..n.min(samples.len()) {
      input[i] = samples[i] * self.hann_window[i];
    }

    let mut planner = self.planner.lock().map_err(|_| "Mutex poisoned".to_string())?;
    let fft = planner.plan_fft_forward(n);
    drop(planner);

    let mut output = fft.make_output_vec();
    if fft.process(&mut input, &mut output).is_err() {
      return Ok((vec![0.0; n / 2], 0.0));
    }

    // frequencyBinCount = fft_size / 2 (skip the Nyquist bin), matching AnalyserNode
    let magnitudes: Vec<f32> = output
      .iter()
      .take(n / 2)
      .map(|c| {
        let norm = c.norm() / (n as f32 / 2.0);
        let db = 20.0 * norm.max(1e-5).log10();
        ((db + 100.0) / 70.0).clamp(0.0, 1.0)
      })
      .collect();

    let bass_bins = 8.min(magnitudes.len());
    let bass_energy: f32 = if bass_bins > 0 {
      magnitudes[0..bass_bins].iter().sum::<f32>() / bass_bins as f32
    } else {
      0.0
    };

    Ok((magnitudes, bass_energy))
  }
}
