---------------------------- MODULE CacheWrites ----------------------------
EXTENDS Naturals
VARIABLES cache, backup, phase
vars == <<cache, backup, phase>>
Init == /\ cache = 0 /\ backup = 0 /\ phase = "ready"
Preview == /\ phase = "ready" /\ phase' = "preview" /\ UNCHANGED <<cache,backup>>
Save == /\ phase = "preview" /\ backup' = cache /\ cache' = 1 - cache /\ phase' = "written"
Fail == /\ phase = "preview" /\ phase' = "failed" /\ UNCHANGED <<cache, backup>>
Retry == /\ phase = "failed" /\ phase' = "preview" /\ UNCHANGED <<cache, backup>>
Restore == /\ phase = "written" /\ cache' = backup /\ backup' = cache /\ phase' = "restored"
NextPreview == /\ phase \in {"written", "restored"} /\ phase' = "preview" /\ UNCHANGED <<cache,backup>>
Next == Preview \/ Save \/ Fail \/ Retry \/ Restore \/ NextPreview
BackupBeforeWrite == phase = "written" => cache # backup
=============================================================================
