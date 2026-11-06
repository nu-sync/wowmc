export const title = "Editing a macro: current behavior";
export const state = (v) => v.phase === "failed" ? "Save failed" : v.phase === "dirty" ? "Unsaved changes" : (v.draft === v.disk ? "Library matches editor" : "Editor appears saved but differs from disk");
export const step = (a) => ({Edit:"edit", WriteFails:"save fails", WriteSucceeds:"save succeeds", Discard:"discard changes", Reset:"start over"})[a] ?? a;
