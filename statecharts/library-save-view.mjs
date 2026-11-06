export const title = "Saving while keeping a useful backup";
export const state = v => {
  if (v.phase === "saved") return v.draft === 2 ? "Edit saved; previous version backed up"
    : v.backup === 0 ? "Unchanged; recovery backup kept" : "Unchanged; recovery backup overwritten";
  if (v.phase === "conflict") return "Conflict detected; draft kept";
  if (v.phase === "failed") return v.disk === 3 ? "Save failed after external edit"
    : v.draft === 1 ? "Unchanged save failed" : "Edited save failed";
  if (v.disk === 3) return "File edited outside wowmc";
  return v.draft === 1 ? "Unchanged macro" : "Edited macro";
};
export const step = a => ({
  Save:"save", ExternalEdit:"another tool edits the file", RejectConflict:"save detects conflict",
  Fail:"write cannot begin", Retry:"retry", Done:"finished"
})[a] ?? a;
