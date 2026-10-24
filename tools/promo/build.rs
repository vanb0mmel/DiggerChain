fn main() {
    println!("cargo:rerun-if-env-changed=DIGGERCHAIN_PROMO_X264");
    if let Ok(directory) = std::env::var("DIGGERCHAIN_PROMO_X264") {
        println!("cargo:rustc-link-search=native={directory}");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{directory}");
    }
}
