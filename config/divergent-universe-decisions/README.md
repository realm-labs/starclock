# Typed Divergent Universe decisions

This project owns executable decision definitions, not the reference pack's
identity/evidence catalog. Its 29 tables and 352 rows include policy-bound
choices, rewards and reviewed Curio components used by the production Activity
graph, plus nine explicitly selected source decks and their 125 distinct card
instances. Deck compilation is available, but automatic mask selection and
production domain routing remain unbound. This is partial executable coverage,
not complete gameplay parity.
The reference project remains a required current input. This is not a second
Activity engine or an old-format compatibility path.

- Authoring surface: `data/DivergentUniverseDecisions.xlsx`.
- Schema owner: `tools/divergent-universe-runtime/generate-decision-schema.mjs`.
- Bootstrap author: `tools/divergent-universe-runtime/author_decision_workbook.py`.
- Only schema/export authority: pinned Sora 0.6.1.
- Runtime input: `config/divergent-universe-decisions-generated/config.sora`.
- Private reader/domain owner: `starclock-data::divergent_universe_decisions`.
- Policy, evidence and unfinished execution requirements:
  [occurrence rewards](../../docs/divergent-universe-occurrence-rewards.md).

Source references and policy are separate tables. Choices carry typed costs;
ordered child rows carry reward kind, integer count/amount and optional rarity
bounds. There are no raw programs, scripts or `payload_json` fields.

Generate the schema with the pinned Node executable, then use Sora's
`schema-lock` and `excel-template` commands. The bootstrap author requires an
explicit new `--output` workbook and refuses existing targets. Existing
designer edits must be preserved; do not reauthor over them implicitly.
Any authoring change must be accompanied by domain fixtures and updated
bootstrap inputs in the same focused change.

Build the current input with:

```text
.cache/tools/sora-cli-0.6.1/bin/sora.exe --serial build --project config/divergent-universe-decisions.toml
```

Verify schema/generator, reader, export and fresh-authoring drift with:

```powershell
$env:STARCLOCK_PYTHON = (Resolve-Path .cache/g22-python/Scripts/python.exe).Path
.cache/tools/node-v24.15.0-win-x64/node.exe tools/divergent-universe-runtime/verify-decision-workbook.mjs
```

The verifier writes fresh artifacts into a unique temporary directory and
retains it for inspection. It does not overwrite or delete production inputs.
JSON debug exports are diagnostics only; runtime loading accepts binary Sora.

Run the domain checks with Cargo directly:

```text
cargo test -p starclock-data --lib divergent_universe_decisions
```

These checks prove data correctness and reproducibility, not original-game
parity or event execution. Gameplay acceptance must include real choices,
costs, random selection, acquisition effects and room completion after this
catalog is bound into the Activity configuration digest.
The factory now performs that identity binding, and the baseline replay binds
the same decision/reference inputs in its ModeContent component. These identity
checks alone do not satisfy the still-pending complete-gameplay acceptance
requirements. Current domain-entry allowance and discard behavior is documented
in the [Curio expiry contract](../../docs/divergent-universe-curio-domain-expiry.md).
Battle effects and their separate result-counted lifetimes are documented in
[Curio battle stats](../../docs/divergent-universe-curio-battle-stats.md) and
[Curio battle reactions](../../docs/divergent-universe-curio-battle-reactions.md).
The [Tawot service definitions](../../docs/divergent-universe-tawot-service.md)
drive an explicitly admitted shared Activity purchase graph; automatic Forge
placement and optional-entry adapter/replay configuration remain pending.
