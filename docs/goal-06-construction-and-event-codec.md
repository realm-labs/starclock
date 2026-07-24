# Goal 06 Construction and Event Codec Migration

`G06-P1-B4` closes the temporary construction and attribution bridges introduced
while combat-input identity and replay v3 were staged.

## Battle construction

`BattleSpec::new` accepts an opaque `AssemblyDigest`, canonicalizes the supplied
battle values and computes `CombatInputDigest` inside `starclock-combat`. No
public battle constructor accepts `BattleSpecDigest` or a caller-computed
combat-input digest. Build, data, Standard battle fixtures and Standard Universe
materialization use this one contract.

`BattleSpecDigest` remains only in historical replay-v1/v2 and Activity payload
decoders. It is not a battle-construction authority.

## Event attribution

Executable event attribution is `Cause::source_definition`. The former
`activity_source` option had no writer and duplicated outer provenance, so it
was removed from combat state rather than exposing Activity vocabulary to the
resolver.

Battle-event payload v2 omits that field. The replay-v2 recorder and verifier
retain explicit payload-v1 support and reproduce its permanently empty option
byte exactly. Replay v3 emits payload v2. Unknown event payload revisions fail
closed and are reported as event divergence.

The Standard Universe event commitment remains byte-compatible at
`deterministic-battle-input-event-shape-v1`; it explicitly writes the historical
empty reserved option. Dual identity is bound by the handoff/result contract,
so replay transport cleanup does not silently relabel that older commitment.

## Responsibility split

Versioned cause encoding lives in `battle_event_cause.rs`. Standard Universe
battle request construction lives in
`battle_materialization/battle_spec.rs`. This keeps the touched event codec and
materializer below the project source-size limit without changing their public
ownership boundaries.
