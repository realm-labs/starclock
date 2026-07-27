# Goal 07 Incremental Partition Evidence

Goal 07 is completed through ordered content partitions. Later partitions are
expected to extend the same authoritative Excel workbooks and regenerate the
same production Sora bundle. Evidence for a completed partition therefore must
remain verifiable without freezing unrelated future rows.

## Evidence model

Each completed partition owns two complementary forms of evidence:

1. A scoped semantic golden contains only the frozen IDs assigned to that
   partition and the selected Excel/Sora table rows that implement those IDs.
   Its digest excludes whole-workbook and whole-bundle hashes.
2. A completion receipt records the exact evidence files and production Sora
   bundle inspected when the partition was accepted. Each file entry contains
   its SHA-256 digest and Git blob identity.

The verifier first accepts the current working-tree file when its SHA-256
matches the receipt. If a later partition has legitimately regenerated that
shared file, the verifier reads the receipt's immutable Git blob and checks the
same SHA-256 against the historical bytes. Missing blobs, mismatched digests
and uncommitted evidence still fail closed.

This separation is intentional:

- scoped goldens prove that the partition's authored rows still mean the same
  thing in current Excel and Sora output;
- Git-backed receipt evidence proves which complete artifacts were reviewed at
  acceptance time;
- later rows may be added to shared workbooks and bundles without rewriting
  the meaning of an earlier partition;
- modifying an earlier partition's assigned rows causes its authoring check to
  drift and requires an explicit corrected receipt.

## Required checks

For every completed partition:

```text
python tools/goal07/author-<family>-partition.py --partition <id> --check
node tools/goal07/verify-content-partition.mjs --partition <id>
```

The family authoring check always reads the authoritative `.xlsx` files with
`openpyxl` and compares the assigned rows with committed Sora debug exports.
The receipt check verifies exact frozen assignment coverage, dispositions,
runtime evidence, scoped goldens and accepted artifact bytes.

Whole-workbook or whole-`config.sora` hashes must not be reintroduced into a
partition semantic golden. They belong in build/release drift gates or in the
Git-backed acceptance evidence, where their lifecycle is explicit.
