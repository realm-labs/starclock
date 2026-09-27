# Divergent Universe Hex content taxonomy

The 17 current `RogueTournHex.TournMode = Tourn3` definitions represent **Weighted
Curios**, not Grand Miracles. 名称 `Hex` 对应加权奇物，不能据此建立宏大奇迹运行时。
The two gameplay families remain separate requirements of
[the complete runtime goal](goals/22-divergent-universe-runtime.md).

## Released evidence

The released Version 4.4 source is pinned to
`fd978d6ef09f941fba644c731ab54abd6f7c3568` in
[turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata.git), accessed
2026-09-27. The current generated
[taxonomy inventory](../content-manifests/divergent-universe-runtime-v1/grand-miracle-gamble-execution.json)
binds exact file SHA-256 values and the following locators:

| Source | Locator | Independent interpretation |
| --- | --- | --- |
| `ExcelOutput/RogueTournHex.json` | `TournMode=Tourn3`, 17 rows | Current Hex reference definitions |
| `ExcelOutput/RogueTournWorkbenchFunc.json` | `FuncID=11`, `FuncType=HexEquipment` | Service equips/adjusts Weighted Curios |
| `TextMap/TextMapEN.json` | title `1823313480930548529`, description `14848339980219349693` | Released service text establishes the meaning of Hex |

This proof does not infer membership from matching names or adjacent IDs.
The 235 ordinary Curio state rows and their generic weighted sampling membership
are not the Weighted Curio family. Likewise, neither Persona gift quality nor an
unproven selector supplies actual Grand Miracle membership.

## Current implementation boundary

`DivergentUniverseHexContentKind::WeightedCurio` is derived at the owned Sora
conversion boundary. Catalog validation rejects treating these references as
`GrandMiracle`; production-bundle tests cover all 17 rows and the rejected
counterfactual. This semantic field is not a new author-authored gameplay fact.

The former flag-only Grand Miracle install/activate/teardown API is absent. Its
reference flags did not execute combat effects, and are not retained as a
compatibility implementation. Slot 39 remains reserved and empty in the baseline
graph, without a loadout or effect API; its physical layout has not changed.

The existing reference table, field and stable-key names still say Grand Miracle.
They are misclassified transport metadata, not runtime admission. Aligning the
reference importer, production workbooks, schema, generated readers, debug export
and real Sora bundle is pending and must travel together through the documented
openpyxl/Sora authoring path. This change does not claim that authoring alignment.
All source obligations and reference denominators are retained; no completeness
credit is added or removed on the basis of names alone.

Weighted Curio obtaining/equipping, eligibility consumers, loadout limits, actual
effects and Forge card enhancement are not implemented. Actual Grand Miracle
selectors, acquisition and effects require separate evidence and implementation.
The current Forge level-one source card must not be wired to the removed API or
to Respite blessing enhancement. Higher-level Tawot service tests are not proof
of a reachable level-one Forge payload.

The two accepted Gamble Coin units continue to credit their exact fragment
amounts. All 126 unresolved groups and 87 unresolved units remain fail-closed,
preserving authoritative state and RNG. Native tests verify those behaviors;
the inventory generator checks test-target existence and never emits a pass
receipt or promotes 575 catalog/reference obligations to terminal runtime coverage.
Other audit producers are not granted a completion prerequisite by this inventory.
In particular, the Titan execution producer currently rejects its obligation
closure before regeneration; its existing report is not fresh behavioral proof.
Repairing that separate audit and its downstream digest closure remains required
before the full release gates can pass.
The current diagnostic command is
`node tools/divergent-universe-runtime/generate-titan-runtime-execution.mjs`;
it returns `Error: P5-B3 obligation closure drift`. The scoped taxonomy inventory
checks and native Cargo tests do not substitute for that unresolved release audit.

## Verification

From the repository root, using the pinned Node authoring toolchain:

```text
node tools/divergent-universe-runtime/generate-grand-miracle-gamble-execution.mjs --check
node tools/divergent-universe-runtime/verify-grand-miracle-gamble-execution.mjs --check-source
cargo test -p starclock-data production_hex_source_references
cargo test -p starclock-mode-universe gamble_
```

The source check requires the pinned released source cache and verifies its exact
digests, selector and HexEquipment text locators. The default inventory drift
check uses committed reference files and Rust source, not a network lookup. Rust
behavioral verification is run directly by Cargo, never orchestrated by Node.
