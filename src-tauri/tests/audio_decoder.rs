use audiowave_studio_lib::audio_decoder::AudioData;

fn make_audio(samples: Vec<f32>, sample_rate: u32) -> AudioData {
  let duration_seconds = if sample_rate > 0 {
    samples.len() as f64 / sample_rate as f64
  } else {
    0.0
  };
  AudioData {
    samples,
    sample_rate,
    channels: 1,
    duration_seconds,
  }
}

#[test]
fn get_sample_window_returns_full_window_in_middle() {
  let audio = make_audio(vec![0.5; 44100], 44100);
  let window = audio.get_sample_window(0.5, 1024);
  assert_eq!(window.len(), 1024);
  assert!(window.iter().all(|&s| (s - 0.5).abs() < 1e-6));
}

#[test]
fn get_sample_window_at_start_no_zero_padding() {
  let audio = make_audio(vec![1.0; 44100], 44100);
  let window = audio.get_sample_window(0.0, 256);
  assert_eq!(window.len(), 256);
  assert!(window.iter().all(|&s| s == 1.0));
}

#[test]
fn get_sample_window_near_end_pads_zeros() {
  let audio = make_audio(vec![0.5; 44100], 44100);
  let window = audio.get_sample_window(1.0, 256);
  assert_eq!(window.len(), 256);
  let zeros = window.iter().filter(|&&s| s == 0.0).count();
  assert!(zeros > 0, "Expected some zero-padded samples at end of audio");
}

#[test]
fn get_sample_window_beyond_duration_returns_zeros() {
  let audio = make_audio(vec![0.5; 44100], 44100);
  let window = audio.get_sample_window(10.0, 512);
  assert_eq!(window.len(), 512);
  assert!(window.iter().all(|&s| s == 0.0));
}

#[test]
fn get_sample_window_empty_audio() {
  let audio = make_audio(vec![], 44100);
  let window = audio.get_sample_window(0.5, 256);
  assert_eq!(window.len(), 256);
  assert!(window.iter().all(|&s| s == 0.0));
}

#[test]
fn get_sample_window_respects_window_size() {
  let audio = make_audio(vec![1.0; 44100 * 2], 44100);
  let window = audio.get_sample_window(1.0, 2048);
  assert_eq!(window.len(), 2048);
}

#[test]
fn get_sample_window_zero_sample_rate() {
  let audio = make_audio(vec![0.5; 100], 0);
  let window = audio.get_sample_window(0.5, 256);
  assert_eq!(window.len(), 256);
  assert!(window.iter().all(|&s| s == 0.0));
}
