# RestoreLifecycle

TLC reached 10 states and 11 steps. The chart draws 8 states and 6 arrows.
The chart does not draw 4 steps that stay in one chart state.

## ValidBackupAccepted breaks
The shortest path has 1 step:
0. Backup with wowmc-check selected
1. preview backup: Valid backup rejected

## Rules that hold
MalformedNeverWritten, PreviewDoesNotWrite, BackupBeforeRestore, NoDeadlock
