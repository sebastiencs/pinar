use std::env;

fn main() {
    let is_nightly = env::var("RUSTUP_TOOLCHAIN")
        .map(|toolchain| {
            toolchain.to_lowercase().contains("nightly")
        })
        .unwrap_or(false);

    if is_nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}
