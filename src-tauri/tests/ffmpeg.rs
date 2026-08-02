use audiowave_studio_lib::ffmpeg::*;

#[test]
fn download_url_returns_non_empty() {
  let url = download_url();
  assert!(!url.is_empty());
  assert!(url.starts_with("http"));
}

#[test]
fn is_in_path_returns_true_for_existing_commands() {
  assert!(is_in_path("echo"));
}

#[test]
fn is_in_path_returns_false_for_nonexistent_commands() {
  assert!(!is_in_path("this_command_does_not_exist_xyz123"));
}

#[test]
fn find_ffmpeg_binary_finds_nested_exe() {
  let tmp = std::env::temp_dir().join("audiowave_ffmpeg_test");
  let _ = std::fs::remove_dir_all(&tmp);
  let bin = tmp.join("bin");
  std::fs::create_dir_all(&bin).unwrap();
  let name = if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" };
  std::fs::write(bin.join(name), b"dummy").unwrap();
  let found = find_ffmpeg_binary(&tmp);
  let _ = std::fs::remove_dir_all(&tmp);
  assert!(found.is_some());
  assert!(found.unwrap().file_name().unwrap().to_string_lossy().eq_ignore_ascii_case(name));
}

#[test]
fn auto_install_supported_consistent() {
  if cfg!(target_os = "linux") {
    assert!(!auto_install_supported());
  }
}
