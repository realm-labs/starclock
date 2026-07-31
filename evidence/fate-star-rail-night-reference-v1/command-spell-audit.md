# Goal 19 Command Spell and Resource Audit

`G19-P1-B5` normalizes 223 enabled records at digest
`1cf7ed0c3f6c7ac6a71c402d5ca7feeaa3313825dab612f16285cfb2e8dcfcc7`:
70 Command Spell/Reiju definitions, 60 Reiju affixes, 71 client/common
constant rows and 22 exact Fate Reiju configuration-program digests.

Definitions, affixes, constants and program identities stay separate. Costs,
parameters and choices remain canonical source facts; hidden resource
settlement, candidate order, reroll timing and same-boundary transitions remain
explicit fixture/replacement-condition work. Upstream programs are identified
by digest and are not copied or executed.

```text
node --max-old-space-size=4096 tools/fate-star-rail-night-reference/normalize.mjs \
  --source-cache .cache/fate-star-rail-night-sources --batch G19-P1-B5 --check
```
