fn main() {
    // Expose the compile target so the installer knows which release archive to fetch
    let target = std::env::var("TARGET").expect("cargo sets TARGET");
    println!("cargo:rustc-env=TARGET={target}");
    println!("cargo:rerun-if-changed=build.rs");
}
