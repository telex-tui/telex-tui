//! Verifies that all numbered examples compile.
//!
//! The examples in crates/telex/examples/ are the runnable code that the
//! mdbook references. If the API changes, this test catches it.
//!
//! Run with:  cargo test -p telex-tui --test book_compiles

use std::process::Command;

#[test]
fn all_examples_compile() {
    let output = Command::new("cargo")
        .args(["build", "-p", "telex-tui", "--examples", "--message-format=short"])
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .expect("Failed to run cargo build");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "Examples failed to compile:\n\n{}",
            stderr
                .lines()
                .filter(|l| l.contains("error"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    // Count how many examples were built
    let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let count = std::fs::read_dir(&examples_dir)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .map(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                .unwrap_or(false)
        })
        .count();

    eprintln!("OK: all {} examples compile", count);
}
