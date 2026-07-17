# Battle transaction boundary

Goal 01 batch `G01-P3-B2` makes every accepted battle command an owned,
synchronous transaction. Legality validation remains a read-only boundary; only
after it succeeds may `Battle` prepare non-authoritative scratch.

## Owned working state

Each battle lazily retains one private `ResolutionScratch`. Its working
`BattleState` receives a semantic `clone_from` into existing vectors and fixed
storage. A successful or faulted commit swaps this working allocation with the
authoritative state, so the former authoritative allocation becomes the next
scratch buffer. Scratch, journal capacity and preparation counters are excluded
from views and canonical state.

The forward journal records state mutations, fixed-width identity allocations,
events and the required RNG/snapshot/queue fact families in append order. It is
an audit and rollback mechanism, not a replay format. Retained journal capacity
is cleared after settlement and discarded when it exceeds 4,096 entries; the
current construction bound limits retained unit/actor storage to 40 initial
participants. Returned event vectors transfer ownership to `Resolution` and
are not reused behind the caller's back.

## Settlement and faults

Successful commands settle at `AwaitingCommand` or a terminal phase, stream the
complete working state into SHA-256 and then swap. A `Rollback` failure discards
all uncommitted semantic work, rebuilds its single stable fault transition from
the pre-command state and commits that. A `CommitFault` failure retains facts
from completed atomic work, appends a cause-linked fault fact and commits the
terminal state. Stable fault kind, boundary, policy, context code and optional
integer context persist in authoritative state; platform error strings do not.

Test-only injection at two transaction depths proves rollback convergence to
byte-identical state and hashes without adding any public command or runtime
failure switch. Another fixture proves that commit-fault retains already
emitted facts and links the fault to the immediate parent. Rejected commands
leave the canonical hash unchanged and do not create or prepare scratch.

## Events, causes and canonical hashing

Every accepted command receives a monotonic `CommandId`; every emitted fact
receives a monotonic `EventId`. `Cause` carries the root command, immediate
parent, action/phase/hit, rule owner, actor, applier, source definition, primary
target and optional activity source as distinct fields. Unavailable attribution
is explicit `None` and is never inferred from a neighboring role.

The private state codec writes a fixed versioned field order directly to a sink
and includes compatibility identity, phase/fault, decision and commands,
tombstone-capable stores, formation/team/encounter state, RNG seed/draw count,
all sequence allocators and accepted-command revision. It excludes catalog
bodies, pointers, caches, scratch, journals, events and diagnostic text. The
production path streams into SHA-256 without allocating a complete state byte
vector; a test-only collecting sink proves byte/hash equivalence. Fixed vectors
cover initial state, StartBattle and Concede boundaries.

Action plans, operation/reaction queues and real RNG-consuming resolution begin
in `G01-P3-B3`; they must use the journal families frozen here rather than
creating a second transaction or event language.
