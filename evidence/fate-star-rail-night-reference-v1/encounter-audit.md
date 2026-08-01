# Goal 19 Encounter Audit

`G19-P2-B3` expands eight `StageType=FateActivity` manifest obligations into
112 normalized encounter rows at digest
`b8db101d46245a4915fde7f515fca88d30f3fbe6cbf3ffad641c15ebac8124e5`:
eight stages, 24 ordered waves and eighty ordered enemy slots.

Wave and slot records are deterministic children of the eight source
obligations and therefore do not enlarge the exact-once denominator. Stage
level, conditions and source ordering remain canonical. Every slot retains the
exact source enemy-variant ID; P2-B4 closes variants, templates and skills.

```text
node tools/fate-star-rail-night-reference/shared.mjs \
  --source-cache .cache/fate-star-rail-night-sources --batch G19-P2-B3 --check
```
