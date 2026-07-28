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
}
