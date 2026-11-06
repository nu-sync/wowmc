# Contributing

wowmc is an experimental Windows CLI. Start with the [README](https://github.com/nu-sync/wowmc/blob/main/README.md) for supported behavior; `spec.md` also contains unimplemented proposals.

## Set up and check

Install a Windows x64 Rust toolchain, clone the repository, and run from its root:

```powershell
cargo build --locked --bin wowmc
cargo run -- --help
powershell -File scripts/check.ps1
```

The check script runs rustfmt, Clippy with warnings denied, and all-feature tests. Use `cargo fmt --all` to format changes and `cargo test --test install` for cache-specific integration tests.

For lifecycle or concurrency changes, also run:

```powershell
powershell -File scripts/check.ps1 -Models
```

This checks seven current TLA+ models. See [model instructions](https://github.com/nu-sync/wowmc/blob/main/statecharts/README.md) for Node and Java requirements, the pinned TLC download, and intentionally failing historical models.

## Keep development data separate

Tests use `tempfile` and helpers in `tests/common`. For manual checks, point `WOWMC_LIBRARY` and `WOWMC_CONFIG` at files in a disposable directory, and use copied game caches with `--target`. Do not test writes against your regular library or live game files.

Preserve preview defaults, explicit `--apply`, conflict detection, and backups. A successful local cache write is not proof of in-game acceptance. Never commit real account names, character data, caches, configuration, or build output.

## Make a focused change

Keep shared rules in library modules so the CLI and GUI agree. Follow rustfmt, `snake_case` functions and modules, `PascalCase` types, and contextual errors. Add behavior tests for relevant failure paths, including malformed input, conflicts, retries, and restoration.

Use concise imperative commit subjects, such as `Preserve cache records during replacement`. Pull requests should explain the problem and resulting behavior, link related issues, list checks and limitations, and include screenshots for GUI changes. Update documentation when commands or supported behavior change.
