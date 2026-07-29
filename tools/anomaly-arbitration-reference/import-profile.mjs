#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(".");
const sourceCache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/content-reference"));
const fallbackValue = option("--fallback-source-cache")
  ?? process.env.STARCLOCK_FALLBACK_SOURCE_CACHE;
const fallbackSourceCache = fallbackValue === undefined
  ? undefined
  : path.resolve(fallbackValue);
const sourceRoot = path.join(sourceCache, "turnbasedgamedata");
const fallbackRoot = fallbackSourceCache === undefined
  ? undefined
  : path.join(fallbackSourceCache, "turnbasedgamedata");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const outputRoot = path.join(
  root,
  "content-reference",
  "anomaly-arbitration-v1",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "anomaly-arbitration-v1",
  "content-manifest.json",
), "utf8"));
const paths = {
  group: "ExcelOutput/ChallengePeakGroupConfig.json",
  definition: "ExcelOutput/ChallengePeakConfig.json",
  boss: "ExcelOutput/ChallengePeakBossConfig.json",
  common: "ExcelOutput/ChallengePeakCommonConst.json",
  stage: "ExcelOutput/StageConfig.json",
  textZh: "TextMap/TextMapCHS.json",
  textEn: "TextMap/TextMapEN.json",
};

function option(name) {
  const index = args.indexOf(name);
  if (index === -1) return undefined;
  if (args[index + 1] === undefined || args[index + 1].startsWith("--"))
    throw new Error(`${name} requires a path`);
  return args[index + 1];
}

function git(repositoryRoot, gitArgs, options = {}) {
  return execFileSync("git", ["-C", repositoryRoot, ...gitArgs], {
    encoding: "utf8",
    maxBuffer: 512 * 1024 * 1024,
    ...options,
  });
}

function assertCache(repositoryRoot, label) {
  const actual = git(repositoryRoot, ["rev-parse", "HEAD"]).trim();
  if (actual !== revision)
    throw new Error(`${label} source revision mismatch: ${actual}`);
  if (git(repositoryRoot, ["status", "--porcelain"]).trim())
    throw new Error(`${label} source cache has local changes`);
}

function batchBlobs(relativePaths) {
  const ordered = [...relativePaths].sort();
  const output = execFileSync("git", [
    "-C",
    sourceRoot,
    "cat-file",
    "--batch",
  ], {
    input: `${ordered.map((relativePath) =>
      `HEAD:${relativePath}`).join("\n")}\n`,
    encoding: null,
    env: {
      ...process.env,
      GIT_NO_LAZY_FETCH: "1",
      ...(fallbackRoot === undefined
        ? {}
        : {
          GIT_ALTERNATE_OBJECT_DIRECTORIES:
            path.join(fallbackRoot, ".git", "objects"),
        }),
    },
    maxBuffer: 512 * 1024 * 1024,
  });
  const blobs = new Map();
  let offset = 0;
  for (const relativePath of ordered) {
    const headerEnd = output.indexOf(0x0a, offset);
    const header = output.subarray(offset, headerEnd).toString("utf8");
    const match = /^([0-9a-f]+) blob ([0-9]+)$/u.exec(header);
    if (match === null)
      throw new Error(`source blob unavailable: ${relativePath}: ${header}`);
    const size = Number(match[2]);
    const start = headerEnd + 1;
    const end = start + size;
    if (output[end] !== 0x0a)
      throw new Error(`source blob truncated: ${relativePath}`);
    blobs.set(relativePath, output.subarray(start, end));
    offset = end + 1;
  }
  if (offset !== output.length) throw new Error("unexpected cat-file bytes");
  return blobs;
}

function losslessJson(bytes) {
  return JSON.parse(bytes.toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu,
    '$1"$2"',
  ));
}

function digest(value) {
  return createHash("sha256")
    .update(typeof value === "string" ? value : JSON.stringify(value))
    .digest("hex");
}

function manifestRecord(categoryId, recordId) {
  const record = manifest.categories[categoryId]?.records.find(
    ({ id }) => id === recordId,
  );
  if (record === undefined)
    throw new Error(`manifest record is missing: ${categoryId}/${recordId}`);
  return record;
}

function structuredRef(categoryId, recordId, note, mechanismQuality) {
  const record = manifestRecord(categoryId, recordId);
  return {
    source_id: `turnbasedgamedata:${record.source_path}:${record.row_locator}`,
    repository_or_url:
      "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: revision,
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: "ExactStructured",
    mechanism_quality: mechanismQuality,
    note,
  };
}

function publicRef(id, url, locator, fact, evidenceQuality, mechanismQuality) {
  return {
    source_id: id,
    repository_or_url: url,
    revision_or_access_date: "accessed 2026-07-29",
    game_version: "4.4",
    path_or_page: url,
    locator,
    sha256: digest(fact),
    evidence_quality: evidenceQuality,
    mechanism_quality: mechanismQuality,
    note: fact,
  };
}

function textRef(locale, hash, value) {
  const sourcePath = locale === "zh_cn" ? paths.textZh : paths.textEn;
  return {
    source_id: `turnbasedgamedata:${sourcePath}:Hash=${hash}`,
    repository_or_url:
      "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: revision,
    game_version: "4.4",
    path_or_page: sourcePath,
    locator: `Hash=${hash}`,
    sha256: digest({ hash, value }),
    evidence_quality: "ExactStructured",
    mechanism_quality: "IdentityCrossCheck",
    note: `Exact ${locale} name for the referenced TextMap hash.`,
  };
}

function envelope({
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  ownership = "AnomalyArbitration",
  evidenceQuality,
  mechanismQuality,
  manifestRecordIds,
  sourceRefs,
  tags,
  fields,
}) {
  return {
    id,
    schema_revision: "starclock.anomaly-arbitration-row.v1",
    kind,
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    ownership,
    coverage_state: "DataReady",
    evidence_quality: evidenceQuality,
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [...manifestRecordIds].sort(),
    source_refs: sourceRefs,
    tags: [...tags].sort(),
    ...fields,
  };
}

function normalizedFile(file, kind, records) {
  return {
    schema_revision: "starclock.anomaly-arbitration-normalized-file.v1",
    goal_id: "anomaly-arbitration-reference-v1",
    profile: "anomaly-arbitration-v1",
    file,
    record_kind: kind,
    records,
  };
}

assertCache(sourceRoot, "primary");
if (fallbackRoot !== undefined) assertCache(fallbackRoot, "fallback");
const blobs = batchBlobs(Object.values(paths));
const groupRows = losslessJson(blobs.get(paths.group));
const definitionRows = losslessJson(blobs.get(paths.definition));
const bossRows = losslessJson(blobs.get(paths.boss));
const commonRows = losslessJson(blobs.get(paths.common));
const stageRows = losslessJson(blobs.get(paths.stage));
const textZh = losslessJson(blobs.get(paths.textZh));
const textEn = losslessJson(blobs.get(paths.textEn));
const group = groupRows.find(({ ID }) => ID === 8);
const definitions = definitionRows.filter(({ ID }) =>
  [801, 802, 803, 804].includes(ID));
const boss = bossRows.find(({ ID }) => ID === 804);
const stages = stageRows.filter(({ StageID }) =>
  [30508011, 30508012, 30508013, 30508021, 30508022].includes(StageID));
if (group === undefined || definitions.length !== 4
  || boss === undefined || stages.length !== 5)
  throw new Error("active profile source closure drift");
const constants = new Map(commonRows.map((row) => [
  row.ConstValueName,
  row.Value.IntValue ?? row.Value.StringValue ?? row.Value.ArrayValue,
]));

const officialGuide = publicRef(
  "official:hoyolab:anomaly-arbitration-gameplay-guide",
  "https://www.hoyolab.com/article/41091494",
  "Availability, Participation Requirement and How to Play",
  "Released official guide: permanent periodically updated mode; Equilibrium 6 plus maximum-star records in the three rotating endgame modes; three Knight stages in arbitrary order and one King stage.",
  "ExactPublicText",
  "ExactRelationship",
);
const officialFact = (id, locator, fact) => publicRef(
  `official:hoyolab:anomaly-arbitration-gameplay-guide:${id}`,
  "https://www.hoyolab.com/article/41091494",
  locator,
  fact,
  "ExactPublicText",
  "ExactRelationship",
);
const independentUnlock = publicRef(
  "public:fandom:anomaly-arbitration:king-difficulty-order",
  "https://honkai-star-rail.fandom.com/wiki/Anomaly_Arbitration",
  "King in Check",
  "Independent released cross-check: Plight is initially available and normal King unlocks after all three Knight stages are cleared.",
  "ApproximateFromReleasedText",
  "ObservedBehavior",
);
const rotationObservation = publicRef(
  "public:hoyolab:45950079",
  "https://www.hoyolab.com/article/45950079",
  "title",
  "4.4 Anomaly Arbitration Nr 8 | 2026/07/15 - 2026/08/25",
  "Observed",
  "IdentityCrossCheck",
);
const independentRotation = publicRef(
  "public:hsrtierlist:anomaly-arbitration-4-4",
  "https://hsrtierlist.net/anomaly-arbitration/4-4",
  "Version and stage overview",
  "Version 4.4 Enwreathed by the World; 2026-07-15 through 2026-08-26; Knight I-III, King and Plight.",
  "Observed",
  "IdentityCrossCheck",
);

const profile = envelope({
  id: "anomaly-arbitration-v1",
  kind: "Profile",
  nameEn: "Anomaly Arbitration",
  nameZh: "异相仲裁",
  summaryEn:
    "A permanent high-difficulty activity whose period replaces three Knight stages and one King stage.",
  summaryZh:
    "常驻高难玩法，每期更新三个骑士关与一个王棋关的资料边界。",
  evidenceQuality: "ExactPublicText",
  mechanismQuality: "ExactRelationship",
  manifestRecordIds: [
    "profiles:anomaly-arbitration-v1",
    "mode_constants:constant:ChallengePeak_Entrance",
    "mode_constants:constant:ChallengePeak_Entrance_MapInfo",
    "mode_constants:constant:ChallengePeak_Pre_Boss_Quest",
    "mode_constants:constant:ChallengePeak_Pre_GameplayGuide_Quest",
    "mode_constants:constant:ChallengePeak_Pre_Maze_Quest",
    "mode_constants:constant:ChallengePeak_Pre_Quest",
    "mode_constants:constant:ChallengePeak_Pre_Story_Quest",
    "mode_constants:constant:ChallengePeak_TutorialMissionID",
  ],
  sourceRefs: [
    officialGuide,
    structuredRef(
      "active_periods",
      "period:8",
      "Pinned active group anchors the Version 4.4 profile instance.",
      "IdentityCrossCheck",
    ),
    structuredRef(
      "mode_constants",
      "constant:ChallengePeak_Pre_Quest",
      "Structured prerequisite quest locator.",
      "ContextOnly",
    ),
    structuredRef(
      "mode_constants",
      "constant:ChallengePeak_Entrance_MapInfo",
      "Structured entrance map locator.",
      "ContextOnly",
    ),
    structuredRef(
      "mode_constants",
      "constant:ChallengePeak_Entrance",
      "Structured entrance locator.",
      "ContextOnly",
    ),
    structuredRef(
      "mode_constants",
      "constant:ChallengePeak_TutorialMissionID",
      "Structured tutorial mission locator.",
      "ContextOnly",
    ),
    ...[
      ["ChallengePeak_Pre_Maze_Quest", "Structured maze prerequisite locator."],
      ["ChallengePeak_Pre_Story_Quest", "Structured story prerequisite locator."],
      ["ChallengePeak_Pre_Boss_Quest", "Structured boss prerequisite locator."],
      [
        "ChallengePeak_Pre_GameplayGuide_Quest",
        "Structured gameplay-guide prerequisite locator.",
      ],
    ].map(([name, note]) => structuredRef(
      "mode_constants",
      `constant:${name}`,
      note,
      "ContextOnly",
    )),
  ],
  tags: ["activity", "high-difficulty", "periodic", "profile"],
  fields: {
    availability: "PermanentWithPeriodicStages",
    minimum_equilibrium_level: 6,
    participation_requirements: [
      "equilibrium-level-at-least-6",
      "maximum-stars-in-highest-memory-of-chaos-stage",
      "maximum-stars-in-highest-pure-fiction-stage",
      "maximum-stars-in-highest-apocalyptic-shadow-stage"
    ],
    requirements_may_be_completed_in_different_versions: true,
    entry_locators: {
      prerequisite_quest_id: String(constants.get("ChallengePeak_Pre_Quest")),
      entrance_map_id: String(constants.get("ChallengePeak_Entrance_MapInfo")),
      entrance_id: String(constants.get("ChallengePeak_Entrance")),
      tutorial_mission_id: String(
        constants.get("ChallengePeak_TutorialMissionID"),
      )
    },
    secondary_prerequisite_locators: {
      maze_quest_id: String(constants.get("ChallengePeak_Pre_Maze_Quest")),
      story_quest_id: String(constants.get("ChallengePeak_Pre_Story_Quest")),
      boss_quest_id: String(constants.get("ChallengePeak_Pre_Boss_Quest")),
      gameplay_guide_quest_id: String(
        constants.get("ChallengePeak_Pre_GameplayGuide_Quest"),
      ),
      disposition: "MechanicalLocatorsOnly",
    },
    active_period_id: "period.8",
    stage_ids: [
      "stage.knight-1",
      "stage.knight-2",
      "stage.knight-3",
      "stage.king-normal",
      "stage.king-plight"
    ],
    runtime_executable: false
  },
});

const period = envelope({
  id: "period.8",
  kind: "Period",
  nameEn: "Enwreathed by the World",
  nameZh: "尘世卷中",
  summaryEn:
    "Version 4.4 rotation 8 binds aliases 801-804 to four normal encounters and one Plight variant.",
  summaryZh:
    "4.4 版本第 8 期，将别名 801–804 绑定到四个常规遭遇与一个绝境变体。",
  evidenceQuality: "ApproximateFromReleasedText",
  mechanismQuality: "IdentityCrossCheck",
  manifestRecordIds: [
    "active_periods:period:8",
    "mode_constants:constant:ChallengePeak_About_To_Expire_Days",
    "mode_constants:constant:ChallengePeak_Record_Keep_Days",
    "mode_constants:constant:ChallengePeak_Record_Keep_Num",
  ],
  sourceRefs: [
    structuredRef(
      "active_periods",
      "period:8",
      "Exact group, title hash and alias relationships.",
      "ExactRelationship",
    ),
    rotationObservation,
    independentRotation,
    textRef("zh_cn", group.Title.Hash, textZh[group.Title.Hash]),
    textRef("en", group.Title.Hash, textEn[group.Title.Hash]),
    structuredRef(
      "mode_constants",
      "constant:ChallengePeak_About_To_Expire_Days",
      "Structured period-expiry warning locator.",
      "ContextOnly",
    ),
    structuredRef(
      "mode_constants",
      "constant:ChallengePeak_Record_Keep_Num",
      "Structured retained-record count.",
      "ExactRelationship",
    ),
    structuredRef(
      "mode_constants",
      "constant:ChallengePeak_Record_Keep_Days",
      "Structured retained-record day window.",
      "ContextOnly",
    ),
  ],
  tags: ["active-at-snapshot", "period", "version-4.4"],
  fields: {
    source_group_id: "8",
    active_at_snapshot: true,
    structured_access_date: "2026-07-22",
    observed_start_date: "2026-07-15",
    observed_end_dates: ["2026-08-25", "2026-08-26"],
    expiry_warning_days: Number(
      constants.get("ChallengePeak_About_To_Expire_Days"),
    ),
    retained_record_count: Number(
      constants.get("ChallengePeak_Record_Keep_Num"),
    ),
    retained_record_days: Number(
      constants.get("ChallengePeak_Record_Keep_Days"),
    ),
    alias_ids: ["801", "802", "803", "804"],
    stage_ids: [
      "30508011",
      "30508012",
      "30508013",
      "30508021",
      "30508022"
    ],
    approximations: [
      {
        field_path: "observed_end_dates",
        unavailable_fact:
          "The released structured snapshot has no timezone-aware period boundary and public pages display adjacent end dates.",
        selected_policy:
          "Preserve both source-local observed dates and make no canonical instant claim.",
        alternatives: [
          "choose the HoYoLAB display date",
          "choose the independent guide display date"
        ],
        rationale:
          "The one-day difference is consistent with locale/timezone presentation but no released offset was captured.",
        affected_fixture_ids: ["fixture.profile-entry.period-window"],
        confidence: "High",
        replacement_condition:
          "Replace with an exact timezone-aware start/end instant from released structured or official text evidence."
      }
    ],
    runtime_executable: false
  },
});

const definitionById = new Map(definitions.map((row) => [row.ID, row]));
const stageById = new Map(stages.map((row) => [row.StageID, row]));
const stageSpecs = [
  {
    id: "stage.knight-1",
    alias: 801,
    stage: 30508011,
    kind: "Knight",
    difficulty: "Normal",
    order: 1,
    peerOrder: "ArbitraryAmongKnights",
  },
  {
    id: "stage.knight-2",
    alias: 802,
    stage: 30508012,
    kind: "Knight",
    difficulty: "Normal",
    order: 2,
    peerOrder: "ArbitraryAmongKnights",
  },
  {
    id: "stage.knight-3",
    alias: 803,
    stage: 30508013,
    kind: "Knight",
    difficulty: "Normal",
    order: 3,
    peerOrder: "ArbitraryAmongKnights",
  },
  {
    id: "stage.king-normal",
    alias: 804,
    stage: 30508021,
    kind: "King",
    difficulty: "Normal",
    order: 4,
    peerOrder: "AfterThreeKnightClears",
  },
  {
    id: "stage.king-plight",
    alias: 804,
    stage: 30508022,
    kind: "King",
    difficulty: "Plight",
    order: 5,
    peerOrder: "DirectAlternative",
  },
];
const stageRecords = stageSpecs.map((spec) => {
  const definition = definitionById.get(spec.alias);
  const stage = stageById.get(spec.stage);
  const titleHash = spec.difficulty === "Plight"
    ? boss.HardTitle.Hash
    : definition.Title.Hash;
  const nameEn = textEn[titleHash];
  const nameZh = textZh[titleHash];
  if (typeof nameEn !== "string" || typeof nameZh !== "string")
    throw new Error(`stage TextMap name missing: ${spec.id}`);
  const manifestIds = [
    `stage_definitions:alias:${spec.alias}`,
    `stage_configs:stage:${spec.stage}`,
  ];
  if (spec.difficulty === "Plight")
    manifestIds.push("boss_difficulty_definitions:boss:804:plight");
  const sourceRefs = [
    structuredRef(
      "stage_definitions",
      `alias:${spec.alias}`,
      "Active group alias definition and normal objective/trait references.",
      "ExactRelationship",
    ),
    structuredRef(
      "stage_configs",
      `stage:${spec.stage}`,
      "Released StageConfig encounter parent.",
      "ExactRelationship",
    ),
  ];
  if (spec.difficulty === "Plight")
    sourceRefs.push(structuredRef(
      "boss_difficulty_definitions",
      "boss:804:plight",
      "Boss alias 804 Plight extension.",
      "ExactRelationship",
    ));
  if (spec.id === "stage.king-normal") sourceRefs.push(independentUnlock);
  if (spec.id === "stage.king-plight")
    sourceRefs.push(officialFact(
      "direct-plight-clear",
      "King in Check Stage",
      "Directly defeating the Plight difficulty King stage is accepted and counts all Knight stages as three-star clears.",
    ));
  sourceRefs.push(textRef("zh_cn", titleHash, nameZh));
  sourceRefs.push(textRef("en", titleHash, nameEn));
  sourceRefs.push(officialGuide);
  const approximations = spec.id === "stage.king-normal"
    ? [{
      field_path: "legal_order",
      unavailable_fact:
        "Official public text recommends clearing Knights before King but does not state the exact normal-difficulty unlock transition.",
      selected_policy:
        "Record normal King as following all three Knight clears for reference review; keep Plight as the direct alternative.",
      alternatives: [
        "normal King always available under full protection",
        "normal King and Plight both initially available"
      ],
      rationale:
        "Released independent observations describe normal King unlocking after all Knight clears, while the official guide only gives a recommendation.",
      affected_fixture_ids: ["fixture.stage-order.normal-king"],
      confidence: "Medium",
      replacement_condition:
        "Replace when a released structured selector or official instruction states the normal/Plight availability transition."
    }]
    : [];
  return envelope({
    id: spec.id,
    kind: "Stage",
    nameEn,
    nameZh,
    summaryEn: spec.kind === "Knight"
      ? `${nameEn} is a level ${stage.Level} Knight encounter and may be attempted in any order among the three Knight stages.`
      : `${nameEn} is the level ${stage.Level} King encounter on ${spec.difficulty} difficulty.`,
    summaryZh: spec.kind === "Knight"
      ? `${nameZh}是等级 ${stage.Level} 的骑士遭遇，可在三个骑士关之间自由选择挑战顺序。`
      : `${nameZh}是等级 ${stage.Level} 的${spec.difficulty === "Plight" ? "绝境" : "常规"}王棋遭遇。`,
    evidenceQuality: approximations.length === 0
      ? "ExactStructured"
      : "ApproximateFromReleasedText",
    mechanismQuality: approximations.length === 0
      ? "ExactRelationship"
      : "PolicyBoundary",
    manifestRecordIds: manifestIds,
    sourceRefs,
    tags: [
      spec.difficulty.toLowerCase(),
      spec.kind.toLowerCase(),
      "stage",
    ],
    fields: {
      period_id: "period.8",
      source_alias_id: String(spec.alias),
      source_stage_id: String(spec.stage),
      stage_kind: spec.kind,
      difficulty: spec.difficulty,
      display_order: spec.order,
      legal_order: spec.peerOrder,
      level: stage.Level,
      released: stage.Release,
      recommended_damage_types: definition.DamageType,
      battle_target_ids: definition.NormalTargetList.map(String),
      trait_ids: spec.difficulty === "Plight"
        ? boss.HardTagList.map(String)
        : definition.TagList.map(String),
      stage_graph_path: stage.LevelGraphPath,
      approximations,
      runtime_executable: false
    },
  });
});

const outcomeSpecs = [
  {
    id: "king-normal-clear",
    nameEn: "King normal clear",
    nameZh: "常规王棋通关",
    stageScope: "KingNormal",
    result: "Success",
    projection: "records the King result and evaluates its targets",
  },
  {
    id: "king-plight-clear",
    nameEn: "King Plight clear",
    nameZh: "绝境王棋通关",
    stageScope: "KingPlight",
    result: "Success",
    projection:
      "treats all Knight stages as three-star clears before downstream settlement",
  },
  {
    id: "knight-stage-clear",
    nameEn: "Knight stage clear",
    nameZh: "骑士关通关",
    stageScope: "Knight",
    result: "Success",
    projection: "records this Knight team and independently evaluated stars",
  },
  {
    id: "stage-attempt-failure",
    nameEn: "Stage attempt failure",
    nameZh: "关卡挑战失败",
    stageScope: "Any",
    result: "Failure",
    projection: "does not create a successful stage record",
  },
];
const outcomeRecords = outcomeSpecs.map((spec) => envelope({
  id: `outcome.${spec.id}`,
  kind: "TerminalOutcome",
  nameEn: spec.nameEn,
  nameZh: spec.nameZh,
  summaryEn: `${spec.nameEn} terminates the current stage attempt; it ${spec.projection}.`,
  summaryZh:
    `${spec.nameZh}会终止当前关卡尝试，并执行对应的记录或结算投影。`,
  evidenceQuality: "ExactPublicText",
  mechanismQuality: "ExactRelationship",
  manifestRecordIds: [`terminal_outcomes:${spec.id}`],
  sourceRefs: [officialFact(
    `terminal-${spec.id}`,
    spec.id === "stage-attempt-failure"
      ? "Cycles"
      : spec.id === "king-plight-clear"
        ? "King in Check Stage direct Plight clear"
        : "Knight Stage and King in Check Stage",
    spec.id === "stage-attempt-failure"
      ? "Exceeding the stage's own cycle limit makes the attempt fail."
      : spec.id === "king-plight-clear"
        ? "Direct Plight victory counts all Knight stages as three-star clears."
        : spec.id === "knight-stage-clear"
          ? "A cleared Knight stage records its team and independently evaluated stars."
          : "The King stage has its own clear and target evaluation result.",
  )],
  tags: ["attempt", spec.result.toLowerCase(), "terminal-outcome"],
  fields: {
    stage_scope: spec.stageScope,
    result: spec.result,
    projection_summary: spec.projection,
    detailed_projection_owner_batch: spec.id === "king-plight-clear"
      ? "G13-P1-B3"
      : "G13-P1-B6",
    runtime_executable: false
  },
}));

const files = {
  "profiles.json": normalizedFile("profiles.json", "Profile", [profile]),
  "periods.json": normalizedFile("periods.json", "Period", [period]),
  "stages.json": normalizedFile("stages.json", "Stage", stageRecords),
  "terminal-outcomes.json": normalizedFile(
    "terminal-outcomes.json",
    "TerminalOutcome",
    outcomeRecords,
  ),
};
for (const [file, payload] of Object.entries(files)) {
  const encoded = `${JSON.stringify(payload, null, 2)}\n`;
  const target = path.join(outputRoot, file);
  if (check) {
    const committed = await readFile(target, "utf8");
    if (committed !== encoded)
      throw new Error(`normalized profile generated drift: ${file}`);
  } else {
    await mkdir(outputRoot, { recursive: true });
    await writeFile(target, encoded, "utf8");
  }
}
console.log(
  `Anomaly Arbitration profile ${check ? "verified" : "generated"}: ` +
  "1 profile, 1 period, 5 stages, 4 terminal outcomes.",
);
