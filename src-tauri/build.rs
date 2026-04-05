fn main() {
    #[cfg(target_os = "ios")]
    println!("cargo:rustc-link-lib=framework=CoreBluetooth");

    tauri_build::build()
}
