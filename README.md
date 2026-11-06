# wowmc

Keep your World of Warcraft macros in a local library: find them, edit them outside the game, recover earlier versions, and copy them back when needed. Your library stays separate from the game's cache files.

**Experimental Windows CLI, initially focused on WoW Forever.** Library management, revision history, and copy/export work without a game installation. Direct cache writes have limited live validation; they do not confirm what the game has loaded.

## Get started

Until a first release is available on the [Releases page](https://github.com/nu-sync/wowmc/releases), build from source with a Windows x64 Rust toolchain:

```powershell
cargo build --locked --release --bin wowmc
.\target\release\wowmc.exe --help
```

The examples below run from the repository root using `cargo run --`. With an extracted CLI package, replace that prefix with `.\wowmc.exe`; packaged executables do not require Rust.

Create a macro and copy its text into the game's macro editor:

```powershell
'/say Ready' | cargo run -- add "Ready" --file -
cargo run -- list
cargo run -- show "Ready"
cargo run -- copy "Ready"
cargo run -- export "Ready"
```

`copy` uses the Windows clipboard; `export` prints the body for scripts or manual copying. `add NAME` without `--file` and `edit NAME` open an editor. Save and close it to finish. Interactive Windows sessions default to Notepad; `VISUAL` or `EDITOR` can specify a blocking editor executable, without arguments.

## Organize and recover

| Task | Command |
| --- | --- |
| Search macros | `cargo run -- list --search TEXT` |
| Inspect earlier versions | `cargo run -- history "Ready"` |
| Restore version 1 | `cargo run -- restore "Ready" --revision 1` |
| Move to recoverable trash | `cargo run -- delete "Ready"` |
| Restore a deleted macro | `cargo run -- restore "Ready"` |
| Include deleted macros | `cargo run -- list --all` |
| Get structured output | `cargo run -- list --json` |

Revision numbers start at 1, oldest first. For duplicate names, use the ID shown by `list`, or a unique ID prefix of at least eight characters.

The library defaults to `%APPDATA%\wowmc\library.json`; override it with `--library PATH` or `WOWMC_LIBRARY`. Saves retain the previous file as `<library filename>.bak` and detect conflicting changes.

## Import existing macros

```powershell
cargo run -- setup
cargo run -- import          # preview the selected cache
cargo run -- import --apply  # save new names to the library
```

`setup` discovers Forever installations and selects a character automatically when only one is found, or prompts when there are several. Use `setup --path <edition-folder>` if discovery misses your installation, or `setup --target <path-to-macros-cache.txt>` for an explicit account or character cache. Configuration defaults to `%APPDATA%\wowmc\config.json`; `WOWMC_CONFIG` overrides it.

`import PATH` also accepts a copied cache, and `import -` reads standard input. Previews distinguish new entries, existing entries, and name conflicts; applying imports only new names. Scope is inferred from game paths, otherwise account; `--scope` overrides it.

## Preview a cache write

```powershell
cargo run -- install "Ready"          # preview
cargo run -- install "Ready" --apply  # write after review
cargo run -- compare "Ready"
```

`install`, `compare`, and `restore-cache` accept `--target PATH` to override setup. The account or character directory must already exist; the cache itself may be missing. An identical macro needs no write. A different same-named macro requires `--replace` in the preview and apply commands.

Existing cache files are backed up beside the target, and other records are preserved. To undo, run `restore-cache --backup "<printed backup path>"` to preview, then repeat with `--apply`. Restoring also backs up the current cache. A newly created cache has no earlier file to restore. Action bars are outside this tool's scope.

One append to a character cache on Forever build 1.60.1.70790 was reported working after `/reload` on November 5, 2026. Restart persistence, account-wide writes, new caches, replacement, and other clients remain unverified in game. Avoid editing macros in game during a write; the game can overwrite cache files afterward. Reload and inspect the result in game. `compare` checks only the local file.

## Development

Run these commands from the repository root:

```powershell
powershell -File scripts/check.ps1          # formatting, linting, all-feature tests
powershell -File scripts/check.ps1 -Models  # also check seven lifecycle models
powershell -File scripts/package.ps1       # Windows x64 CLI ZIP in dist
cargo run --features gui --bin wowmc-gui    # source-only GUI prototype
```

Packaging requires Windows x64 MSVC and a fresh output path; use `-Output` for subsequent packages. The ZIP includes license notices and has a SHA-256 sidecar. Model checks need Node 22.18–22.x or 23.6+ and Java 11+, and download checksum-pinned TLC 1.7.4 on first use. Tests use disposable fixtures without a real game installation.

See [contributing](https://github.com/nu-sync/wowmc/blob/main/CONTRIBUTING.md), [model checks](https://github.com/nu-sync/wowmc/blob/main/statecharts/README.md), and [review evidence](https://github.com/nu-sync/wowmc/blob/main/docs/review-fixes.md). The GUI is a prototype and is not included in the CLI ZIP.

## License

wowmc is available under the [MIT license](https://github.com/nu-sync/wowmc/blob/main/LICENSE). The vendored model-checking runtime retains its [license and attribution](https://github.com/nu-sync/wowmc/blob/main/tools/statechart/README.md).
