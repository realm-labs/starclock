# Version 4.4 active-period selection

## Conclusion

The Version 4.4 reference snapshot selects Anomaly Arbitration group `8`,
“尘世卷中” / “Enwreathed by the World.” This is not inferred from the maximum
ID. Two released public observations identify the Version 4.4 rotation and
title; the pinned structured row then supplies the exact selector chain.

## Released cross-checks

- [HoYoLAB community record: 4.4 Anomaly Arbitration Nr 8](https://www.hoyolab.com/article/45950079),
  accessed 2026-07-29. The title identifies Version 4.4, rotation 8 and its
  public activity window. Quality: `Observed`.
- [Independent Version 4.4 stage guide](https://hsrtierlist.net/anomaly-arbitration/4-4),
  accessed 2026-07-29. The page identifies “Enwreathed by the World,” Knight
  I–III, King in Check and King in Check: Plight. Quality: `Observed`.

Neither page is used for authoritative numeric mechanics. All active IDs,
parameters and source membership come from the pinned released structured
snapshot.

## Structured selector chain

| Source path | Stable row locator | Selected relationship |
|---|---|---|
| `ExcelOutput/ChallengePeakGroupConfig.json` | `row=7;ID=8` | `PreLevelIDList=[801,802,803]`, `BossLevelID=804`; title hash resolves to the released bilingual title. |
| `ExcelOutput/ChallengePeakConfig.json` | `ID=801..804`, four individually receipted rows | Selects four normal stage IDs, seven battle targets and six normal trait MazeBuffs. |
| `ExcelOutput/ChallengePeakBossConfig.json` | `ID=804` | Selects the Plight stage, hard target, two Plight traits and three King buff options. |
| `ExcelOutput/StageConfig.json` | `StageID=30508011,30508012,30508013,30508021,30508022` | All five rows are released and select ordered encounter entries plus battle events `30502`–`30504`. |

Every row has its own SHA-256 evidence digest in
`content-manifests/anomaly-arbitration-v1/content-manifest.json`.
Groups `1`–`7`, their 28 aliases, seven boss extensions and 35 StageConfig
rows are separately receipted as `ExcludedHistoricalPeriod`.

## Empty-pool boundary

The generated selector closure follows only fields explicitly reached from
group `8`: aliases, normal/Plight stages, targets, MazeBuffs, battle events,
mechanical constants, enemy variants/templates/skills/statuses and selected
configuration symbols. It finds no Blessing, Curio, Occurrence, gameplay
service, currency or random content-pool selector.

The `ChallengePeak_Shop` constant and reward tables are account-facing
locators and remain `EvidenceOnly`; they do not establish a mechanically
reachable service. Each zero proof carries a digest and a replacement
condition requiring a stronger released active selector.
