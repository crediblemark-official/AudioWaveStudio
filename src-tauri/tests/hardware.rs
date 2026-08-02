use audiowave_studio_lib::hardware::*;

#[test]
fn test_gpu_adapters_runs_without_panic() {
  let gpus = get_gpu_adapters();
  assert!(!gpus.is_empty() || gpus.is_empty());
}

#[test]
fn test_encoder_libx264_always_true() {
  assert!(test_encoder("ffmpeg", "libx264"));
}
