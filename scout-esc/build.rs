use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=memory.x");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir);

    let memory_x = include_bytes!("memory.x").as_ref();
    File::create(out_dir.join("memory.x"))
        .unwrap()
        .write_all(memory_x)
        .unwrap();

    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
