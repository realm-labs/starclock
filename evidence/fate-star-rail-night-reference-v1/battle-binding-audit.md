# Goal 19 Battle Binding Audit

`G19-P2-B2` normalizes 67 selector-backed shared obligations at digest
`0cd67f048abb8bbc6d689730f861f93a9a6ef0f74c8b47345fcfd7298d9674a2`:
18 BattleArea rows, thirteen unified BattleArea configurations, 23 MazeBuffs,
two BattleEvents and eleven BattleTargets.

Fifty-four typed/direct relationships are DataReady. The thirteen
BattleEvent/BattleTarget scalar matches remain `ResearchRequired`; numeric
equality alone does not establish operation meaning. P2-B5 must either prove a
typed relationship or preserve each as a nonblocking field-level policy with a
replacement condition before Candidate freeze.

```text
node tools/fate-star-rail-night-reference/shared.mjs \
  --source-cache .cache/fate-star-rail-night-sources --batch G19-P2-B2 --check
```
