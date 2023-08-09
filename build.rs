//! This build script compiles the user interface from `.slint` files into native rust code.

use std::fs;

fn main() {
    slint_build::compile("src/ui/main.slint").unwrap();

    // By default, Cargo will re-run a build script whenever any file in the project changes.
    // Adding all files in `src/ui/` here to ensure it is only re-run when these are changed.
    for entry in fs::read_dir("src/ui").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        println!(
            "cargo:rerun-if-changed={:?}",
            path.into_os_string().into_string()
        );
    }
}
