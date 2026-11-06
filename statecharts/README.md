# State models

These finite models describe selected observable transitions, not every detail of the implementation. `SaveBefore` and `WritersBefore` capture earlier designs and their counterexamples. The corresponding `After` models describe the intended fixes. `ImportLifecycle` covers a single logical name, a preview, a successful or failed library write, and a retry. `RecordLifecycle` abstracts one library record and one history flag. Generated reports and charts are in the neighboring folders.

The writer model assumes cooperating updated processes, the same library and lock path, working operating-system locks, and atomic file replacement. Reload and reapply are manual. External programs and older wowmc versions do not honor the lock. It does not model crashes, unusual filesystem behavior, or application state beyond the modeled transitions. The import model assumes the snapshot is parsed and uses a fixed set of duplicate names; it does not prove the parser or game synchronization. The record model abstracts actual revision contents and does not establish liveness. The editor model does not cover clipboard, rendering, or in-game behavior. No model proves implementation conformance; regression tests and interface checks provide separate evidence.

## Run checks

Install Node 22.18–22.x or 23.6+ and Java 11+ (tested here with Node 24.15.0
and Java 22.0.2). No personal Codex skill, mise installation, or npm install is
needed. The runtime is vendored with its license in `tools/statechart/`.

```powershell
.\statecharts\run.ps1 -Model RestoreAfter
.\statecharts\run.ps1 -Model ImportLifecycle -Out "$env:TEMP\wowmc-import-new"
.\statecharts\run.ps1 -Model RestoreBefore -ExpectViolation
.\scripts\check.ps1 -Models
```

The first run downloads TLC 1.7.4 to `target/tools/`; its SHA-256 must match
`936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88`.
For offline use, set `TLA2TOOLS` to an existing jar with that hash; set `JAVA`
to override the Java executable. Each model runs in a temporary folder outside
the repository. Output defaults to a fresh temporary folder; `-Out` must name a
new directory. Unexpected violations or runner failures fail the command.
Raw `tlc*.out` logs are ignored because they contain local temporary paths;
keep only reviewed, sanitized reports and diagrams in Git.

Current models: `SaveAfter`, `WritersAfter`, `ImportLifecycle`,
`RecordLifecycle`, `CacheWrites`, `RestoreAfter`, and `LibrarySaveAfter`. Historical
counterexamples: `SaveBefore`, `WritersBefore`, `RestoreBefore`, and `LibrarySaveBefore`;
use `-ExpectViolation` to check that a counterexample remains detectable.

`ImportLifecycle` checks retained previews after failed saves, no duplicate
logical record on retry, no game writes, and publishing memory only after a
successful save. `CacheWrites` checks bounded existing-cache writes and backups;
it omits creation, partial writes, crash windows, rollback failures, and concurrent
game writes. `RestoreBefore` and `RestoreAfter` use the same model and invariants
with different validation rules. They cover supported/malformed backups, name
collisions, previews, failures before writing, retries, and undo backups.
They exclude partial writes, crashes, external writers, and game load behavior.

`LibrarySaveBefore` and `LibrarySaveAfter` check unchanged saves, useful recovery
backups, failed attempts, and external changes before a save. The old behavior
violates `UnchangedKeepsRecovery`; the fix preserves the backup after checking
for conflicts. See [review fixes and charts](../docs/review-fixes.md) for the real
reproduction, bounded model results, and scope limitations.

See [model checks and charts](../docs/model-checks.md) for results, the replayed
restore bug, and each chart's Mermaid source.
