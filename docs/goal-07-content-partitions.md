# Goal 07 Content Partitions

`G07-P0-B3` expands the fifteen inherited content milestones into 104 ordered,
atomic `Snn` batches. Together with the seventeen fixed batches, Goal 07 has a
frozen denominator of 121 batch commits.

## Partition limits

- at most 16 mechanic rules per combat or service batch;
- at most 32 effect-bearing noncombat records per occurrence/service batch;
- at most 12 ordinary enemy variants per enemy batch;
- one Boss variant per Boss batch;
- at most 96 map-node metadata rows, with each logical map kept whole;
- no native handler is admitted by partition generation.

The generated manifest assigns all 2,201 records, 786 rules, 78 fixtures, 86
enemy variants and 173 encounter members exactly once. Each batch owns its
Excel/openpyxl changes, Sora export, runtime lowering, production fixture,
provenance and terminal coverage update.

The authoritative expanded ledger is
`docs/goals/07-standard-universe-mechanics-content-ledger.md`. It is generated
from `content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json`;
manual row editing is forbidden.

## Batch distribution

| Family | Generated batches |
|---|---:|
| Ability Tree and shared Activity | 3 |
| Nine Paths | 36 |
| Curios | 9 |
| Occurrences | 14 |
| Services | 6 |
| Enemies, encounters, topology and difficulty | 36 |
| Total | 104 |
