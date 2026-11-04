# SimplyPDF progress

## 2026-11-04 · Stage 0, repository and build baseline

- Added the staged README checklist and repository policy. Do not commit or push Bluebeam code or any proprietary source code or files to GitHub. The local installer and references directory are ignored.
- Adopted Rust for all first-party code and updated the implementation brief. Added a dependency-free Cargo workspace with a headless core library crate, Rust 2024 edition, and a declared minimum of Rust 1.85. Desktop UI and PDF adapter choices remain open for the foundation spike.
- Ran `cargo build --workspace --locked --offline` and `cargo test --workspace --locked --offline` on macOS 27.0.1 arm64 with Rust 1.98.1. Both commands succeeded. The scaffold has zero tests. Windows, Linux, and Rust 1.85 verification are pending.
- Attempted `cargo fmt --all -- --check`. The installed toolchain lacks rustfmt.
- Verified ignore rules for the local Revu installer, vendor reference directory, and build output with `git check-ignore`.

Next slice is a deterministic fixture generator in Rust, a fixture manifest with independently specified lengths and transforms, and the first headless geometry tests.
