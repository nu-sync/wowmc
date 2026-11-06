--------------------------- MODULE LibrarySave ---------------------------
EXTENDS Naturals
CONSTANT PreserveNoOpBackup
VARIABLES phase, draft, disk, backup, memory
vars == <<phase, draft, disk, backup, memory>>
Init == /\ phase = "ready" /\ draft \in {1, 2}
        /\ disk = 1 /\ backup = 0 /\ memory = 1
Save == /\ phase = "ready" /\ disk = 1
        /\ phase' = "saved" /\ disk' = draft /\ memory' = draft
        /\ backup' = IF PreserveNoOpBackup /\ draft = disk THEN backup ELSE disk
        /\ UNCHANGED draft
ExternalEdit == /\ phase = "ready" /\ disk = 1 /\ disk' = 3
                /\ UNCHANGED <<phase, draft, backup, memory>>
RejectConflict == /\ phase = "ready" /\ disk = 3 /\ phase' = "conflict"
                  /\ UNCHANGED <<draft, disk, backup, memory>>
Fail == /\ phase = "ready" /\ phase' = "failed"
        /\ UNCHANGED <<draft, disk, backup, memory>>
Retry == /\ phase = "failed" /\ phase' = "ready"
         /\ UNCHANGED <<draft, disk, backup, memory>>
Done == /\ phase \in {"saved", "conflict"} /\ UNCHANGED vars
Next == Save \/ ExternalEdit \/ RejectConflict \/ Fail \/ Retry \/ Done
UnchangedKeepsRecovery == phase = "saved" /\ draft = 1 => backup = 0
ChangedBacksUpPrevious == phase = "saved" /\ draft = 2 => backup = 1
SavedMatchesDraft == phase = "saved" => disk = draft /\ memory = draft
ConflictLeavesFilesAlone == phase = "conflict" => disk = 3 /\ backup = 0
FailureDoesNotPublish == phase # "saved" => memory = 1
=============================================================================
