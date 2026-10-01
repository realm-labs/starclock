# Battle-entry participant carry

The shared Activity handoff binds carried HP and Energy to the maxima of the
currently accepted immutable battle input. This covers lawful changes of
capacity between battles without adding a mode-specific state machine.

For a participant with a prior verified carry record, `start_pending_battle`
applies the requested contract's resource and life/presence policies:

| Policy | Destination projection |
| --- | --- |
| HP / Energy `CarryExact` | Preserve the absolute value; reject values above the new maximum. |
| HP / Energy `CarryClamped` | Preserve values within the new maximum; clamp values above it. |
| HP `RestoreFull` | Use the new maximum, with an explicitly compatible life policy. |
| Energy `ResetZero` | Use zero, including when the new capacity is zero. |
| Life / presence | Apply their declared policies; reject inconsistent HP/life pairs. |

Increasing capacity never grants HP or Energy under exact/clamped policies.
Fractional Energy retains its fixed-point value; no ratio scaling, floating
conversion or implicit resurrection occurs. A first entry with no ledger record
retains the participant's authored initial state instead of applying a restore
policy to a nonexistent prior battle.

Projection is read-only until the awaiting-battle boundary commits. It does not
replace the ledger: room operations continue to see the last verified carry,
and the next verified result settles the destination maxima normally. The
result maximum checks remain strict. An invalid exact value or incompatible
life projection returns typed `CarryInvariant` before publishing a battle;
canonical state, pending input and RNG remain unchanged.

Tests cover reduced and increased capacities, exact-boundary rejection,
fractional and zero Energy, independent life/presence policy, actual two-battle
handoff/settlement, repeat rejection and fresh reconstruction. This is a shared
prerequisite, not completion credit for Divergent Universe Curio 1004 or any
other full gameplay effect.
