export const title = "Restoring a cache backup";
export const state = v => {
  if (v.phase === "chosen") return !v.valid ? "Malformed backup selected" : v.collision ? "Backup with wowmc-check selected" : "Other valid backup selected";
  if (v.phase === "rejected") return v.valid ? "Valid backup rejected" : "Malformed backup rejected";
  return {preview:"Ready to restore", restored:"Restored; undo backup saved", failed:"Restore failed; original kept"}[v.phase];
};
export const step = a => ({Preview:"preview backup",Apply:"apply restore",Fail:"write cannot begin",Retry:"retry",Done:"finished"})[a] ?? a;
