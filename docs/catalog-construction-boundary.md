# Catalog construction boundary

Goal 01 batch `G01-P1-B11` closes the reproducible-data-foundation phase by
loading real Sora bundles into immutable Starclock-owned definitions. It does
not preempt the final combat catalog/index work owned by `G01-P2-B2`.

## Production path

`starclock_data::catalog::load` accepts bundle bytes, uses only the private
generated Sora reader, validates the singleton compatibility manifest and
shared identity/provenance/evidence rows, then converts reviewed executable
tables. The returned `Arc<SimulationCatalog>` exposes Starclock-owned manifest
metadata and deterministic counts. Generated rows and preliminary definition
storage remain private.

The current production bundle contains 283 released but disabled identities and
no executable rows, so it produces a valid empty-domain catalog. Loading also
enforces these boundaries:

- generated transport IDs become positive fixed-width domain IDs;
- canonical decimals are parsed directly to signed millionths without floats;
- released `DataReady`/`GoldenVerified` state is exactly the enabled state;
- production bundles reject synthetic/project-fixture labels;
- every identity has bilingual metadata, sources and an evidence binding;
- only tables with a reviewed lowering may be populated; an unknown populated
  table fails explicitly instead of being silently ignored;
- disabled production identities cannot carry executable rows.

The Phase 1 lowering currently covers the representative character/ability/
ordered-hit-plan slice. Later schema-family batches extend the same exhaustive
boundary before they enable content.

## Representative golden

`config/catalog-fixtures/representative` contains 13 small TSV fixture tables
and a committed real `.sora` bundle. TSV is only deterministic input to the
standalone workbook materializer; it is neither production authoring input nor
a runtime path. `tools/config-catalog/verify.mjs` creates all 80 workbooks from
the committed Sora templates in two ignored clean roots, exports both through
pinned Sora 0.3.0, and requires identical binary and diagnostic results.

The bundle contains three disabled `ProjectFixture` identities and lowers one
character, one Basic ability and one one-hit plan. Crate tests prove its exact
IDs, fixed-point values, stat boundaries and references, while the public
production loader proves that the same fixture bundle is rejected.

Run:

```powershell
node tools/config-production/verify.mjs
node tools/config-catalog/verify.mjs
cargo test -p starclock-data --all-features
```

The representative bundle SHA-256 is recorded in its `golden.json`. Production
clean regeneration remains governed by `config/production-golden.json`; both
checks are part of the repository generated-drift gate.
