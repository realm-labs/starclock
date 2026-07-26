# Goal 07 Curio Partition S02

`G07-P3-M11-S02` executes eight positive, neutral and special Curios from the
authoritative Standard Simulated Universe workbooks:

- Rubert Empire Mechanical Cogwheel (`universe.curio.112`);
- Cavity System Model (`universe.curio.113`);
- Illusory Automaton (`universe.curio.118`);
- Society Ticket (`universe.curio.12`);
- Man-Made Meteorite (`universe.curio.120`);
- Thalan Toxi-Flame (`universe.curio.121`);
- The Pinkest Collision (`universe.curio.122`);
- Sealing Wax of Propagation (`universe.curio.123`).

The partition owns 16 records, 16 rules and three frozen semantic fixtures.
Excel remains authoritative: the relevant Curio, state, parameter, rule,
fixture and provenance rows live in `Universe.xlsx`,
`UniverseBindings.xlsx` and `UniverseEvidence.xlsx`. The partition authoring
tool reads them with pinned openpyxl, checks the committed Sora 0.3.0 export
and emits only derived evidence.

## Activity and battle boundary

Curio inventory, Cosmic Fragments, acquisition tokens and domain events remain
Activity state. Combat receives an immutable `CurioContributionSet`; it never
queries or mutates the run.

Cavity System Model captures `floor(fragments / 100)` when acquired, spends the
entire fragment balance and stores the captured count under a stable generic
runtime-value key. Only non-zero runtime values extend the contribution digest,
preserving all earlier zero-state snapshots. Battle assembly converts the
captured count to exactly 24% CRIT DMG per complete 100 fragments.

Rubert Empire Mechanical Cogwheel settles at each non-terminal Domain route.
It credits 50 fragments through the ordinary checked gain pipeline, then tests
the resulting balance. A value above 500 atomically removes the Curio, clears
the balance and increments the destroyed-Curio counter; 500 itself is retained.

Society Ticket is a post-battle reward category modifier. It multiplies those
fragment rewards by 175% after the shared Gossip multiplier. It does not affect
occurrence, service, acquisition or Domain-entry fragment changes.

## Executable combat rules

Illusory Automaton installs a generic `TurnStarted` rule that heals the current
allied actor for 20% of maximum HP through the ordinary healing formula.

Thalan Toxi-Flame marks the present allied unit with the highest current ATK at
`BattleStarted`. At each marked unit turn start it consumes 24% maximum HP with
a one-HP floor, then adds one permanent 5% base-SPD stack, capped at five.
Selection, HP consumption, effect stacking and stat recomputation are all typed
Rule IR operations.

The Pinkest Collision counts distinct Blessing Path IDs in the frozen battle
snapshot and grants 20% Break Effect for each. Duplicate Blessings of one Path
do not inflate the count.

No S02 mechanic uses a native handler or Curio-ID branch in the combat
resolver.

## Hidden random Blessing grants

Man-Made Meteorite and Sealing Wax of Propagation use the generic Activity
random-option boundary. The former samples an inclusive count from one to
three, then chooses that many unowned Blessings without replacement from the
selected run Path. The latter chooses one unowned Propagation Blessing.
Selected IDs, Reward-stream draw counters, Activity events and the resulting
state hash are authoritative and replayable; callers cannot provide outcomes.
Both use the ordinary Blessing acquisition operation and increment the
corresponding Path count.

Acquisition records a bounded pending Curio token. The runtime exposes the
canonically ordered pending Curio IDs and resolves one token through
`resolve_curio_acquisition_blessings`. The extension transaction may run while
a player decision is already visible without consuming or replacing that
decision. Failure restores both Activity state and RNG exactly.

Sealing Wax also increases Propagation Blessing weights in postcombat offers.
The public description does not publish a multiplier. Runtime revision v1
therefore freezes an explicit project-policy approximation of `x2`; the
conditional-weight primitive is generic and checked, so stronger evidence can
replace this scalar without changing offer or Activity architecture.

`standard-universe-entry-v8` and `standard-universe-topology-v8` identify the
new pending random-reward boundary, conditional offer weights and Domain-entry
Curio settlement. Replays produced under v7 cannot silently adopt these rules.

## Executable evidence

Production battle tests verify exact 48% captured CRIT DMG, 20% turn healing,
the Toxi-Flame HP-floor/five-stack SPD program and 20% per distinct Path Break
Effect. Activity tests verify Cavity acquisition, both Cogwheel threshold sides
and Society Ticket composition with Gossip. The random-boundary fixture proves
same-seed selection/hash parity, without-replacement selection and stale-command
state/RNG preservation. Topology tests verify every postcombat offer carries
the conditional Propagation subset policy.
