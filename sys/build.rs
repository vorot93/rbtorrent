use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    let libtorrent_path = manifest_dir.join("libtorrent");

    let dst = cmake::Config::new("libtorrent")
        .define("NDEBUG", "1")
        .define("CMAKE_CXX_STANDARD", "17")
        .define("TORRENT_USE_OPENSSL", "1")
        .define("TORRENT_USE_LIBCRYPTO", "1")
        .define("TORRENT_LINKING_SHARED", "1")
        .define("TORRENT_BUILDING_SHARED", "1")
        .build();

    println!("cargo:rustc-link-lib=torrent-rasterbar");
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build").display()
    );

    let mut cxx = cxx_build::bridge("src/lib.rs");
    let out_dir = env::var("OUT_DIR").unwrap();

    cxx.file("src/rb.cpp")
        .include(libtorrent_path.join("include"))
        .include(manifest_dir)
        .include(out_dir)
        .flag_if_supported("-std=c++17")
        .flag("-Wno-deprecated-declarations")
        .compile("rbtorrent-sys");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/rb.hpp");
    println!("cargo:rerun-if-changed=src/rb.cpp");
}
