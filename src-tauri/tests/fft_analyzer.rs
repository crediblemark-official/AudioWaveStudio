use std::f32::consts::PI;
use audiowave_studio_lib::fft_analyzer::FftAnalyzer;

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
  assert!(bass < 0.5);
}

#[test]
fn compute_spectrum_bass_bins_high_for_low_freq() {
  let analyzer = FftAnalyzer::new(1024);
  let sample_rate = 44100.0;
  let n = 1024;
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
  assert!((analyzer.hann_window[128] - 1.0).abs() < 0.001);
  assert!((analyzer.hann_window[0] - 0.0).abs() < 0.001);
  assert!((analyzer.hann_window[255] - 0.0).abs() < 0.001);
}
