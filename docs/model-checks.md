# Model checks — November 6, 2026

Checked with TLC 1.7.4 (pinned checksum), Java 22.0.2, Node 24.15.0 on Windows.
Counts describe the finite abstractions, not application coverage or reliability.

The restore counterexample was replayed against the real CLI and captured in
`restore_accepts_valid_backups_independent_of_macro_names`: the test failed
before the change and passes afterward. Rule `ValidBackupAccepted` means
“a supported backup must not be rejected because of a macro's name.”
Shortest path: **0. Backup with wowmc-check selected → 1. preview backup:
Valid backup rejected**.

The restore model covers validation, preview, a failure before writing, retry,
and a completed restore with an undo backup. It excludes partial writes, crashes,
rollback failure, external writers, and in-game behavior. Import models one
logical name and atomic library saves; it does not prove parsing, conflict
resolution for arbitrary datasets, or filesystem behavior. Application tests
provide separate evidence for those implemented cases.

Reproduce with new output directories (automatically selected when `-Out` is omitted):

```powershell
.\statecharts\run.ps1 -Model RestoreBefore -ExpectViolation
.\statecharts\run.ps1 -Model RestoreAfter
.\statecharts\run.ps1 -Model ImportLifecycle
```

The remaining models were rechecked unchanged; their scope limitations are in
`statecharts/README.md`. Charts omit transitions that stay in the same displayed
state, including terminal “finished” steps. No additional transitions are hidden.


## restore-before

TLC reached 10 states and 11 steps. The chart draws 8 states and 6 arrows.
The chart does not draw 4 steps that stay in one chart state.

## ValidBackupAccepted breaks
The shortest path has 1 step:
0. Backup with wowmc-check selected
1. preview backup: Valid backup rejected

## Rules that hold
MalformedNeverWritten, PreviewDoesNotWrite, BackupBeforeRestore, NoDeadlock

```mermaid
---
title: "Restoring a cache backup"
---
stateDiagram-v2
  state "Legend" as legend {
    lg0: Breaks ValidBackupAccepted
    lg1: Path to the ValidBackupAccepted bug
  }
  class lg0 breaks
  class lg1 bugPath
  s0: Malformed backup selected
  s1: Other valid backup selected
  s2: Backup with wowmc-check selected
  s3: Ready to restore
  s4: Malformed backup rejected
  s5: Valid backup rejected
  s6: Restore failed#59; original kept
  s7: Restored#59; undo backup saved
  [*] --> s0
  [*] --> s1
  [*] --> s2
  s0 --> s4: preview backup
  s1 --> s3: preview backup
  s2 --> s5: preview backup
  s3 --> s6: write cannot begin
  s3 --> s7: apply restore
  s6 --> s3: retry
  classDef breaks fill:#ffecee,stroke:#e72e3b,stroke-width:2.5px,color:#cd2834
  class s2 bugPath
  class s5 breaks
  class s5 bugPath
```

## restore-after

TLC reached 12 states and 14 steps. The chart draws 7 states and 6 arrows.
The chart does not draw 4 steps that stay in one chart state.

## Rules that hold
ValidBackupAccepted, MalformedNeverWritten, PreviewDoesNotWrite, BackupBeforeRestore, NoDeadlock

```mermaid
---
title: "Restoring a cache backup"
---
stateDiagram-v2
  s0: Malformed backup selected
  s1: Other valid backup selected
  s2: Backup with wowmc-check selected
  s3: Ready to restore
  s4: Malformed backup rejected
  s5: Restore failed#59; original kept
  s6: Restored#59; undo backup saved
  [*] --> s0
  [*] --> s1
  [*] --> s2
  s0 --> s4: preview backup
  s1 --> s3: preview backup
  s2 --> s3: preview backup
  s3 --> s5: write cannot begin
  s3 --> s6: apply restore
  s5 --> s3: retry
```

## import

TLC reached 6 states and 8 steps. The chart draws 4 states and 5 arrows.

## Rules that hold
GameUntouched, NoDuplicate, FailedKeepsPreview, PublishedOnlyAfterSave, NoDeadlock

```mermaid
---
title: "Importing a local snapshot"
---
stateDiagram-v2
  s0: No preview
  s1: Preview ready
  s2: Imported into library
  s3: Import failed#59; preview retained
  [*] --> s0
  s0 --> s1: preview snapshot
  s1 --> s2: import new names
  s1 --> s3: write fails
  s2 --> s1: preview again
  s3 --> s1: retry
```

## save-after

TLC reached 10 states and 22 steps. The chart draws 5 states and 9 arrows.
The chart does not draw 2 steps that stay in one chart state.

## Rules that hold
SavedMeansDurable, MemoryMeansDurable

```mermaid
---
title: "Editing and saving a macro"
---
stateDiagram-v2
  s0: Saved library
  s1: Unsaved draft
  s2: Close confirmation
  s3: Save failed#59; draft retained
  s4: Closed after discard
  [*] --> s0
  s0 --> s1: edit
  s1 --> s0: discard changes or save succeeds
  s1 --> s2: close with unsaved changes
  s1 --> s3: save fails
  s2 --> s1: keep editing
  s2 --> s4: discard and close
  s3 --> s0: discard changes
  s3 --> s1: retry
  s3 --> s2: close with unsaved changes
```

## writers-after

TLC reached 5 states and 5 steps. The chart draws 5 states and 4 arrows.
The chart does not draw 1 steps that stay in one chart state.

## Rules that hold
NoLostUpdate

```mermaid
---
title: "Conflicting library writes"
---
stateDiagram-v2
  s0: Both opened the same library
  s1: First change saved
  s2: Second save rejected
  s3: Second writer reloads
  s4: Both changes saved
  [*] --> s0
  s0 --> s1: first writer saves
  s1 --> s2: stale save rejected
  s2 --> s3: reload and reapply
  s3 --> s4: save again
```

## record

TLC reached 5 states and 7 steps. The chart draws 4 states and 6 arrows.
The chart does not draw 1 steps that stay in one chart state.

## Rules that hold
TrashRecoverable

```mermaid
---
title: "Library record lifecycle"
---
stateDiagram-v2
  s0: No record
  s1: Active
  s2: Active with history
  s3: In library trash
  [*] --> s0
  s0 --> s1: add
  s1 --> s2: edit and keep revision
  s1 --> s3: delete
  s2 --> s3: delete
  s3 --> s1: restore
  s3 --> s2: restore
```

## cache-writes

TLC reached 11 states and 16 steps. The chart draws 5 states and 7 arrows.

## Rules that hold
BackupBeforeWrite

```mermaid
---
title: "Cache writes and recovery"
---
stateDiagram-v2
  s0: No change staged
  s1: Change ready for review
  s2: Write failed
  s3: Cache written with backup
  s4: Backup restored
  [*] --> s0
  s0 --> s1: preview
  s1 --> s2: write fails
  s1 --> s3: write and keep backup
  s2 --> s1: retry
  s3 --> s1: preview another change
  s3 --> s4: restore backup
  s4 --> s1: preview another change
```
