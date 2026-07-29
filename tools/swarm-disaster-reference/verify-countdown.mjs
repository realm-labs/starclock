#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-countdown.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function unique(values) {
  return values.length === new Set(values).size;
}

const normalizedRoot = "content-reference/swarm-disaster-v1";
const countdownRows = json(`${normalizedRoot}/countdown-and-disarray.json`);
const decayRows = json(`${normalizedRoot}/boss-decay-levels.json`);
const manifest = json(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
assert(countdownRows.length === 1, "countdown lifecycle row count drift");
assert(decayRows.length === 42, "boss-decay row count drift");
assert(unique(decayRows.map(({ id }) => id)), "duplicate boss-decay ID");

const countdown = countdownRows[0];
assert(countdown.initial_value === "20"
  && countdown.initial_value_quality === "ProjectPolicy",
"countdown initial-value policy drift");
assert(countdown.movement_delta === "-1"
  && countdown.movement_delta_quality === "ExactReleasedText",
"countdown movement delta drift");
assert(
  countdown.transition_boundary === "AcceptedMoveWhenPreMoveCountdownIsZero"
    && countdown.transition_result.disruption_level === "1",
  "countdown transition boundary drift",
);
assert(countdown.warning_threshold === "5", "warning threshold drift");
assert(countdown.disarray_tiers.length === 3
  && countdown.disarray_tiers[2].maximum_level === "20"
  && countdown.cap_policy === "Level21AndAboveRetainsLevel20Modifiers",
"Planar Disarray tier/cap drift");

const expectedConstants = new Set(manifest.categories.mode_constants.records
  .map(({ id }) => id));
for (const binding of countdown.source_constant_bindings)
  assert(expectedConstants.delete(binding.id),
    `unexpected or duplicate constant ${binding.id}`);
assert(expectedConstants.size === 0, "mode constant exact-once mismatch");

const expectedDecayIds = new Set(manifest.categories.boss_decay_levels.records
  .map(({ id }) => `swarm-disaster.boss-decay.${id}`));
for (const row of decayRows) {
  assert(expectedDecayIds.delete(row.id),
    `unexpected or duplicate decay ${row.id}`);
  assert(row.coverage_state === "DataReady"
    && row.evidence_quality === "ProjectPolicy",
  `${row.id} coverage/evidence drift`);
  assert(row.effect_refs.length > 0, `${row.id} has no effect reference`);
  assert(row.stacking_policy === "SelectedRowsCoexistByStableBossDecayId"
    && row.application_boundary === "FinalBossBattleSpecCreation",
  `${row.id} stacking/application policy drift`);
}
assert(expectedDecayIds.size === 0, "boss-decay exact-once mismatch");
const swarmSpecific = decayRows.filter(({ swarm_applicability: value }) =>
  value === "EnabledByReleasedSwarmText");
const unproven = decayRows.filter(({ swarm_applicability: value }) =>
  value === "DisabledUnprovenSharedDlcRow");
assert(swarmSpecific.length === 15,
  `expected 15 Swarm-specific rows, found ${swarmSpecific.length}`);
assert(unproven.length === 27,
  `expected 27 disabled shared rows, found ${unproven.length}`);

console.log(
  "Swarm Disaster countdown verification passed: 19 exact constants, " +
  "1 policy-bound lifecycle, 15 Swarm boss effects and 27 disabled " +
  "unproven shared DLC rows.",
);
