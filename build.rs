fn main() {
    // Tell MSVC linker to allocate 8 MB stack for the final binary on Windows
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-arg=/STACK:8388608");

    // Run Slint compilation in a dedicated thread with a 16 MB stack frame
    // to prevent STATUS_STACK_OVERFLOW (0xc00000fd) during AST compilation on Windows MSVC.
    let handle = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            slint_build::compile("ui/app_window.slint").expect("Slint build failed");
        })
        .expect("Failed to spawn Slint compiler thread");

    handle.join().expect("Slint compiler thread panicked");
}
