# SaveBefore

TLC reached 8 states and 12 steps. The chart draws 4 states and 6 arrows.
The chart does not draw 1 steps that stay in one chart state.

## MemoryMeansDurable breaks
The shortest path has 2 steps:
0. Library matches editor
1. edit: Unsaved changes
2. save fails: Save failed

## SavedMeansDurable breaks
The shortest path has 3 steps:
0. Library matches editor
1. edit: Unsaved changes
2. save fails: Save failed
3. discard changes: Editor appears saved but differs from disk
