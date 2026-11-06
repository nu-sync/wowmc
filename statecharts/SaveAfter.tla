--------------------------- MODULE SaveAfter ---------------------------
EXTENDS Naturals
VARIABLES disk, memory, draft, phase
vars == <<disk, memory, draft, phase>>
Init == /\ disk = 0 /\ memory = 0 /\ draft = 0 /\ phase = "clean"
Edit == /\ phase = "clean" /\ draft' = 1 - disk /\ phase' = "dirty" /\ UNCHANGED <<disk, memory>>
WriteFails == /\ phase = "dirty" /\ phase' = "failed" /\ UNCHANGED <<disk, memory, draft>>
Retry == /\ phase = "failed" /\ phase' = "dirty" /\ UNCHANGED <<disk, memory, draft>>
WriteSucceeds == /\ phase = "dirty" /\ disk' = draft /\ memory' = draft /\ phase' = "clean" /\ UNCHANGED draft
Discard == /\ phase \in {"dirty", "failed"} /\ draft' = memory /\ phase' = "clean" /\ UNCHANGED <<disk, memory>>
RequestClose == /\ phase \in {"dirty", "failed"} /\ phase' = "close" /\ UNCHANGED <<disk, memory, draft>>
KeepEditing == /\ phase = "close" /\ phase' = "dirty" /\ UNCHANGED <<disk, memory, draft>>
DiscardClose == /\ phase = "close" /\ phase' = "closed" /\ UNCHANGED <<disk, memory, draft>>
Done == /\ phase = "closed" /\ UNCHANGED vars
Next == Edit \/ WriteFails \/ Retry \/ WriteSucceeds \/ Discard \/ RequestClose \/ KeepEditing \/ DiscardClose \/ Done
SavedMeansDurable == phase = "clean" => draft = disk
MemoryMeansDurable == memory = disk
=============================================================================
