# LibrarySave

TLC reached 12 states and 18 steps. The chart draws 9 states and 11 arrows.
The chart does not draw 4 steps that stay in one chart state.

## UnchangedKeepsRecovery breaks
The shortest path has 1 step:
0. Unchanged macro
1. save: Unchanged; recovery backup overwritten

## Rules that hold
ChangedBacksUpPrevious, SavedMatchesDraft, ConflictLeavesFilesAlone, FailureDoesNotPublish, NoDeadlock
