# Light Cone Compilation

Goal 01 batch `G01-P5-B4` extends the pure `starclock-build` boundary with exact
Light Cone loadouts. The definitions and selections remain upstream of combat;
successful compilation exposes only generic base-stat and definition bindings
owned by `starclock-combat`.

## Exact selection and catalog shape

`LightConeLevel` accepts levels 1 through 80, `Superimposition` accepts S1
through S5, and `LightConeLoadout` names one definition, level, promotion and
rank. Each definition owns unique exact `(level, promotion)` HP/ATK/DEF rows.
Missing rows fail instead of interpolating, clamping or reusing another
promotion boundary.

Every catalog definition requires exactly five passive rows. Catalog
construction sorts stat and passive rows, rejects duplicate stat keys and
missing S-ranks, and checks every passive rule/modifier reference against the
captured immutable combat catalog. S-ranks are alternatives for one passive:
selecting S5 applies the S5 row only, not S1 through S5 cumulatively.

## Applicability

Base-stat composition and passive activation are separate operations:

- `MatchingPath` activates the selected passive row only when the wearer and
  cone paths match;
- `Always` activates it for every wearer path;
- `BaseStatsOnly` never activates it.

All three policies apply the equipped cone's exact HP/ATK/DEF row. A path
mismatch therefore remains a valid equipped fixture with base stats and no
passive; it is not silently rejected and does not silently activate.

The B4 executable passive subset adds combat rule bundles and modifiers in the
authored row order. Duplicate bindings and conflicts with the already compiled
character/Trace/Eidolon workspace fail at `LightConeSelection` without a
partial result. Content-specific path or item branches are absent.

## Combat boundary and evidence

The compiler adds the exact HP contribution with checked integer arithmetic and
the exact fixed-point ATK/DEF contributions with checked six-place arithmetic.
`ResolvedCombatantSpec` carries those generic values and the canonical resolved
rule/modifier bindings; it contains no Light Cone ID, level, path,
applicability or superimposition type.

[`light_cone_compilation.rs`](../crates/starclock-build/tests/light_cone_compilation.rs)
proves S1/S5 selection, order-independent catalog input, matching/invalid/always
wearer policies, exact stat composition, missing rows, invalid bounds, duplicate
definitions, incomplete S-ranks, unresolved passive references and fixed
binding conflicts.

The committed production workbooks remain the only future authoritative
content source. This batch adds no JSON runtime path, enables no workbook row
and claims no Light Cone coverage; production remains 0/283 `DataReady`.

Run the focused gate with:

```text
cargo test -p starclock-build --all-targets --all-features
```
