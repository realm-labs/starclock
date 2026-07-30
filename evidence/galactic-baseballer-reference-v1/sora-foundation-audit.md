# Goal 16 Sora Foundation Audit

`G16-P3-B3` generated the isolated Sora 0.3.0 project, four schema partitions,
schema lock and four empty authoring templates for all 40 Goal 16 normalized
families. No reader, binary bundle or debug export is present; those remain
owned by `G16-P3-B4`.

## Result

| Field | Result |
|---|---|
| Sora executable | repository policy installation, `sora 0.3.0` |
| Package | `starclock_galactic_baseballer_reference` |
| Schema partitions | `profiles`, `arsenal`, `encounters`, `review` |
| Tables | 40 |
| Source ownership | 40 normalized files mapped exactly once |
| Templates | 4 workbooks / 40 sheets |
| Private keys | per-table `i32` workbook key |
| Stable identity | required `stable_key` plus unique index |
| Runtime disposition | reference-only; no lowering or production import |
| Schema lock SHA-256 | `bd2468e99125883535cec0ef4f4e6df3122d27acb939a477293de07da3ac093b` |
| Foundation tree SHA-256 | `aad564f7ef40e729a204f304fdae5bd293f2cbc6cd7debd2b857cc41bfd1511d` |

Schema source digests:

| File | SHA-256 |
|---|---|
| `project.toml` | `144372772bd12cc4ae576e99161d492ab6fcf6c9b95540b62fe0ea9bd79a3e54` |
| `profiles.toml` | `aaa794877ad2fc17ca5d2a2335290958a9be3b1089dcd418685cdb1ab551113e` |
| `arsenal.toml` | `a360e766273f739894a5a41a94c3c49bd6813f32d07c97cf13247d40ccbf76b1` |
| `encounters.toml` | `20c12994ea7eef85f01e19e78641a0a39a44fd309e460f564ad2c8859d130b8d` |
| `review.toml` | `68eff9bf9bd943b7a8ee7ad45918665e81654cdba62674a39572ebcf290d1ce2` |

Template digests:

| Workbook | SHA-256 |
|---|---|
| `GalacticBaseballerProfiles.xlsx` | `6f31007e39640355c40b03065db938927354d13538f8c64b5c2f67b5787bc0d1` |
| `GalacticBaseballerArsenal.xlsx` | `b98cb6a6b08ce5e6d8d9b7637e0985200753c7e01b594b271c5a99d201850dc9` |
| `GalacticBaseballerEncounters.xlsx` | `320c621ba7df32b31635d1c2425e4b9081b4cd18aa90b5ab457b3a03452d8042` |
| `GalacticBaseballerReview.xlsx` | `6beda6024ec1e102a6232df96f7a53a7eb8648ee83a19ee8404bd99ad9889408` |

## Determinism and compatibility

The schema lock and templates were generated independently in two clean
directories and compared byte for byte. A third clean regeneration matched
the committed foundation tree. ZIP member order, member timestamps and both
OOXML core creation/modification timestamps are canonicalized by the owning
Python generator.

`sora --serial check` accepts the complete authored workbooks. The workbook
verifier additionally proves that:

- all seven metadata rows in every authored sheet equal its Sora template;
- all 10,615 authored rows round-trip to normalized values;
- optional fields retain the Sora `optional<string>` schema instead of
  inventing placeholder values;
- all 40 stable-key indexes are unique and every schema source is isolated
  under the Goal 16 workbook root.

## Reproduction

```text
node tools/galactic-baseballer-reference/generate-sora-schema.mjs \
  --check --root "$PWD"

node tools/galactic-baseballer-reference/generate-sora-foundation.mjs \
  --root "$PWD" --output <new-output-directory> \
  --python /Users/mikai/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3

node tools/galactic-baseballer-reference/verify-sora-foundation.mjs \
  --root "$PWD" \
  --python /Users/mikai/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3
```
