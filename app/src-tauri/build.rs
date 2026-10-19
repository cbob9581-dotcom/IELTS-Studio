fn main() {
    #[cfg(feature = "desktop")]
    {
        let icon =
            std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("ielts-icon.ico");
        std::fs::copy("icons/icon.ico", &icon).expect("copy icon to compiler output");
        tauri_build::try_build(
            tauri_build::Attributes::new()
                .windows_attributes(tauri_build::WindowsAttributes::new().window_icon_path(icon)),
        )
        .expect("Tauri build configuration");
    }
}
