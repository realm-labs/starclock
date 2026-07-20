# Ability-Level and Trace Compilation

`G01-P5-B2` extends the independent build compiler with exact level/promotion
stat rows, complete ability-level tables, Trace graphs and the first closed
typed build-patch subset. These remain build-domain definitions; combat sees
only the selected ability, rule-bundle and modifier IDs in a generic
`ResolvedCombatantSpec`.

## Ability curves

An ability table owns one stable family ID, an invested-level cap and a
contiguous row for every effective level from one through its maximum. Each row
selects an already validated combat ability definition. This is the permitted
“select a validated combat-definition variant” path from the normative build
contract; it does not copy coefficients or generated Sora rows into combat.

Catalog validation sorts tables and rows, rejects duplicate families, gaps,
overlapping resolved variants, missing combat definitions and variants absent
from the owning unit. An exact build supplies one investment for every table.
Missing, extra, duplicate or above-cap investments are errors; the compiler
never clamps them.

## Trace graph and patches

Each form may own one graph of stable `TraceNodeId` values. Nodes retain an
ordered prerequisite set, promotion requirement and authored patch sequence.
Catalog construction rejects duplicate/cross-form nodes, unresolved or
noncanonical prerequisites, cycles and patch references that are not valid for
the owning combat form.

B2 admits exactly four generic patch families:

- add one combat ability (ability unlock);
- add one rule bundle (major passive);
- add one modifier definition (minor-stat or persistent stat contribution);
- add a signed ability-level bonus and cap delta.

No patch contains a field path, JSON, Rust callback or content-ID branch.
Adding Eidolon replacement/conflict semantics remains B3 work.

The graph compiles through Kahn topological ordering with `TraceNodeId` as the
ready-set tie-break. Build input Trace IDs and ability investments are sorted
into exact sets while duplicates are rejected. Compilation validates complete
prerequisite closure and promotion, applies selected nodes in canonical graph
order and patch rows in authored order, then resolves effective levels with
checked signed arithmetic and an explicit cap.

## Evidence boundary

The synthetic golden compiles invested level two plus a two-node Trace chain to
effective level three, an unlocked ability, one rule bundle and one stat
modifier. Reversing catalog insertion and build-selection order produces an
equal report and equal combatant. Invalid curves, cycles, prerequisites,
promotion and caps are typed failures.

The committed production workbooks remain the only future authoritative
content source. This batch adds no JSON runtime path, does not enable a workbook
row and does not claim character coverage; production remains 0/283
`DataReady`. B3-B5 add Eidolons, Light Cones and canonical source/digest output
before any compiled build is accepted by production orchestration.

Run the focused gate with:

```text
cargo test -p starclock-build --all-targets --all-features
```
