-------------------------- MODULE RecordLifecycle --------------------------
VARIABLES state, history
vars == <<state,history>>
Init == /\ state = "absent" /\ history = FALSE
Add == /\ state = "absent" /\ state' = "active" /\ UNCHANGED history
Edit == /\ state = "active" /\ history = FALSE /\ history' = TRUE /\ UNCHANGED state
Delete == /\ state = "active" /\ state' = "trash" /\ UNCHANGED history
Restore == /\ state = "trash" /\ state' = "active" /\ UNCHANGED history
RestoreRevision == /\ state = "active" /\ history /\ UNCHANGED vars
Next == Add \/ Edit \/ Delete \/ Restore \/ RestoreRevision
TrashRecoverable == state = "trash" => ENABLED Restore
=============================================================================
