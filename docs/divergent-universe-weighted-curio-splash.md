# Divergent Universe Weighted Curio attack splash

## Released operands and ownership

The production decision workbook's `WeightedCurioSplashes` row binds Weighted
Curio 1001 / Ten Light-Years' Foresight to generic MazeBuff 633401. Sources 73–76
pin released Version 4.4 revision `fd978d6ef09f941fba644c731ab54abd6f7c3568`
of [Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
accessed 2026-09-27. They retain exact locators and full SHA-256 digests:

| File / locator | Evidence |
| --- | --- |
| `RogueTournHex.json`, HexID 1001, `TournMode=Tourn3` | Display 1014, MazeBuff 633401, inline `Rogue` / Hunt selector |
| `MazeBuff.json`, ID 633401, level 1 | Canonical parameter `0.3`; `StageAbilityBeforeCharacterBorn` / `StageAbility_633401` |
| `TextMapEN.json` / `TextMapCHS.json`, hash `8678628073262894449` | Hunt attack damage copies thirty percent to adjacent targets |

All seventeen current Hex MazeBuff references resolve in **generic**
`MazeBuff.json`. Absence from `RogueMazeBuff` did not establish absence of released
operands or text. The generic file digest is
`2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac`.
The original reference catalog still records its RogueMazeBuff-only unresolved
lookup; the executable table is an independent validated source join, not
permission to mark that catalog as a complete program. The owning inventory
verifier's `--check-source` checks all seventeen generic joins and the production
splash operand against the pinned source. No upstream program/prose is committed.

The [publisher's released introduction](https://www.hoyolab.com/article/33175494)
supports the Hunt group-damage role, not current numbers or hidden timing.
Public earlier-module texts differ; the pinned current operand wins. A filename
search of the pinned tree did not locate this binding's executable program.
A broad content scan required uncached source blobs and was stopped; it did not
prove program absence. Source reconstruction remains a replacement opportunity.

## Executable policy

`VersionedProjectPolicyHitCalculatedCopyAdjacentTrueDamage` preserves the exact
fraction and Hunt eligibility while explicitly choosing:

- Eligibility at immutable assembly from the mapped character build's Path;
  non-Hunt players and enemy specs receive no binding. Linked/new summon
  inheritance is not implicit. The locked build, base stats and old bindings
  remain unchanged.
- Each positive ordinary Ability damage event with an action is observed once
  per event at the shared `AfterEvent` phase. This includes separate hits and separately
  damaged targets. The source class prevents copied damage from retriggering.
- The basis is calculated integer damage, including shield-absorbed and overkill
  portions, not effective HP loss or unfloored raw damage. Checked fixed-point
  multiplication by `0.3` precedes integral floor: `19 → 5`.
- Current alive/present opposing units exactly one formation slot from the
  damaged target are selected in formation/stable-ID order. A defeated primary
  remains the origin; holes are not compressed into neighbors.
- Shared `TrueDamage` operations can defeat and bypass source/target damage
  multipliers and critical rolls. Their event class is generic Additional with
  no element; this is not an authored elemental additional-damage formula.
- The equipment survives to the next battle until replacement, unequip or Run
  teardown. Battle construction binds an immutable equipped snapshot as the
  seventh ordered contribution component; no live Activity mutation or RNG
  occurs during assembly/execution. A stale snapshot rejects before assembly.

Timing, cardinality, calculated-versus-HP basis, damage category, shield/overkill
treatment, adjacency snapshot and summon scope are low-confidence policy, not
observed parity. Alternatives are per-action aggregation or source-unboosted
elemental damage, which would reapply target factors to an already-calculated
amount. Replace each field independently when released executable evidence or
reproducible current observations establish it. Keep the thirty-percent operand
and Hunt selector. Do not claim the source `BeforeCharacterBorn` binding proves
this policy's observation phase.

## Verification scope and remaining work

Fixtures use actual production lowering and shared battle commands in both run
families, with controlled probe attacks for multi-hit and multi-target copies,
edge/gapped formations, shielded and defeated primaries, adjacent defeats,
zero/non-Hunt/non-ordinary exclusion, fresh-factory deterministic events,
preserved builds, stale snapshots and unequip. An actual
production battle handoff additionally runs with the equipment. These are not
original Forge offers, encoded equipment-command replay or complete 13/17/20
position runs. A separate [Harmony shield](divergent-universe-weighted-curio-shield.md)
and [Automated Experience](divergent-universe-weighted-curio-attack-debuff.md)
and [The Story Presently](divergent-universe-weighted-curio-support-attack.md)
also execute. See [current repository state](state.md) for effect totals;
unsupported equipped effects fail closed.

The production workbook is authored through openpyxl and generated only by
Sora 0.6.1. Typed domain validation rejects bad joins, noncanonical/out-of-range
fractions, duplicate definitions and missing provenance/policy metadata. Current
configuration and equipment snapshots bind the authored inputs; no old-format
decoder or historical evidence is retained. No original obligation, mechanic
program, semantic family, gap or policy source receives terminal coverage credit
from this partial implementation.
