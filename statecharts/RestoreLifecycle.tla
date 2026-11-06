------------------------- MODULE RestoreLifecycle -------------------------
EXTENDS Naturals
CONSTANT IndependentValidation
VARIABLES phase, valid, collision, cache, backup
vars == <<phase, valid, collision, cache, backup>>
Init == /\ phase = "chosen" /\ valid \in BOOLEAN /\ collision \in BOOLEAN
        /\ cache = "original" /\ backup = "none"
Preview == /\ phase = "chosen"
           /\ phase' = IF valid /\ (IndependentValidation \/ ~collision)
                       THEN "preview" ELSE "rejected"
           /\ UNCHANGED <<valid, collision, cache, backup>>
Apply == /\ phase = "preview" /\ phase' = "restored"
         /\ backup' = cache /\ cache' = "replacement"
         /\ UNCHANGED <<valid, collision>>
Fail == /\ phase = "preview" /\ phase' = "failed" /\ UNCHANGED <<valid, collision, cache, backup>>
Retry == /\ phase = "failed" /\ phase' = "preview" /\ UNCHANGED <<valid, collision, cache, backup>>
Done == /\ phase \in {"rejected", "restored"} /\ UNCHANGED vars
Next == Preview \/ Apply \/ Fail \/ Retry \/ Done
ValidBackupAccepted == phase = "rejected" => ~valid
MalformedNeverWritten == ~valid => cache = "original"
PreviewDoesNotWrite == phase \in {"chosen", "preview", "rejected", "failed"} => cache = "original"
BackupBeforeRestore == cache = "replacement" => backup = "original"
=============================================================================
