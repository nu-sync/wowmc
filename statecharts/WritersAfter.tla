--------------------------- MODULE WritersAfter ---------------------------
VARIABLES disk, stage, lost
vars == <<disk, stage, lost>>
Init == /\ disk = 0 /\ stage = "both read" /\ lost = FALSE
SaveA == /\ stage = "both read" /\ disk' = 1 /\ stage' = "A saved" /\ UNCHANGED lost
RejectB == /\ stage = "A saved" /\ stage' = "conflict" /\ UNCHANGED <<disk,lost>>
Reload == /\ stage = "conflict" /\ stage' = "reloaded" /\ UNCHANGED <<disk,lost>>
SaveB == /\ stage = "reloaded" /\ disk' = 3 /\ stage' = "both saved" /\ UNCHANGED lost
Done == /\ stage = "both saved" /\ UNCHANGED vars
Next == SaveA \/ RejectB \/ Reload \/ SaveB \/ Done
NoLostUpdate == ~lost /\ (stage \in {"A saved","conflict","reloaded"} => disk = 1) /\ (stage = "both saved" => disk = 3)
=============================================================================
