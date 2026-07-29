# Divergent Universe Equation Boundary

## Exact definition closure

The pinned Version 4.4 source contains eighty `RogueTournFormula` rows whose
selector is exactly `Tourn3`:

- eight `PathEcho`;
- thirty-two `Rare`;
- twenty-four `Epic`; and
- sixteen `Legendary`.

Every row resolves one `RogueTournFormulaDisplay` record and one
`RogueMazeBuff` record. The normalized definition retains the bilingual
MazeBuff name, category, main/sub Path type IDs, required counts, MazeBuff
identity, binding locator, display extra-effect IDs and handbook visibility.
Story payloads remain excluded.

All current Equations use Path types `121`, `122`, `124`, `125`, `126`,
`127`, `128` or `129`. Each recipe preserves the exact main and optional sub
Path counts. Expansion progress is a derived count over currently owned
Blessing identities; the effect changes between `Unexpanded` and `Expanded`
when the recipe becomes unsatisfied or satisfied. How enhanced or rewritten
Blessings contribute is deliberately deferred to `G11-P1-B4`.

## Keyword and effect closure

The direct mode table contributes twenty-five keyword rows and nine parameter
rows. Twenty-three keyword rows use a current Equation Path; two are retained
only as unselected-Path catalog evidence. Eight of the nine parameter rows
belong to a current Path.

Keyword records preserve Path type, MazeBuff IDs, listed Formula locators,
extra-effect locators and canonical decimal parameters. They are reference
contributions only and are not lowered to runtime handlers.

## Offer and transition boundary

`RogueTournFormulaRandom` contains 136 stable `RandomID` rows and no published
candidate list, weights, consumer, selection count, reroll rule or fallback.
Every normalized offer therefore has an empty candidate list and explicit
`Unspecified` fields. An ID shape or nearby Formula ID never supplies
membership.

The source does not publish complete acquire, replace, discard or no-legal
transition programs. The normalized transition rules are replaceable
`ProjectPolicy`:

1. an accepted Equation begins unexpanded and immediately recomputes progress;
2. a Blessing-set change recomputes owned Equations in stable-ID order;
3. replacement validates explicitly selected input/output IDs before commit;
4. discard removes the selected Equation and its derived state; and
5. rejection or no legal candidate preserves authoritative state.

These policies do not assert observed parity or runtime executability.
Released service programs or reproducible observations must replace the
unknown offer membership, weights, costs, timing and fallback.

## Reproduction

```text
node tools/divergent-universe-reference/import-equations.mjs \
  --source-cache <turnbasedgamedata-cache>
node tools/divergent-universe-reference/import-equations.mjs --check \
  --source-cache <turnbasedgamedata-cache>
node tools/divergent-universe-reference/verify-equations.mjs \
  --source-cache <turnbasedgamedata-cache>
```

The verifier closes all 330 manifest receipts, checks all definition/display/
MazeBuff joins, recipe and expansion pairs, category and current-Path
distributions, fail-closed random offers and state-preserving policy fallbacks.
