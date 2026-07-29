#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const output = path.join(
  root,
  "content-reference/anomaly-arbitration-v1/battle-events.json",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";

function digest(value) {
  return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

function structuredRef(category, id, note) {
  const record = manifest.categories[category].records.find(
    (candidate) => candidate.id === id,
  );
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
    mechanism_quality: "ExactRelationship",
    note,
  };
}

function textRef(locale, hash, value) {
  const sourcePath = locale === "zh_cn"
    ? "TextMap/TextMapCHS.json"
    : "TextMap/TextMapEN.json";
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
    note: `Exact ${locale} released action-bar text.`,
  };
}

const descriptionEn =
  "Starting from Cycle 3, at the beginning of each Cycle, all allies enter the \"Middlegame Mayhem\" state, which greatly increases their final DMG dealt. This effect can be stacked.";
const descriptionZh =
  "从第3轮起，每个轮次开始时，我方全体进入【中盘激战】状态，造成的最终伤害大幅提高，该效果可以叠加。";
const specs = [
  {
    id: 30502,
    nameEn: "Knight countdown event",
    nameZh: "骑士倒计时事件",
    stages: ["stage.knight-1", "stage.knight-2", "stage.knight-3"],
    sourceStages: ["30508011", "30508012", "30508013"],
    params: ["0.5", "7", "4", "0.5"],
    textHash: "16601839680787935738",
    abilities: [
      "BattleEventAbility_SummonMonsterInfinite",
      "BattleEventAbility_ChallengePeakBattle_CountDown",
    ],
    presentationOnlyAbilities: [],
  },
  {
    id: 30503,
    nameEn: "Normal King countdown event",
    nameZh: "常规王棋倒计时事件",
    stages: ["stage.king-normal"],
    sourceStages: ["30508021"],
    params: ["0.5", "7", "4", "0.5"],
    textHash: "12468610405161025562",
    abilities: [
      "BattleEventAbility_SummonMonsterInfinite",
      "BattleEventAbility_ChallengePeakBattle_CountDown",
    ],
    presentationOnlyAbilities: [],
  },
  {
    id: 30504,
    nameEn: "Plight King countdown event",
    nameZh: "困厄王棋倒计时事件",
    stages: ["stage.king-plight"],
    sourceStages: ["30508022"],
    params: ["0.5", "3", "0", "0"],
    textHash: null,
    abilities: [
      "BattleEventAbility_SummonMonsterInfinite",
      "BattleEventAbility_ChallengePeakBattle_CountDown",
    ],
    presentationOnlyAbilities: [
      "BattleEventAbility_ChallengePeakBattle_HardBossScreenEffect",
    ],
  },
];
const records = specs.map((spec) => ({
  id: `battle-event.${spec.id}`,
  schema_revision: "starclock.anomaly-arbitration-row.v1",
  kind: "BattleEvent",
  name_en: spec.nameEn,
  name_zh_cn: spec.nameZh,
  summary_en:
    `${spec.nameEn} owns the stage-local countdown actor and its mechanical ability list.`,
  summary_zh_cn:
    `${spec.nameZh}承载关卡局部倒计时参与者及其机械能力列表。`,
  ownership: "Shared",
  coverage_state: "DataReady",
  evidence_quality: "ExactStructured",
  mechanism_quality: "ExactRelationship",
  manifest_record_ids: [`battle_events:battle-event:${spec.id}`],
  source_refs: [
    structuredRef(
      "battle_events",
      `battle-event:${spec.id}`,
      "Active StageConfig _CreateBattleEvent resolves this exact row.",
    ),
    ...spec.sourceStages.map((stageId) => structuredRef(
      "stage_configs",
      `stage:${stageId}`,
      `StageConfig ${stageId} selects battle event ${spec.id}.`,
    )),
    ...(spec.textHash === null ? [] : [
      textRef("en", spec.textHash, descriptionEn),
      textRef("zh_cn", spec.textHash, descriptionZh),
    ]),
  ],
  tags: ["battle-event", "countdown", spec.id === 30502 ? "knight"
    : spec.id === 30503 ? "king-normal" : "king-plight"].sort(),
  source_numeric_id: spec.id,
  team: "TeamNeutral",
  event_subtype: "BattleVersusBarWarningEvent",
  stage_ids: spec.stages,
  source_stage_ids: spec.sourceStages,
  mechanical_ability_names: spec.abilities,
  presentation_only_ability_names: spec.presentationOnlyAbilities,
  override_properties: {
    base_hp: "90",
    base_attack: "100",
    speed: "100",
  },
  source_parameters: spec.params,
  action_bar_text_en: spec.textHash === null ? null : descriptionEn,
  action_bar_text_zh_cn: spec.textHash === null ? null : descriptionZh,
  countdown_program_owner_batch: "G13-P2-B3",
  presentation_assets_included: false,
  runtime_executable: false,
}));
const document = {
  schema_revision: "starclock.anomaly-arbitration-normalized-file.v1",
  goal_id: "anomaly-arbitration-reference-v1",
  profile: "anomaly-arbitration-v1",
  file: "battle-events.json",
  record_kind: "BattleEvent",
  records,
};
const bytes = `${JSON.stringify(document, null, 2)}\n`;
await mkdir(path.dirname(output), { recursive: true });
if (check) {
  const existing = await readFile(output, "utf8").catch(() => "");
  if (existing !== bytes) throw new Error("battle-events.json generation drift");
} else {
  await writeFile(output, bytes);
}
console.log(`Anomaly Arbitration battle events generated: ${records.length}.`);
