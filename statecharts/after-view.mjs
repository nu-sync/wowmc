export const title = "Editing and saving a macro";
export const state = v => ({clean:"Saved library",dirty:"Unsaved draft",failed:"Save failed; draft retained",close:"Close confirmation",closed:"Closed after discard"})[v.phase];
export const step = a => ({Edit:"edit",WriteFails:"save fails",Retry:"retry",WriteSucceeds:"save succeeds",Discard:"discard changes",RequestClose:"close with unsaved changes",KeepEditing:"keep editing",DiscardClose:"discard and close",Done:"finished"})[a] ?? a;
