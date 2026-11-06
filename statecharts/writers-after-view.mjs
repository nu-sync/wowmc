export const title = "Conflicting library writes";
export const state = v => ({"both read":"Both opened the same library","A saved":"First change saved",conflict:"Second save rejected",reloaded:"Second writer reloads","both saved":"Both changes saved"})[v.stage];
export const step = a => ({SaveA:"first writer saves",RejectB:"stale save rejected",Reload:"reload and reapply",SaveB:"save again",Done:"finished"})[a] ?? a;
