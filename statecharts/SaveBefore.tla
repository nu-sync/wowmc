--------------------------- MODULE SaveBefore ---------------------------
EXTENDS Naturals
VARIABLES disk, memory, draft, phase
vars == <<disk, memory, draft, phase>>
Init == /\ disk = 0 /\ memory = 0 /\ draft = 0 /\ phase = "clean"
Edit == /\ phase = "clean" /\ draft' = 1 /\ phase' = "dirty" /\ UNCHANGED <<disk, memory>>
WriteFails == /\ phase = "dirty" /\ memory' = draft /\ phase' = "failed" /\ UNCHANGED <<disk, draft>>
WriteSucceeds == /\ phase = "dirty" /\ disk' = draft /\ memory' = draft /\ phase' = "clean" /\ UNCHANGED draft
Discard == /\ phase = "failed" /\ draft' = memory /\ phase' = "clean" /\ UNCHANGED <<disk, memory>>
Reset == /\ phase = "clean" /\ disk = 1 /\ disk' = 0 /\ memory' = 0 /\ draft' = 0 /\ UNCHANGED phase
Next == Edit \/ WriteFails \/ WriteSucceeds \/ Discard \/ Reset
SavedMeansDurable == phase = "clean" => draft = disk
MemoryMeansDurable == memory = disk
=============================================================================
