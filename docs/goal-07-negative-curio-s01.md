# Goal 07 Negative Curio Partition S01

`G07-P3-M12-S01` executes the first seven negative and repairable Standard
Simulated Universe Curios:

- Fission Cuckoo Clock (`universe.curio.108`);
- Fool's Mask (`universe.curio.115`);
- Void Wick Trimmer (`universe.curio.17`);
- Shining Trapezohedron Die (`universe.curio.21`);
- Corrupted Code (`universe.curio.45`);
- Odd Code (`universe.curio.47`);
- Normal Code (`universe.curio.49`), fixed state only.

The partition owns 16 records, 16 rules and six review fixtures. The editable
definitions, parameters, rules and provenance remain authoritative in
`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx`.
`author-curio-partition.py` verifies the selected Excel rows against the Sora
0.3.0 bundle and emits the derived partition golden. Curio states may be split
between M12 partitions, so the verifier resolves state ownership and initial
links against the complete workbook instead of requiring a partition-local
closed graph.

## Run-level replacement and repair

Destroyed Curios are retained as deterministic identity counters rather than
only a total. Void Wick Trimmer samples up to two distinct destroyed Curios
from the Reward RNG stream and restores their ordinary initial states,
charges and acquisition events atomically.

Shining Trapezohedron Die removes every currently owned Curio, including
itself, and selects the same number of distinct replacements from the
currently unowned pool without replacement. The complete removal prefix and
all acquisitions commit in one random boundary.

Fool's Mask accepts a canonical replay-recorded mapping covering the complete
Blessing inventory. It validates distinct targets of the same or higher
rarity, removes every old Blessing before adding any replacement, and retains
each exact enhancement level. The released description does not publish the
higher-rarity probability, so the runtime does not invent one.

## Battle-count repair and combat rules

Every won battle consumes one repair charge from each Curio currently in a
repairing state. The third win transitions Corrupted Code and Odd Code to
their fixed states in the same battle-settlement transaction. The state
machine and charge data remain generic Curio runtime primitives.

Corrupted Code reacts when its wearer breaks Weakness. The repairing state
sets Energy to zero; the fixed state sets Energy to the actor's maximum.
Odd Code reacts after the wearer's Ultimate. The repairing state consumes 30%
of current HP with the ordinary one-HP floor; the fixed state heals 30% of
current HP. Normal Code's fixed state applies 50% mitigation to ordinary,
DoT, Break, Super Break, additional, joint, Elation and true damage. Its
repairing state belongs to `G07-P3-M12-S02`.

Fission Cuckoo Clock applies one 5% ATK penalty per concurrent copy. The
Activity stores additional copies as a bounded runtime value, caps the total
at three and snapshots the count into battle contribution data. A won battle
creates one pending split decision. Because the released split probability is
not public, the replay command records `NoSplit` or `Split` explicitly.

All combat effects lower to typed Rule IR. Run-level replacement, repair and
unpublished random outcomes use generic Activity operations and explicit
commands. No native handler is admitted, and the resolver contains no Curio-ID
branch.

## Revisions and executable evidence

`standard-universe-entry-v13` and `standard-universe-topology-v13` identify
destroyed-Curio identity tracking, repair settlement and negative-Curio
pending state. Unit tests execute acquisition replacements, three-battle
repair and fission copy limits. Combat integration tests execute a real
Ultimate with Odd Code, a real normal attack under the Fission ATK penalty,
and structurally verify both Code state programs and all eight Normal Code
mitigation purposes.

All nine native-handler candidate reviews close as `IrSufficient`. The only numeric
uncertainties in this partition are represented as external replay decisions,
not guessed probabilities.
