export const title = "Two writers: current behavior";
export const state = v => v.stage === "both read" ? "Both opened the same library" : v.stage === "A saved" ? "First change saved" : "First change lost";
export const step = a => ({SaveA:"first writer saves",SaveB:"second writer saves",Done:"finished"})[a] ?? a;
