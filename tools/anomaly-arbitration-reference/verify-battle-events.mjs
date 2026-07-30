#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const output = path.join(
  root,
  "content-reference/anomaly-arbitration-v1/battle-events.json",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));
const schema = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/normalized-schema.json",
), "utf8"));

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join(
    root,
    "tools/anomaly-arbitration-reference/import-battle-events.mjs",
  ),
  "--check",
], { stdio: "inherit" });
const encoded = await readFile(output);
const document = JSON.parse(encoded);
assert(
  document.schema_revision
    === "starclock.anomaly-arbitration-normalized-file.v1"
  && document.goal_id === "anomaly-arbitration-reference-v1"
  && document.profile === "anomaly-arbitration-v1"
  && document.file === "battle-events.json"
  && document.record_kind === "BattleEvent",
  "battle-event envelope drift",
);
assert(document.records.length === 3, "battle-event count drift");
const expected = [
  [30502, ["30508011", "30508012", "30508013"],
    ["0.5", "7", "4", "0.5"]],
  [30503, ["30508021"], ["0.5", "7", "4", "0.5"]],
  [30504, ["30508022"], ["0.5", "3", "0", "0"]],
];
for (const [index, record] of document.records.entries()) {
  const [id, stageIds, parameters] = expected[index];
  for (const field of schema.common_envelope.required_fields)
    assert(record[field] !== undefined, `${record.id} lacks ${field}`);
  assert(record.kind === "BattleEvent"
    && record.name_en && record.name_zh_cn
    && record.summary_en && record.summary_zh_cn
    && record.ownership === "Shared"
    && record.coverage_state === "DataReady"
    && record.runtime_executable === false
    && record.source_numeric_id === id,
  `${record.id} boundary drift`);
  assert(JSON.stringify(record.source_stage_ids) === JSON.stringify(stageIds)
    && JSON.stringify(record.source_parameters)
      === JSON.stringify(parameters),
  `${record.id} selector/parameter drift`);
  assert(record.mechanical_ability_names.includes(
    "BattleEventAbility_ChallengePeakBattle_CountDown",
  ) && record.countdown_program_owner_batch === "G13-P2-B3",
  `${record.id} countdown relationship drift`);
  assert(record.presentation_assets_included === false
    && (id !== 30504
      || JSON.stringify(record.presentation_only_ability_names)
        === JSON.stringify([
          "BattleEventAbility_ChallengePeakBattle_HardBossScreenEffect",
        ])),
  `${record.id} presentation boundary drift`);
  assert(record.source_parameters.every(
    (value) => typeof value === "string"
      && /^(?:0|[1-9]\d*)(?:\.\d*[1-9])?$/u.test(value),
  ), `${record.id} has noncanonical parameter`);
  const manifestReference = `battle_events:battle-event:${id}`;
  assert(record.manifest_record_ids.length === 1
    && record.manifest_record_ids[0] === manifestReference
    && manifest.categories.battle_events.records.some(
      ({ id: recordId }) => `battle_events:${recordId}` === manifestReference,
    ),
  `${record.id} manifest relationship drift`);
  for (const source of record.source_refs) {
    for (const field of schema.types.source_ref.required_fields)
      assert(source[field] !== undefined && source[field] !== "",
        `${record.id} source lacks ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256),
      `${record.id} source digest drift`);
  }
  assert((id === 30504) === (record.action_bar_text_en === null),
    `${record.id} action-bar evidence drift`);
}
console.log(
  "Anomaly Arbitration battle events verified: "
    + `battle-events.json=${createHash("sha256").update(encoded).digest("hex")}`,
);
