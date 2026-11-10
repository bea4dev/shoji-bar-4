//! Picks the interface the binary is built with, from the plugins directory.
//!
//! The plugins directory is `plugins/` at the root of the project (next to the top-level
//! `Cargo.toml`), or `$SHOJI_PLUGINS_DIR` when that is set. In it:
//!
//! - `init.rs` with `pub fn init()` replaces the stock interface: `init` runs as the root of the
//!   shell and puts on the screen whatever it wants.
//! - `init-default.rs` with `pub fn patch(ui: &mut shoji_default::Ui)` keeps the stock interface
//!   and changes it before it is put on the screen.
//! - With neither, the stock interface is used. With both, `init.rs` wins.
//!
//! The chosen file is compiled into this binary as a module (`#[path]`), so the rest of the
//! directory (`mod helpers;` next to `init.rs`) works as in any crate. Editing a file there, or
//! adding or removing one, makes `cargo build` rebuild only this small crate.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn plugins_dir() -> PathBuf {
    if let Some(dir) = env::var_os("SHOJI_PLUGINS_DIR") {
        return PathBuf::from(dir);
    }
    // This crate is at <root>/crates/shoji-bar-4.
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .expect("the crate is two levels below the project root")
        .join("plugins")
}

fn module(name: &str, file: &Path) -> String {
    // `{:?}` quotes and escapes the path as a Rust string literal.
    format!("#[path = {:?}]\nmod {name};\n", file.display().to_string())
}

fn main() {
    println!("cargo:rerun-if-env-changed=SHOJI_PLUGINS_DIR");
    println!("cargo:rerun-if-changed=build.rs");

    let dir = plugins_dir();
    // Cargo reruns a build script whose watched path is missing on every build, so the directory
    // is created: an empty one means the stock interface.
    if let Err(e) = fs::create_dir_all(&dir) {
        println!("cargo:warning=cannot create {}: {e}", dir.display());
    }
    println!("cargo:rerun-if-changed={}", dir.display());

    let init = dir.join("init.rs");
    let init_default = dir.join("init-default.rs");
    let (source, code) = if init.is_file() {
        if init_default.is_file() {
            println!("cargo:warning=init.rs and init-default.rs both exist: using init.rs");
        }
        (
            init.display().to_string(),
            format!(
                "{}\npub fn run() {{\n    shoji_default::run_custom(init::init);\n}}\n",
                module("init", &init)
            ),
        )
    } else if init_default.is_file() {
        (
            init_default.display().to_string(),
            format!(
                "{}\npub fn run() {{\n    shoji_default::run_with(init_default::patch);\n}}\n",
                module("init_default", &init_default)
            ),
        )
    } else {
        (
            String::new(),
            "pub fn run() {\n    shoji_default::run();\n}\n".to_owned(),
        )
    };

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("plugin_entry.rs");
    let code = format!("pub const SOURCE: &str = {source:?};\n{code}");
    fs::write(out, code).expect("cannot write plugin_entry.rs");
}
