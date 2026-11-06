--------------------------- MODULE WritersBefore ---------------------------
VARIABLES disk, a, b, stage, lost
vars == <<disk, a, b, stage, lost>>
Init == /\ disk = 0 /\ a = 0 /\ b = 0 /\ stage = "both read" /\ lost = FALSE
SaveA == /\ stage = "both read" /\ disk' = 1 /\ stage' = "A saved" /\ UNCHANGED <<a,b,lost>>
SaveB == /\ stage = "A saved" /\ disk' = 2 /\ stage' = "B saved" /\ lost' = TRUE /\ UNCHANGED <<a,b>>
Done == /\ stage = "B saved" /\ UNCHANGED vars
Next == SaveA \/ SaveB \/ Done
NoLostUpdate == ~lost
=============================================================================
