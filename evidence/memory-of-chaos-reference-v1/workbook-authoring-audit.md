# Goal 17 workbook authoring audit

## Result

- Authoring authority: Python `openpyxl==3.1.5`
- Workbooks: 3/3 complete
- Sora primary sheets: 27/27
- Authored rows: 1,521
- Semantic digest: `eb944598c653ebdbb2051cfa8d3f584d24bc550a152df0f90d66679fa2e02189`
- Formula/error cells: 0
- Excel text overflow cells: 0
- No-overwrite policy: verified
- Independent generation: byte-identical
- Runtime publication: forbidden

## Workbook identities

| Workbook | SHA-256 |
|---|---|
| `MemoryOfChaos.xlsx` | `776c55b71b13c4d7fec06cfe35cbfa4c3da14e59332e2df67d41cbf417bfb2c9` |
| `MemoryOfChaosBindings.xlsx` | `ba3ba3775df3c6861947dca0eefeae31024b76117fa3bfcf41b3e36dca9b7084` |
| `MemoryOfChaosReview.xlsx` | `988945d56061829a7be6b8409586bfc72e3f5aea8ce0b232f51e181ee445a13e` |

The workbooks retain the Sora rows 1–7 unchanged, author data from row 8,
preserve numeric private keys and typed references, and carry the complete
canonical normalized row in `payload_json`. Filters, frozen panes, hidden
gridlines, bounded widths, wrapping, validation and Candidate-policy
highlighting are presentation-only authoring affordances.

## Sora smoke export

Pinned Sora 0.3.0 exported all three workbooks to a temporary binary bundle:

- bytes: 3,313,323
- SHA-256: `2d95f797ad87abc168eec20913f9007672bfab1ef4bbce4b57b5afe482bd505f`

The temporary smoke bundle is not release evidence; deterministic committed
binary/debug export and generated-reader loading belong to `G17-P3-B6`.
