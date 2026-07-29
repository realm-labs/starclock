#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const output = path.join(
  root,
  "content-reference/anomaly-arbitration-v1/clocks.json",
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
  path.join(root, "tools/anomaly-arbitration-reference/import-clocks.mjs"),
  "--check",
], { stdio: "inherit" });
const encoded = await readFile(output);
const document = JSON.parse(encoded);
assert(
  document.schema_revision
    === "starclock.anomaly-arbitration-normalized-file.v1"
  && document.goal_id === "anomaly-arbitration-reference-v1"
  && document.profile === "anomaly-arbitration-v1"
  && document.file === "clocks.json"
  && document.record_kind === "ClockRule",
  "clocks normalized envelope drift",
);
for (const record of document.records) {
  for (const field of schema.common_envelope.required_fields)
    assert(record[field] !== undefined, `${record.id} lacks ${field}`);
  assert(record.kind === "ClockRule"
    && record.ownership === "AnomalyArbitration"
    && record.coverage_state === "DataReady"
    && record.runtime_executable === false,
  `${record.id} boundary drift`);
  assert(record.name_en && record.name_zh_cn
    && record.summary_en && record.summary_zh_cn,
  `${record.id} lacks bilingual text`);
  for (const reference of record.manifest_record_ids) {
    const separator = reference.indexOf(":");
    const category = reference.slice(0, separator);
    const id = reference.slice(separator + 1);
    assert(manifest.categories[category]?.records.some(
      (candidate) => candidate.id === id,
    ), `${record.id} unresolved manifest record ${reference}`);
  }
  for (const source of record.source_refs) {
    for (const field of schema.types.source_ref.required_fields)
      assert(source[field] !== undefined && source[field] !== "",
        `${record.id} source lacks ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256)
      && source.game_version === "4.4",
    `${record.id} source receipt drift`);
  }
  for (const approximation of record.approximations ?? []) {
    for (const field of schema.types.approximation.required_fields)
      assert(approximation[field] !== undefined,
        `${record.id} approximation lacks ${field}`);
    assert(approximation.alternatives.length > 0
      && approximation.affected_fixture_ids.length > 0
      && approximation.replacement_condition.length > 0,
    `${record.id} approximation is not replaceable`);
  }
}

const records = document.records;
assert(records.length === 9, "clock rule count drift");
assert(JSON.stringify(records.map(({ boundary_order: order }) => order))
  === JSON.stringify([10, 20, 30, 40, 50, 60, 70, 80, 90]),
"clock boundary ordering drift");
for (const [id, stageKind, limit, constantName] of [
  ["clock.knight-cycle-limit", "Knight", 6,
    "ChallengePeak_Mob_Turn_Limit"],
  ["clock.normal-king-cycle-limit", "KingNormal", 6,
    "ChallengePeak_Boss_Turn_Limit"],
  ["clock.plight-cycle-limit", "KingPlight", 2,
    "ChallengePeak_HardBoss_Turn_Limit"],
]) {
  const record = records.find((candidate) => candidate.id === id);
  assert(record.stage_kind === stageKind
    && record.limit_cycles === limit
    && record.reset_on_wave_transition === false
    && record.manifest_record_ids.includes(
      `mode_constants:constant:${constantName}`,
    ),
  `${id} exact limit drift`);
}
const first = records.find(
  ({ id }) => id === "clock.first-cycle-action-value",
);
assert(first.qualitative_rule
    === "GreaterTotalActionValueThanLaterCycles"
  && first.numeric_action_value === "Unavailable"
  && first.approximations.length === 1,
"first-cycle rule invented a numeric value");
const carry = records.find(
  ({ id }) => id === "clock.wave-transition-carry",
);
assert(carry.countdown_projection === "CarryRemainingCycles"
  && carry.reset_on_wave_transition === false,
"wave carry drift");
const warning = records.find(
  ({ id }) => id === "clock.warning-threshold",
);
assert(warning.threshold_cycles_remaining === "Unavailable"
  && warning.warning_state === "FewCyclesRemain"
  && warning.approximations.length === 1,
"warning threshold overclaim");
const low = records.find(
  ({ id }) => id === "clock.low-cycle-combat-effect",
);
assert(low.trigger === "CycleStartWhileFewCyclesRemain"
  && low.target === "Allies"
  && low.buff_id === "Unavailable"
  && low.numeric_parameters === "Unavailable",
"low-cycle contribution overclaim");
const expiry = records.find(
  ({ id }) => id === "clock.expiry-and-failure",
);
assert(expiry.boundary === "CycleLimitExceeded"
  && expiry.terminal_outcome_id === "outcome.stage-attempt-failure"
  && expiry.current_record_projection === "Unchanged"
  && expiry.best_record_projection === "Unchanged",
"cycle expiry boundary drift");
const retry = records.find(({ id }) => id === "clock.retry-boundary");
assert(retry.prior_clock_state === "TerminalAndImmutable"
  && retry.new_clock_state === "FreshStageLocalClock"
  && retry.carry_elapsed_cycles === false
  && retry.evidence_quality === "ProjectPolicy",
"retry clock boundary drift");

const covered = new Set(records.flatMap(
  ({ manifest_record_ids: references }) => references,
));
const required = [
  ...manifest.categories.clock_rules.records.map(
    ({ id }) => `clock_rules:${id}`,
  ),
  ...[
    "ChallengePeak_Mob_Turn_Limit",
    "ChallengePeak_Boss_Turn_Limit",
    "ChallengePeak_HardBoss_Turn_Limit",
  ].map((id) => `mode_constants:constant:${id}`),
];
assert(required.length === 12 && covered.size === 12,
  "P1-B4 denominator drift");
for (const reference of required)
  assert(covered.has(reference), `uncovered clock obligation ${reference}`);

console.log(
  "Anomaly Arbitration clocks verified: "
    + `clocks.json=${createHash("sha256").update(encoded).digest("hex")}`,
);
