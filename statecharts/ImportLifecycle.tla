-------------------------- MODULE ImportLifecycle --------------------------
EXTENDS Naturals
VARIABLES phase, memory, disk, game, preview
vars == <<phase, memory, disk, game, preview>>
Init == /\ phase = "idle" /\ memory = 0 /\ disk = 0 /\ game = "unchanged" /\ preview = FALSE
Preview == /\ phase = "idle" /\ phase' = "review" /\ preview' = TRUE /\ UNCHANGED <<memory,disk,game>>
Apply == /\ phase = "review" /\ phase' = "imported" /\ memory' = 1 /\ disk' = 1 /\ preview' = FALSE /\ UNCHANGED game
Failure == /\ phase = "review" /\ phase' = "failed" /\ UNCHANGED <<memory,disk,game,preview>>
Retry == /\ phase = "failed" /\ phase' = "review" /\ UNCHANGED <<memory,disk,game,preview>>
Again == /\ phase = "imported" /\ phase' = "review" /\ preview' = TRUE /\ UNCHANGED <<memory,disk,game>>
Next == Preview \/ Apply \/ Failure \/ Retry \/ Again
GameUntouched == game = "unchanged"
NoDuplicate == memory <= 1 /\ disk <= 1
FailedKeepsPreview == phase = "failed" => preview
PublishedOnlyAfterSave == memory = disk
=============================================================================
