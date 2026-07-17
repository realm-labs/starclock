# Goal Manifest Freeze

`generate.mjs` binds the prepared Version 4.4 reference pack to Goal 01 and
deterministically emits the released-character, released-Light-Cone,
`standard-v1`, and Phase 7 partition manifests.

Generate the committed files:

```powershell
node tools/goal-manifest/generate.mjs
```

Verify generation drift and all structural/completeness gates:

```powershell
node tools/goal-manifest/generate.mjs --check
node tools/goal-manifest/verify.mjs
```

The generator refuses a reference pack whose digest is not
`0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a`.
Changing a selected Standard encounter, scenario, build fixture, partition, or
manifest format is therefore an explicit manifest migration that changes the
goal-manifest digest.
