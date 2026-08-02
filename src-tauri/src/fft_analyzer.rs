use realfft::RealFftPlanner;
use std::f32::consts::PI;
use std::sync::Mutex;

pub struct FftAnalyzer {
  planner: Mutex<RealFftPlanner<f32>>,
  fft_size: usize,
  hann_window: Vec<f32>,
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
      .map(|c| (c.norm() / (n as f32 / 2.0)).min(1.0))
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
      .map(|c| (c.norm() / (n as f32 / 2.0)).min(1.0))
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn compute_spectrum_silence_returns_zero() {
    let analyzer = FftAnalyzer::new(512);
    let samples = vec![0.0; 512];
    let (spectrum, bass) = analyzer.compute_spectrum(&samples, 16).unwrap();
    assert_eq!(spectrum.len(), 16);
    assert!(spectrum.iter().all(|&v| v == 0.0));
    assert_eq!(bass, 0.0);
  }

  #[test]
  fn compute_spectrum_returns_requested_bins() {
    let analyzer = FftAnalyzer::new(1024);
    let samples = vec![0.5; 1024];
    for bins in [8, 16, 32, 64] {
      let (spectrum, _) = analyzer.compute_spectrum(&samples, bins).unwrap();
      assert_eq!(spectrum.len(), bins, "Expected {} bins", bins);
    }
  }

  #[test]
  fn compute_spectrum_sine_wave_peak() {
    let analyzer = FftAnalyzer::new(1024);
    let sample_rate = 44100.0;
    let freq = 440.0;
    let n = 1024;
    let samples: Vec<f32> = (0..n)
      .map(|i| (2.0 * PI * freq * i as f32 / sample_rate).sin())
      .collect();
    let (spectrum, bass) = analyzer.compute_spectrum(&samples, 32).unwrap();
    assert_eq!(spectrum.len(), 32);
    // Bass should be low since 440Hz is not bass frequency
    assert!(bass < 0.5);
  }

  #[test]
  fn compute_spectrum_bass_bins_high_for_low_freq() {
    let analyzer = FftAnalyzer::new(1024);
    let sample_rate = 44100.0;
    let n = 1024;
    // 80Hz sine wave (bass range)
    let samples: Vec<f32> = (0..n)
      .map(|i| (2.0 * PI * 80.0 * i as f32 / sample_rate).sin())
      .collect();
    let (_, bass) = analyzer.compute_spectrum(&samples, 32).unwrap();
    assert!(bass > 0.01, "Bass energy should be > 0 for 80Hz tone");
  }

  #[test]
  fn compute_spectrum_shorter_samples_than_fft_size() {
    let analyzer = FftAnalyzer::new(1024);
    let samples = vec![1.0; 100];
    let (spectrum, _) = analyzer.compute_spectrum(&samples, 16).unwrap();
    assert_eq!(spectrum.len(), 16);
  }

  #[test]
  fn fft_analyzer_new_creates_hann_window() {
    let analyzer = FftAnalyzer::new(256);
    assert_eq!(analyzer.fft_size, 256);
    assert_eq!(analyzer.hann_window.len(), 256);
    // Hann window at center should be ~1.0
    assert!((analyzer.hann_window[128] - 1.0).abs() < 0.001);
    // Hann window at edges should be ~0.0
    assert!((analyzer.hann_window[0] - 0.0).abs() < 0.001);
    assert!((analyzer.hann_window[255] - 0.0).abs() < 0.001);
  }
}
