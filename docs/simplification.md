# Simplification work

This records the initial implementation. See [the subsequent review](review-fixes.md)
for additional fixes, current verification totals, and the latest package.

## Contract — November 6, 2026

Implement the four selected improvements for wowmc 0.1.0: independent cache
validation, shared library operations, actionable CLI guidance, and isolated,
reproducible verification. Local source and documentation changes are authorized;
publishing and edits to real game data are outside this work.

Baseline: 14 tests pass on Windows. A valid backup containing `wowmc-check` is
rejected by restore; changing only that name makes it pass. Empty `list` prints
nothing, and account setup says “Default character.” There is no Git metadata.

Acceptance: preserve unrelated cache bytes, preview defaults, backups, explicit
replacement, stale-writer rejection, existing permissive snapshot reads, JSON
output, revision history, and drafts on failed saves. Both interfaces must use
shared import and mutation rules. Tests must use explicit data and paths.

Verification uses temporary caches, CLI subprocesses, pure operation tests,
formatting, Clippy, and bounded TLA+ models with replayed regression paths.
Model artifacts are generated outside the repository, then retained here.
Work proceeds in bounded units with a failing/passing check for each behavior.
No external service or paid resource is required. Actual newcomer observation
and WoW reload/restart behavior require separate human observation; automated
walkthroughs do not establish those outcomes.

## Delivered behavior

- `snapshot.rs` parses records once and retains their byte ranges. Reads remain
  permissive about older identifiers; writes validate identifiers and scope.
  Restore validates the backup directly, independent of names and free IDs.
- `operations.rs` owns import decisions, edits, history, trash, and restoration.
  Both interfaces use it. Changes publish to memory after a successful save;
  even unchanged saves check for a stale writer. Failed GUI writes keep drafts.
- Running `wowmc` explains the starting workflow. Empty lists suggest a next
  command, while JSON remains machine-readable. Setup labels account targets
  correctly and suggests import or creation. `import` can use the selected
  target, `add NAME` opens an editor, unique ID prefixes work, and install prints
  an undo-preview command with the exact backup and target.
- GUI construction receives explicit paths and installation data. Tests use
  disposable directories and isolated environment variables. The model runner
  includes its runtime and downloads a checksum-pinned TLC jar independently
  of personal tools. `scripts/check.ps1` is the contributor entry point.

## Verification

`scripts/check.ps1` passes formatting, Clippy with warnings denied, and all 26
tests (4 library, 2 GUI, 20 integration). Tests include the complete CLI fixture
journey, a blocking test editor, ambiguous ID prefixes, preview and JSON output,
duplicate imports, failed-save retries, stale unchanged saves, undo, LF/CRLF
and unrelated-byte preservation, and read/write validation compatibility.

`scripts/check.ps1 -Models` also passed all six current models. The historical
restore model produces the expected one-step counterexample. See
[model checks](model-checks.md) for all charts, counts, invariants, omissions,
and reproduction commands. The restore regression was observed failing before
the fix and passing afterward.

The first model run downloaded and verified TLC successfully. Checks ran with
Node 24.15.0 and Java 22.0.2 on Windows. Rust CLI tests ran with and without the
GUI feature. Actual new-player usability, interactive Notepad behavior, GUI
visual inspection, partial-write fault injection, and WoW reload/restart
acceptance have not been verified by this work. Those remain separate checks;
model success does not establish them.

A fresh CLI package is available at
`dist/wowmc-windows-x64-simplified-20261106.zip`. Its extracted executable matches
the release build by SHA-256; version, welcome, and isolated empty JSON checks
passed against the extracted executable.
