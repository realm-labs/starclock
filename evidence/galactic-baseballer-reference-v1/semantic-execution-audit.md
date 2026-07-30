# Goal 16 Semantic Execution Audit

`G16-P4-B1` executes the complete ReferenceOnly review surface against the
committed Version 4.4 normalized facts. It does not enable either profile at
runtime and does not claim gameplay parity.

## Result

| Measure | Result |
|---|---:|
| Required mechanism families | 20 / 20 passed |
| ReferenceOnly rules | 26 |
| Semantic review fixtures | 35 / 35 passed |
| Failed fixtures | 0 |
| Expected-fact assertions | 162 |
| Source/precondition/input-backed assertions | 141 |
| Deterministically derived assertions | 21 |
| Explicit failure-invariance fixtures | 4 / 4 passed |

The machine result is
`semantic-fixture-results.json` with SHA-256
`e19b7751aaf1dee80293f1859fd48ef4f70db1519089a46d1e2517b754d726f3`.
Every case records its family, owning rule set, source-record resolution,
ordered-operation count, assertion counts and canonical precondition, input and
expected-fact digests.

## Execution boundary

The executor independently resolves each declared source stable ID to a
normalized fact row or a retained pre-assembly fragment. Expected primitive
facts must then be present in those resolved facts, the concrete preconditions,
the concrete input or an evidence receipt. The fixture's own expected object is
excluded from the proof index, so it cannot prove itself.

Twenty-one assertions require deterministic evaluation rather than direct
lookup. They cover:

- post-correction values intentionally not reconstructed;
- exact integer balance addition for Surprise Windfall;
- unnamed chest-probability ordinals;
- Cosmic Store balance/level validation and atomic rejection;
- Twin and Supreme prerequisite validation, consumption and rejection;
- the `38 + 2 - 40 = 0` team-experience boundary;
- labeled legal-candidate selection and empty-pool rejection; and
- stage-scoped team-bonus teardown.

Each of those results is compared to the declared expected fact after
calculation. The four rejection cases additionally prove no resource,
inventory, level, output or consumption mutation:

- insufficient Cosmic Store balance;
- missing Supreme input;
- missing Twin input; and
- no legal upgrade candidate.

## Reproduction

```text
fnm exec --using 24.15.0 node tools/galactic-baseballer-reference/execute-semantic-fixtures.mjs
fnm exec --using 24.15.0 node tools/galactic-baseballer-reference/execute-semantic-fixtures.mjs --check
```

The first command writes the complete deterministic result. The second requires
byte-identical output and fails on family/rule/fixture denominator drift,
unresolved source or evidence references, non-contiguous operations, an
unsupported mechanism family, an unproved expected fact, or a semantic
invariance violation.
