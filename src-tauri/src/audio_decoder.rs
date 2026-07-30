use std::fs::File;
use std::path::Path;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Clone)]
pub struct AudioData {
  pub samples: Vec<f32>,
  pub sample_rate: u32,
  pub channels: usize,
  pub duration_seconds: f64,
}

impl AudioData {
  pub fn decode_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.as_ref().extension().and_then(|s| s.to_str()) {
      hint.with_extension(ext);
    }

    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();
    let decoder_opts: DecoderOptions = Default::default();

    let probed = symphonia::default::get_probe()
      .format(&hint, mss, &format_opts, &metadata_opts)
      .map_err(|e| format!("Unsupported audio format: {}", e))?;

    let mut format = probed.format;

    let track = format
      .default_track()
      .ok_or_else(|| "No default audio track found".to_string())?;

    let sample_rate = track
      .codec_params
      .sample_rate
      .ok_or_else(|| "Unknown sample rate".to_string())?;

    let channels = track
      .codec_params
      .channels
      .map(|c| c.count())
      .unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
      .make(&track.codec_params, &decoder_opts)
      .map_err(|e| format!("Failed to create decoder: {}", e))?;

    let track_id = track.id;
    let mut samples: Vec<f32> = Vec::new();

    loop {
      let packet = match format.next_packet() {
        Ok(packet) => packet,
        Err(Error::IoError(_)) => break, // EOF
        Err(Error::ResetRequired) => break,
        Err(e) => return Err(format!("Error decoding packet: {}", e)),
      };

      if packet.track_id() != track_id {
        continue;
      }

      match decoder.decode(&packet) {
        Ok(audio_buf) => {
          append_audio_samples(&audio_buf, &mut samples, channels);
        }
        Err(Error::DecodeError(_)) => continue,
        Err(e) => return Err(format!("Decode error: {}", e)),
      }
    }

    let total_samples = samples.len();
    let duration_seconds = if sample_rate > 0 {
      total_samples as f64 / sample_rate as f64
    } else {
      0.0
    };

    Ok(Self {
      samples,
      sample_rate,
      channels,
      duration_seconds,
    })
  }

  pub fn get_sample_window(&self, time_sec: f64, window_size: usize) -> Vec<f32> {
    if self.samples.is_empty() || self.sample_rate == 0 {
      return vec![0.0; window_size];
    }

    let center_sample = (time_sec * self.sample_rate as f64) as usize;
    let start = center_sample.saturating_sub(window_size / 2);
    let end = (start + window_size).min(self.samples.len());

    let mut window = Vec::with_capacity(window_size);
    if start < self.samples.len() {
      window.extend_from_slice(&self.samples[start..end]);
    }

    while window.len() < window_size {
      window.push(0.0);
    }

    window
  }
}



fn append_audio_samples(buf: &AudioBufferRef, samples: &mut Vec<f32>, _channels: usize) {
  match buf {
    AudioBufferRef::F32(b) => {
      let num_frames = b.frames();
      let chan_count = b.spec().channels.count();
      for f in 0..num_frames {
        let mut sum = 0.0;
        for c in 0..chan_count {
          sum += b.chan(c)[f];
        }
        samples.push(sum / chan_count as f32);
      }
    }
    AudioBufferRef::U8(b) => {
      let num_frames = b.frames();
      let chan_count = b.spec().channels.count();
      for f in 0..num_frames {
        let mut sum = 0.0;
        for c in 0..chan_count {
          sum += (b.chan(c)[f] as f32 - 128.0) / 128.0;
        }
        samples.push(sum / chan_count as f32);
      }
    }
    AudioBufferRef::U16(b) => {
      let num_frames = b.frames();
      let chan_count = b.spec().channels.count();
      for f in 0..num_frames {
        let mut sum = 0.0;
        for c in 0..chan_count {
          sum += (b.chan(c)[f] as f32 - 32768.0) / 32768.0;
        }
        samples.push(sum / chan_count as f32);
      }
    }
    AudioBufferRef::S16(b) => {
      let num_frames = b.frames();
      let chan_count = b.spec().channels.count();
      for f in 0..num_frames {
        let mut sum = 0.0;
        for c in 0..chan_count {
          sum += b.chan(c)[f] as f32 / 32768.0;
        }
        samples.push(sum / chan_count as f32);
      }
    }
    AudioBufferRef::S32(b) => {
      let num_frames = b.frames();
      let chan_count = b.spec().channels.count();
      for f in 0..num_frames {
        let mut sum = 0.0;
        for c in 0..chan_count {
          sum += b.chan(c)[f] as f32 / 2147483648.0;
        }
        samples.push(sum / chan_count as f32);
      }
    }
    _ => {}
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_audio(samples: Vec<f32>, sample_rate: u32) -> AudioData {
    let duration_seconds = if sample_rate > 0 {
      samples.len() as f64 / sample_rate as f64
    } else {
      0.0
    };
    AudioData { samples, sample_rate, channels: 1, duration_seconds }
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
    // At start, center=0, start=0, so all 256 samples are real (no padding)
    assert!(window.iter().all(|&s| s == 1.0));
  }

  #[test]
  fn get_sample_window_near_end_pads_zeros() {
    let audio = make_audio(vec![0.5; 44100], 44100);
    let window = audio.get_sample_window(1.0, 256);
    assert_eq!(window.len(), 256);
    // Near the end, center=sample_rate, but end is capped at len.
    // start = 44100 - 128 = 43972, end = (43972+256).min(44100) = 44100
    // So 128 real samples, 128 zero-padded
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
}
