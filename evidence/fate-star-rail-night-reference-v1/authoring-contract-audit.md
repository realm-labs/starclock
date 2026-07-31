# Goal 19 Authoring Contract Audit

`G19-P0-B4` binds the 1,904-obligation manifest to a canonical normalized
record envelope, fact-level evidence, field-level approximation,
source-path/locator/digest reconciliation and a closed semantic fixture fact
language.

The authoring surface contains 48 uniquely owned sheets across four complete
workbooks:

- `FateStarRailNight.xlsx`: profile, flow, progress, participant and trait data;
- `FateStarRailNightCombat.xlsx`: fights, encounters, enemies and battle facts;
- `FateStarRailNightBindings.xlsx`: Masters, Servants, Noble Phantasms,
  Command Spells, resources and lifecycle/rule bindings;
- `FateStarRailNightReview.xlsx`: sources, coverage, gaps, reconciliation,
  fixtures and pack identity.

Python openpyxl 3.1.5 owns workbook creation and inspection. Sora 0.3.0 owns
schema validation, generated readers and export. Workbooks are generated only
as complete clean targets and never patched or silently overwritten. JSON
remains normalized research/debug material and is never a runtime loading
path.

```text
node tools/fate-star-rail-night-reference/contracts.mjs
node tools/fate-star-rail-night-reference/contracts.mjs --check
```
