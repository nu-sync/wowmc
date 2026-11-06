export const title = "Library record lifecycle";
export const state = v => v.state === "absent" ? "No record" : v.state === "trash" ? "In library trash" : v.history ? "Active with history" : "Active";
export const step = a => ({Add:"add",Edit:"edit and keep revision",Delete:"delete",Restore:"restore",RestoreRevision:"restore revision"})[a] ?? a;
