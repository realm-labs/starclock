# Agent session settlement

Session creation immediately answers the system-owned battle-start decision and
stops at the first player-owned boundary. `play_action` resolves the selected
opaque token back to its retained exact `Command`, commits it only through
`Battle::apply`, and continues synchronously until the next player decision or
a terminal/fault boundary.

Decision ownership is closed and explicit:

- `Team(Player)` returns a new decision-scoped `OfferedActionSet`;
- `Team(Enemy)` uses `starclock-ai::EnemyController` with the immutable authored
  graph and selects one exact offered command;
- `System` accepts only the exact offered battle-start or pass-interrupt command.

The frozen Standard-v1 enemy graphs use static literal conditions and no
transitions. The agent evaluator supports literal/not/all/any trees and fails
closed if a future production graph requires battle-context condition semantics
that the controller boundary cannot safely evaluate. The controller RNG seed is
derived solely from schema-domain, scenario identity and master seed; it never
depends on session ID, clock, transport or scheduling.

Every successful command appends one `BattleTraceEntry` with the resulting full
state hash and one ordered controller record marked `external_player`,
`authored_enemy` or `system_automatic`. Resolver-owned automatic actions drain
inside that command's `Battle::apply` and remain in its authoritative events;
there is no per-hit mutation path. External settlement returns only counts and
the new controller records, never the retained command.

One external action plus internal settlement is bounded to 4,096 accepted
commands. Creation uses the same total bound for its system/enemy prelude. Event
and resolver-operation budgets are completed with retained paging and
instrumentation in the following session batches.
