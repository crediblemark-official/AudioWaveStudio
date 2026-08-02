use std::fs::File;
use std::path::Path;
use symphonia::core::audio::{AudioBuffer, AudioBufferRef, Signal};
use symphonia::core::sample::Sample;
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



fn push_mixdown<T: Sample + Copy>(
  buf: &AudioBuffer<T>,
  samples: &mut Vec<f32>,
  convert: impl Fn(T) -> f32,
) {
  let num_frames = buf.frames();
  let chan_count = buf.spec().channels.count();
  for f in 0..num_frames {
    let mut sum = 0.0;
    for c in 0..chan_count {
      sum += convert(buf.chan(c)[f]);
    }
    samples.push(sum / chan_count as f32);
  }
}

fn append_audio_samples(buf: &AudioBufferRef, samples: &mut Vec<f32>, _channels: usize) {
  match buf {
    AudioBufferRef::F32(b) => push_mixdown(b, samples, |s| s),
    AudioBufferRef::F64(b) => push_mixdown(b, samples, |s| s as f32),
    AudioBufferRef::U8(b) => push_mixdown(b, samples, |s| (s as f32 - 128.0) / 128.0),
    AudioBufferRef::U16(b) => push_mixdown(b, samples, |s| (s as f32 - 32768.0) / 32768.0),
    AudioBufferRef::U24(b) => push_mixdown(b, samples, |s| (s.0 as f32 - 8388608.0) / 8388608.0),
    AudioBufferRef::U32(b) => {
      push_mixdown(b, samples, |s| (s as f32 - 2147483648.0) / 2147483648.0)
    }
    AudioBufferRef::S8(b) => push_mixdown(b, samples, |s| s as f32 / 128.0),
    AudioBufferRef::S16(b) => push_mixdown(b, samples, |s| s as f32 / 32768.0),
    AudioBufferRef::S24(b) => push_mixdown(b, samples, |s| s.0 as f32 / 8388608.0),
    AudioBufferRef::S32(b) => push_mixdown(b, samples, |s| s as f32 / 2147483648.0),
  }
}
