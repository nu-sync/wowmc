export const title = "Importing a local snapshot";
export const state = v => ({idle:"No preview",review:"Preview ready",failed:"Import failed; preview retained",imported:"Imported into library"})[v.phase];
export const step = a => ({Preview:"preview snapshot",Apply:"import new names",Failure:"write fails",Retry:"retry",Again:"preview again"})[a] ?? a;
