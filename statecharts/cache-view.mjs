export const title = "Cache writes and recovery";
export const state = v => ({ready:"No change staged",preview:"Change ready for review",written:"Cache written with backup",failed:"Write failed",restored:"Backup restored"})[v.phase];
export const step = a => ({Preview:"preview",Save:"write and keep backup",Fail:"write fails",Retry:"retry",Restore:"restore backup",NextPreview:"preview another change"})[a] ?? a;
