use std::env;
use std::path::PathBuf;

// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // bosch bsec2 lib
    println!("cargo:rerun-if-changed=vendor/bindings.h");

    // link libalgobsec.a
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    println!(
        "cargo:rustc-link-search={}/vendor/bosch/bsec2/lib",
        manifest_dir
    );
    println!("cargo:rustc-link-lib=algobsec");

    // generate bindings
    let bindings = bindgen::Builder::default()
        .header("vendor/bindings.h")
        //.ctypes_prefix("cty")
        .use_core()
        .clang_arg("--target=riscv32-unknown-none-elf")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")?;
    Ok(())
}
