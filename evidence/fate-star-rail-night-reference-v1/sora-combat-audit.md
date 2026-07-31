# G19-P3-B3 — Combat Sora Table Audit

Thirteen combat tables add 849 rows for stages, battle areas, encounter
selectors, waves, ordered enemy slots, enemy variants/templates/skills/program
receipts, statuses, buffs, MazeBuffs, BattleEvents and BattleTargets.

The cumulative verifier reports 41 non-empty gameplay tables and 1,600 unique
stable keys. Derived wave/slot/program records preserve their parent source
receipts and do not enlarge the frozen obligation denominator. The thirteen
policy-bound BattleEvent/BattleTarget rows retain identity-only semantics and
are not presented as observed operations.

Focused command:

```text
fnm exec --using 24.15.0 node tools/fate-star-rail-night-reference/verify-sora-tables.mjs --root . --batch G19-P3-B3
```

Result: 41 tables / 1,600 rows, zero empty tables, duplicate stable keys or
generated-schema drift.
