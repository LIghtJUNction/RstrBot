fn main() {
    let attributes =
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[
            "start_local",
            "connect_remote",
            "disconnect",
        ]));
    if let Err(error) = tauri_build::try_build(attributes) {
        eprintln!("Desktop preparation failed: {error}");
        std::process::exit(1);
    }
}
