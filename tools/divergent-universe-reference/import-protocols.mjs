#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  decimal,
  writeOrCheck,
} from "./lib/common.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root")
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const context = await createContext(root, valueAfter("--source-cache"));
const outputs = new Map();

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function ordered(rows) {
  return rows.sort((left, right) =>
    left.id < right.id ? -1 : left.id > right.id ? 1 : 0);
}

const effectEntries = await context.table("RogueTournDivisionEffect");
const protocolPrograms = new Map(Object.entries({
  1: {
    enemy: { attack: "0.32", max_hp: "2", speed: "0.16" },
    difficulty: [
      "IncreaseDomainCount",
      "EnableSecondPlaneConversionDomains",
      "IncreaseEquationRandomness",
    ],
  },
  2: {
    enemy: { attack: "0.64", max_hp: "6", speed: "0.32" },
    difficulty: [
      "IncreaseMaskWishpowerRequirement",
      "IncreaseEquationBlessingRequirement",
    ],
  },
  3: {
    enemy: { attack: "0.96", max_hp: "12", speed: "0.48" },
    difficulty: [
      "ReplaceFirstAndSecondPlaneBosses",
      "AdvanceBerserkOnset",
      "IncreaseBerserkStackRate",
    ],
  },
  4: {
    enemy: { attack: "1.28", max_hp: "20", speed: "0.64" },
    difficulty: ["FurtherIncreaseDomainCount"],
  },
  5: {
    enemy: { attack: "1.6", max_hp: "32", speed: "0.8" },
    difficulty: [
      "GrantOneGrandMiracleAtFirstPlaneEntry",
      "IncreaseMaskWishpowerRequirement",
      "IncreaseEquationBlessingRequirement",
    ],
  },
  6: {
    enemy: {
      attack: "1.92",
      max_hp: "48",
      speed: "0.96",
      max_toughness: "0.2",
    },
    difficulty: [
      "AdvanceBerserkEnemyAfterAttacked",
      "IncreaseAllyDamageAfterBerserk",
      "DecreaseAllyHealingAfterBerserk",
      "DecreaseAllyShieldAfterBerserk",
    ],
  },
  7: {
    enemy: {
      attack: "2.24",
      max_hp: "70",
      speed: "1.12",
      max_toughness: "0.3",
    },
    difficulty: [
      "IncreaseStorePriceBy0.25",
      "GrantTwoRandomLevelOneDomainsAtFirstPlaneEntry",
    ],
  },
  8: {
    enemy: {
      attack: "2.56",
      max_hp: "100",
      speed: "1.28",
      max_toughness: "0.5",
    },
    difficulty: [
      "GrantRandomSpecialAbsoluteFailurePrescriptionAtFirstPlaneEntry",
    ],
  },
}));

const protocols = effectEntries.map((entry) => {
  const level = entry.row.DivisionLevel;
  const program = protocolPrograms.get(String(level));
  if (!program) throw new Error(`missing Protocol program ${level}`);
  const sourceParameters = entry.row.DescParamList.map(decimal);
  const normalizedParameters = [
    program.enemy.attack,
    program.enemy.max_hp,
    program.enemy.speed,
    ...(program.enemy.max_toughness
      ? [program.enemy.max_toughness]
      : []),
    ...(level === 7 ? ["0.25", "2"] : []),
  ];
  if (JSON.stringify(sourceParameters) !== JSON.stringify(normalizedParameters))
    throw new Error(`Protocol ${level} parameter mapping drift`);
  return {
    ...context.envelope({
      id: `divergent-universe.protocol.${level}`,
      kind: "DivergentUniverseProtocol",
      nameEn: `Threshold Protocol ${level}`,
      nameZh: `阈值协议 ${level}`,
      summaryEn:
        `Protocol ${level} applies ${Object.keys(program.enemy).length} plane-scaled enemy maximum modifier(s) and ${program.difficulty.length} additional rule(s).`,
      summaryZh:
        `协议 ${level} 应用 ${Object.keys(program.enemy).length} 个随位面缩放的敌方最大修正和 ${program.difficulty.length} 个附加规则。`,
      sourceRefs: [context.sourceRef(entry)],
      tags: ["threshold-protocol", `level-${level}`],
    }),
    source_id: String(level),
    protocol_level: level,
    entry_rules: level === 5
      ? ["GrantOneGrandMiracleAtFirstPlaneEntry"]
      : level === 7
        ? ["GrantTwoRandomLevelOneDomainsAtFirstPlaneEntry"]
        : level === 8
          ? ["GrantRandomSpecialAbsoluteFailurePrescriptionAtFirstPlaneEntry"]
          : [],
    difficulty_changes: program.difficulty,
    enemy_changes: {
      plane_scaled_maximum_increase: program.enemy,
      first_second_plane_boss_identity: level === 3
        ? "ChangedIdentityDeferredToP2B5"
        : "UnchangedByThisProtocolText",
      berserk_changes: level === 3
        ? ["EarlierOnset", "FasterStacking"]
        : level === 6
          ? ["AdvanceAfterAttacked"]
          : [],
    },
    source_parameters: sourceParameters,
    runtime_lowered: false,
  };
});
outputs.set("protocols.json", ordered(protocols));

const divisionEntries = await context.table("RogueTournDivision");
const effectByLevel = new Map(effectEntries.map((entry) =>
  [String(entry.row.DivisionLevel), entry]));
const retentionByLevel = new Map([
  [1, "NoPublishedRetentionHint"],
  [2, "NoPublishedRetentionHint"],
  [3, "NeverExtinguish"],
  [4, "NeverExtinguish"],
  [5, "RetainAfterFirstPlaneClear"],
  [6, "RetainAfterFirstPlaneClear"],
  [7, "RetainAfterSecondPlaneClear"],
  [8, "NoPublishedRetentionHint"],
  [9, "TerminalDivision"],
]);
const divisions = divisionEntries.map((entry) => {
  const level = entry.row.DivisionLevel;
  const effect = effectByLevel.get(String(level));
  const nameEn = context.text(entry.row.DivisionName, "en")
    || `Astronomical Division ${level}`;
  const nameZh = context.text(entry.row.DivisionName, "zh_cn")
    || `天体差分 ${level}`;
  return {
    ...context.envelope({
      id: `divergent-universe.astronomical-division.${level}`,
      kind: "DivergentUniverseAstronomicalDivision",
      nameEn,
      nameZh,
      summaryEn:
        `Division ${level} uses progress boundary ${entry.row.DivisionProgress ?? "terminal"} and ${effect ? `Protocol ${level}` : "no published Protocol effect row"}.`,
      summaryZh:
        `差分等级 ${level} 使用进度边界 ${entry.row.DivisionProgress ?? "终点"}，${effect ? `对应协议 ${level}` : "没有公开的协议效果行"}。`,
      sourceRefs: [
        context.sourceRef(entry),
        ...(effect ? [context.sourceRef(effect)] : []),
      ],
      tags: [
        "astronomical-division",
        `level-${level}`,
        ...(level === 9 ? ["terminal"] : []),
      ],
    }),
    source_id: String(level),
    division_level: level,
    progress_boundary: entry.row.DivisionProgress === undefined
      ? "Terminal"
      : String(entry.row.DivisionProgress),
    effect_ids: effect
      ? [`divergent-universe.protocol.${level}`]
      : [],
    cognoculi_retention: retentionByLevel.get(level),
    runtime_lowered: false,
  };
});
outputs.set("astronomical-divisions.json", ordered(divisions));

const currentRulesText = context.textEntry(
  "8721319055470494228",
  "en",
);
const currentRulesTextZh = context.textEntry(
  "8721319055470494228",
  "zh_cn",
);
const ruleRefs = [
  context.sourceRef(currentRulesText, "ExactOfficialText"),
  context.sourceRef(currentRulesTextZh, "ExactOfficialText"),
];
const modeRows = [
  {
    id: "star-pioneer",
    nameEn: "Star-Pioneer Mode",
    nameZh: "开拓模式",
    entryRules: [
      "UnlockAfterOrdinaryDifficulty5",
      "ProtocolLevelEqualsCurrentAstronomicalDivision",
      "DifficultyCannotBeLowered",
    ],
    availableContent: ["CurrentDivisionThresholdProtocol"],
    resetRules: [
      "SuccessfulFinalizationLightsCognoculi",
      "UnsuccessfulFinalizationMayExtinguishCognoculi",
      "AstronomicalDivisionNeverDecreases",
      "DivisionRetentionHintsApply",
    ],
  },
  {
    id: "practice",
    nameEn: "Practice Mode",
    nameZh: "演练模式",
    entryRules: [
      "UnlockAfterOrdinaryDifficulty5",
      "ChooseAnyProtocolUpToCurrentDivisionMaximum",
    ],
    availableContent: ["UnlockedThresholdProtocolsWithinDivisionCap"],
    resetRules: [
      "DoesNotChangeAstronomicalDivision",
      "DoesNotChangeCognoculi",
    ],
  },
];
const modeRules = modeRows.map((row) => ({
  ...context.envelope({
    id: `divergent-universe.astronomical-mode.${row.id}`,
    kind: "DivergentUniverseStarPioneerPractice",
    nameEn: row.nameEn,
    nameZh: row.nameZh,
    summaryEn:
      `${row.nameEn} has ${row.entryRules.length} entry rule(s) and ${row.resetRules.length} progression rule(s).`,
    summaryZh:
      `${row.nameZh} 具有 ${row.entryRules.length} 条进入规则和 ${row.resetRules.length} 条进度规则。`,
    sourceRefs: ruleRefs,
    tags: ["astronomical-division", row.id],
  }),
  mode_kind: row.id === "star-pioneer" ? "StarPioneer" : "Practice",
  entry_rules: row.entryRules,
  available_content: row.availableContent,
  reset_rules: row.resetRules,
  runtime_lowered: false,
}));
outputs.set("star-pioneer-practice.json", ordered(modeRules));

const cognoculi = divisionEntries.map((entry) => {
  const level = entry.row.DivisionLevel;
  const retention = retentionByLevel.get(level);
  return {
    ...context.envelope({
      id: `divergent-universe.cognoculi.division.${level}`,
      kind: "DivergentUniverseCognoculi",
      nameEn: `Division ${level} Cognoculi boundary`,
      nameZh: `差分等级 ${level} 认知值边界`,
      summaryEn:
        `Star-Pioneer Cognoculi at Division ${level} use the exact retention boundary ${retention}.`,
      summaryZh:
        `开拓模式在差分等级 ${level} 使用精确保留边界 ${retention}。`,
      coverageState: retention === "NoPublishedRetentionHint"
        ? "Researched"
        : "DataReady",
      evidenceQuality: retention === "NoPublishedRetentionHint"
        ? "ProjectPolicy"
        : "ExactOfficialText",
      sourceRefs: [
        context.sourceRef(entry),
        ...ruleRefs,
      ],
      tags: [
        "cognoculi",
        `division-${level}`,
        ...(retention === "NoPublishedRetentionHint"
          ? ["retention-unspecified"]
          : []),
      ],
    }),
    source_locator: `ExcelOutput/RogueTournDivision.json#${entry.locator}`,
    division_id:
      `divergent-universe.astronomical-division.${level}`,
    effect_scope: "StarPioneerProgression",
    contribution_ids: [
      `divergent-universe.astronomical-division.${level}`,
    ],
    gain: "SuccessfulFinalizationLightsCognoculi",
    loss: "UnsuccessfulFinalizationMayExtinguishCognoculi",
    retention,
    division_floor: "CurrentDivisionNeverDecreases",
    runtime_lowered: false,
  };
});
outputs.set("cognoculi.json", ordered(cognoculi));

await writeOrCheck(context, outputs, check);
if (!check)
  console.log(
    `Wrote ${[...outputs.values()].flat().length} Protocol/Division rows.`,
  );
