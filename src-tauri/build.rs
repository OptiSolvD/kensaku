fn main() {
    tauri_build::build();

    // Copy onnxruntime library from src-tauri to target directories
    // so tests and debug builds can find and load it without manual setup.
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap_or_default());

    #[cfg(windows)]
    let dll_name = "onnxruntime.dll";
    #[cfg(target_os = "linux")]
    let dll_name = "libonnxruntime.so";
    #[cfg(target_os = "macos")]
    let dll_name = "libonnxruntime.dylib";

    let src_dll = manifest_dir.join(dll_name);
    if src_dll.is_file()
        && let Some(target_dir) = out_dir.ancestors().nth(3)
    {
        let _ = std::fs::copy(&src_dll, target_dir.join(dll_name));
        let _ = std::fs::copy(&src_dll, target_dir.join("deps").join(dll_name));
    }
}
