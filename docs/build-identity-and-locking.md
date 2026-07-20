# Build Identity, Attribution, Presets, and Locks

Goal 01 batch `G01-P5-B5` gives the pure build compiler stable canonical
identities and verifies convenience presets before they can enter orchestration.
Build-domain identity remains upstream of combat; the resolved combatant keeps
an independent combat-only digest.

## Canonical digest scopes

`starclock-build` privately reuses the pinned and already reviewed
`sha2 = 0.11.0` package. No dependency type is exposed. Every input uses an
explicit versioned domain prefix, big-endian fixed-width integers and
length-prefixed byte strings/collections. Rust layout, map iteration, workbook
order, localized text and platform byte order are never hashed implicitly.

The four executable scopes are distinct:

- `BuildDefinitionDigest` hashes one canonical character definition (including
  stats, ability rows, Trace graph, E1-E6 and sources) or one canonical Light
  Cone definition (including stats, applicability, S1-S5 and source);
- `BuildCatalogDigest` hashes the build revision, exact compatible combat
  revision/digest, ordered definition digests and normalized named preset
  inputs;
- `CombatantBuildDigest` hashes one exact normalized build selection together
  with its build-catalog digest;
- `CombatantSpecDigest` hashes only the resolved generic combat form, level,
  base stats, combat definition bindings and generic source bindings.

Expected preset digests are verification metadata and are deliberately excluded
from the catalog digest. This avoids a recursive catalog/build hash while still
letting catalog construction compile every preset and reject a mismatched
expected digest.

## Source attribution

Character, Trace, Eidolon and Light Cone definitions carry generic
`RuleSource` values with a stable `SourceDefinitionId`, combat-owned
`SourceClass`, canonical tags and a nonzero evidence digest. Catalog validation
requires the owner-appropriate class (`Unit`, `Progression`, or `Equipment`),
strictly ordered tags and a nonzero digest.

Compilation selects only the sources that contributed to the exact build,
sorts them by generic source ID and rejects collisions. The generic
`ResolvedCombatantSpec` retains only `RuleSource` bindings. The peripheral
`BuildCompilationReport` separately maps those same IDs to
`Character`, `Trace`, `Eidolon`, or `LightCone` owners for diagnostics and
coverage. No build ID or spreadsheet coordinate crosses into combat.

## Named presets and locks

`BuildPreset` owns a stable typed ID, a nonempty unique name, one already
normalized `CombatantBuildSpec`, and an optional expected build digest. Catalog
construction sorts presets by ID, rejects duplicate IDs/names, compiles every
preset under the completed catalog and rejects invalid selections or stale
expected digests. `compile_preset` is therefore expansion followed by the same
normal compiler; it has no alternate compilation path.

Every successful `CompiledBuild` exposes a `BuildLock` containing:

- build-catalog revision and digest;
- exact selected-build digest;
- independent resolved-combatant digest.

Verification checks those scopes in that order and distinguishes catalog,
build and combatant mismatches. A changed selection creates new build and
combatant digests; a changed catalog revision or definition set invalidates the
catalog scope. A live battle still receives only the resolved combatant and
cannot hot-edit the upstream build.

## Evidence and production boundary

[`build_identity.rs`](../crates/starclock-build/tests/build_identity.rs) pins
exact SHA-256 goldens for character, Light Cone, catalog, selected build and
resolved combatant scopes. It also proves input-order invariance, selected
source attribution, direct/preset equality, expected-digest rejection, unknown
presets and stale catalog/build locks.

The committed production workbooks remain the only future authoritative
content source. This batch adds no JSON runtime path, enables no workbook row
and claims no character or Light Cone coverage; production remains 0/283
`DataReady`.

Run the focused gate with:

```text
cargo test -p starclock-build --all-targets --all-features
```
