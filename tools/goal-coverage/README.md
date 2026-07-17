# Goal 01 coverage tools

The Phase 0 coverage generator accounts for every frozen Goal 01 ID and checks
the human-readable counters against the machine-readable manifests. Until a
production Excel/Sora catalog exists, it deliberately reports every enabled ID
as incomplete and grants no `DataReady` or `GoldenVerified` credit.

```sh
node tools/goal-coverage/generate.mjs
node tools/goal-coverage/generate.mjs --check
node tools/goal-coverage/verify.mjs
```

The report distinguishes manifest accounting from runtime readiness. Prepared
reference records, provenance mappings and probe specifications can advance an
entry to `Cataloged`, `Documented` or `Researching`, but never to `DataReady`.
