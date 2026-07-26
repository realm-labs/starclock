# Goal 07 Curio Partition S05

`G07-P3-M11-S05` executes eight positive Standard Simulated Universe Curios:

- Casket of Inaccuracy (`universe.curio.5`);
- Obliteration Wick Trimmer (`universe.curio.58`);
- Ambergris Cheese (`universe.curio.6`);
- Laurel Crown of Planar Shifts (`universe.curio.61`);
- Space-Time Prism (`universe.curio.62`);
- Cosmic Big Lotto (`universe.curio.63`);
- Omniscient Capsule (`universe.curio.64`);
- Punklorde Mentality (`universe.curio.68`).

The partition owns 16 records, 16 rules and the destructible fixture. The
definitions, parameters, rules and provenance remain authoritative in
`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx`.
The openpyxl partition command verifies those rows against the committed Sora
0.3.0 bundle and emits the derived partition golden.

## Reward and destructible boundaries

Casket of Inaccuracy samples one or two unowned Blessings without replacement
from the complete eligible catalog. It deliberately does not reuse the
selected-Path pool used by Indecipherable Box.

Every destroyed object increments one private Activity counter exactly once.
Obliteration Wick Trimmer snapshots that counter at battle assembly and lowers
three percent all-character damage per object to ordinary combat modifiers.
Omniscient Capsule exposes a spatial-free destructible policy: the host
receives the released qualitative `more_frequent` flag and the exact reward
multiplier of two. It does not require a 3D scene.

Cosmic Big Lotto receives a closed, replayable destructible outcome command.
`NoEffect`, Curio acquisition and failure are explicit outcomes because the
public source does not publish the two “small chance” probabilities. Curio
success uses the ordinary acquisition lifecycle. Failure atomically destroys
the Lotto and removes 99% of each living participant's current HP, with the
normal nonlethal one-HP floor. Multiple Lotto Curios attached to one destroyed
object resolve in stable Curio-ID order while the object counter advances only
once.

## Battle settlement and build assembly

Ambergris Cheese adds a checked post-victory carry operation that heals every
living participant for 30% of maximum HP. Combat only returns its participant
projection; inventory and cross-battle healing remain Activity concerns.

Laurel Crown validates the submitted battle result before replacing a loss in
a non-Boss domain with a victory projection. It restores all participants to
alive, present and full HP, consumes its only charge, and retains the original
combat event and final-state hashes as evidence. Standard Universe treats the
Boss domain as the final domain. A Boss loss is never converted.

Space-Time Prism is a build-compiler boundary, not a combat Buff. The mode
adapter requires the exact locked `CombatantBuildSpec`, recompiles and verifies
both its build and resolved-combatant digests, increases Eidolon Resonance by
one up to E6, and then performs ordinary battle materialization. A roster that
omits or mismatches the upstream build selection faults explicitly instead of
silently ignoring the Curio.

## Punklorde weakness implant

Punklorde Mentality lowers to generic Rule IR at `BattleStarted`. It gathers
the Basic ATK elements of allies that are alive and present, sorts and
deduplicates them, selects one through the authoritative battle RNG, and adds
that same weakness to every enemy for three target turns. The released source
defines the candidate set, count, fixed 100% chance and duration but does not
publish how one element is selected from a mixed team. Runtime v1 therefore
records deterministic uniform selection as a replaceable project policy.

The implementation adds a reusable allied-element weakness operation and RNG
purpose. It does not branch on Curio IDs inside the combat resolver.

## Revision and executable evidence

`standard-universe-entry-v11` and `standard-universe-topology-v11` identify the
new destructible, settlement, build-recompilation and weakness-implant
semantics. Tests execute exact Casket selection, one-count destructible
settlement, Capsule policy, both Lotto terminal paths, Ambergris healing,
Laurel conversion, Wick Trimmer damage, Punklorde shared weakness and
Space-Time Prism E2-to-E3 recompilation. All eight native-handler reviews close
as `IrSufficient`.
