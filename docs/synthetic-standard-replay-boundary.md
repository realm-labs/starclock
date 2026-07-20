# Synthetic Standard CLI and Replay Boundary

`G01-P3-B6` closes the executable vertical-slice phase with one deliberately
synthetic Standard profile. It is not the production Standard activity or
enemy catalog: those remain owned by Phase 6. The fixture exists to prove the
current combat, replay and CLI boundaries end to end without adding challenge,
universe, shop, reward, score, clock or generic future-node semantics.

## Synthetic profile

`starclock_mode_standard::synthetic::SyntheticStandardProfile` creates one
immutable low-level battle handoff for `synthetic-standard-v1`:

- one player and one encounter enemy in one ordered wave;
- no implicit Skill Points, Energy, clock, score, objective or mode rule;
- the ordinary Action Gauge, interrupt, target, damage, defeat and victory
  paths from `starclock-combat`;
- a battle RNG seed derived from the master seed through the canonical
  `standard-v1` / `battle` stream path;
- fixed catalog, rules, configuration and battle-spec identities.

The player is faster and owns one single-target 1,000-damage Basic action; the
600-HP enemy owns a structural fallback action that does not execute in the
golden stream. The smoke controller consumes only offered values: StartBattle,
PassInterruptWindow, then UseAbility. It cannot fabricate a command.

This fixture constructs `Battle` directly, which the Standard contract permits
for synthetic tests. It intentionally does not claim the generic Activity,
participant/build-lock or result-projection work assigned to `G01-P6-B1` and
`G01-P6-B2`.

## Battle replay payloads

The frozen Version 1 replay envelope is unchanged. This batch defines only the
payload inside its existing low-level battle record kinds:

```text
AcceptedBattleCommand
  payload_version: u16 = 1
  command_kind: u8
  fixed-width command fields

ExpectedBattleState
  sha256-v2 digest: [u8; 32]
```

Records alternate command then expected state hash. The command decoder checks
the payload version, closed command discriminant, every non-zero typed ID,
option presence and trailing-byte boundary. `verify_battle_replay` validates
configuration, rules, catalog, numeric, RNG, state-hash, encounter and spec
identities before execution. It rebuilds one battle, applies each accepted
command exactly once, compares the hash immediately, retains only the current
battle/report, and reports the first command index on rejection or divergence.
It never replays growing prefixes.

Activity commands, nested battle results, controller scoring diagnostics and
full structural divergence paths remain reserved for `G01-P6-B4`.

## CLI smoke surface

The temporary vertical-slice surface is:

```text
starclock battle run \
  --scenario synthetic-standard-v1 \
  --seed U64 \
  [--replay-out PATH] [--json]

starclock replay verify FILE [--json]
```

`battle run` writes no file unless `--replay-out` is supplied. JSON mode emits
one schema-1 object to stdout; errors go to stderr. Exit classes are `2` usage,
`3` scenario/identity, `4` replay failure, `5` simulation failure and `6` I/O.
Phase 6 completes the broader documented CLI catalog/config/controller surface.

## Golden evidence

For master seed `7`, both independent runs produce:

- 3 accepted commands and 6 alternating replay records;
- terminal phase `Won`;
- final state hash
  `697a383ec91618282442011c89c12616736273e897c2846a793abe4d1a55f272`;
- 530 canonical replay bytes;
- replay SHA-256
  `af1623db54a28aa7374ef560bf76824a4896deb922c923dc52e8465a986e4616`.

[`standard_replay_smoke.rs`](../crates/starclock-cli/tests/standard_replay_smoke.rs)
runs the installed binary twice, compares stdout and replay bytes, verifies the
file, then mutates the first expected command hash and requires a divergence at
command index 0. The mode test rebuilds the same catalog/spec/seed into the same
initial state hash.
