# Currency Wars core Sora schema

Batch `G12-P3-B1` creates the isolated
`config/currency-wars/project.toml` project and the first 22 source-shaped
tables. The tables cover:

- profile, Gambit, module and entry boundaries;
- area, difficulty, three-Plane layer, room and Node topology;
- finish and stage-flow rules;
- Squad HP, action-value, battle-result and run-failure boundaries; and
- roster identity, economy, offers, transactions and team-size states.

Every table has a private positive `i32` workbook key and a unique stable-key
index. Common bilingual summaries, ownership, coverage and evidence fields are
typed columns. Repeated/nested domain fields remain canonical JSON strings at
this authoring boundary so no unreviewed runtime operation is inferred.

The project reads only `CurrencyWars.xlsx`, writes only below
`config/currency-wars-generated/` and has no dependency on the production
`config/project.toml` or `config/generated/` bundle.

The repository installer reproduced the checksum-bound Sora 0.3.0 binary in
the ignored local tool cache. The Goal verifier resolves that exact binary,
checks `sora --version`, regenerates the schema byte-for-byte and runs:

```text
.cache/tools/sora-cli-0.3.0/bin/sora --serial check \
  --project config/currency-wars/project.toml
```

All 22 tables and sheet-name constraints pass under Sora 0.3.0. The unrelated
PATH-level Sora 0.2.0 binary is not accepted by the verifier.
