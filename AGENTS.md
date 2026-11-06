# Repository Guidelines

## Project Structure & Module Organization

wowmc is a Rust 2021 application for managing a local World of Warcraft macro library, primarily on Windows.

- `src/lib.rs`: library storage and installation discovery.
- `src/main.rs`: CLI commands; `src/gui.rs`: optional eframe/egui prototype behind the `gui` feature.
- `src/install.rs`: cache validation, installation previews, writes, backups, and restoration.
- `src/config.rs`: configuration; `src/operations.rs`: shared mutations; `src/snapshot.rs`: parsing.
- `tests/`: CLI integration tests; unit tests also live beside implementation code.
- `statecharts/`: TLA+ models and reports; `tools/statechart/`: vendored runtime.
- `scripts/package.ps1`: Windows packaging; `dist/`: package output; `target/`: build artifacts.

Consult `README.md` for supported behavior. `spec.md` includes proposals beyond the current implementation.

## Build, Test, and Development Commands

Install Rust and run from the repository root:

- `powershell -File scripts/check.ps1`: run formatting, linting, and tests; add `-Models` for lifecycle checks.
- `cargo build --locked --bin wowmc`: build the CLI using the lockfile.
- `cargo run -- --help`: inspect CLI commands.
- `cargo run --features gui --bin wowmc-gui`: launch the GUI prototype.
- `cargo test --all-features`: run unit and integration tests, including GUI tests.
- `cargo fmt --all -- --check`: check Rust formatting; use `cargo fmt --all` to format.
- `cargo clippy --all-features --all-targets -- -D warnings`: lint with warnings treated as errors.
- `powershell -File scripts/package.ps1`: build a CLI-only ZIP; requires Windows x64. Use `-Output` with a fresh path for subsequent packages.

## Coding Style & Naming Conventions

Use standard rustfmt formatting and four-space indentation. Use `snake_case` for modules, functions, and variables; `PascalCase` for types. Keep shared behavior in library modules so CLI and GUI use the same rules. Follow existing `anyhow::Result` and contextual error handling.

## Testing Guidelines

Use Rust's built-in `#[test]` framework and descriptive behavior names. Run `cargo test --test install` for cache tests. Use `tempfile` and `tests/common` for disposable, isolated fixtures. Cover previews, retries, conflicts, malformed input, and backup restoration when relevant. No numerical coverage threshold is configured.

## Commit & Pull Request Guidelines

Use concise imperative commit subjects, such as `Preserve cache records during replacement`. PRs should explain the problem and resulting behavior, link related issues, report checks and limitations, and include screenshots for GUI changes.

## Configuration & Data Safety

Use `WOWMC_LIBRARY`, `--library`, and `WOWMC_CONFIG` to isolate development data from `%APPDATA%\wowmc`. Test against copied caches. Preserve preview defaults, explicit `--apply`, conflict detection, and backups. A local cache write does not confirm in-game state.
