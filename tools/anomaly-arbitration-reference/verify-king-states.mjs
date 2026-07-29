#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const outputRoot = path.join(root, "content-reference/anomaly-arbitration-v1");
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));
const schema = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/normalized-schema.json",
), "utf8"));
const files = ["king-states.json", "king-protection.json"];
const kinds = {
  "king-states.json": "KingState",
  "king-protection.json": "KingProtectionRule",
};

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

execFileSync(process.execPath, [
  path.join(
    root,
    "tools/anomaly-arbitration-reference/import-king-states.mjs",
  ),
  "--check",
], { stdio: "inherit" });
const encoded = Object.fromEntries(await Promise.all(files.map(
  async (file) => [file, await readFile(path.join(outputRoot, file))],
)));
const documents = Object.fromEntries(Object.entries(encoded).map(
  ([file, bytes]) => [file, JSON.parse(bytes)],
));
for (const [file, document] of Object.entries(documents)) {
  assert(
    document.schema_revision
      === "starclock.anomaly-arbitration-normalized-file.v1"
    && document.goal_id === "anomaly-arbitration-reference-v1"
    && document.profile === "anomaly-arbitration-v1"
    && document.file === file
    && document.record_kind === kinds[file],
    `${file} envelope drift`,
  );
  for (const record of document.records) {
    for (const field of schema.common_envelope.required_fields)
      assert(record[field] !== undefined,
        `${file}/${record.id} lacks ${field}`);
    assert(record.kind === kinds[file]
      && record.ownership === "AnomalyArbitration"
      && record.coverage_state === "DataReady"
      && record.runtime_executable === false,
    `${file}/${record.id} boundary drift`);
    assert(record.name_en && record.name_zh_cn
      && record.summary_en && record.summary_zh_cn,
    `${file}/${record.id} lacks bilingual text`);
    for (const reference of record.manifest_record_ids) {
      const separator = reference.indexOf(":");
      const category = reference.slice(0, separator);
      const id = reference.slice(separator + 1);
      assert(manifest.categories[category]?.records.some(
        (candidate) => candidate.id === id,
      ), `${file}/${record.id} unresolved manifest record ${reference}`);
    }
    for (const source of record.source_refs) {
      for (const field of schema.types.source_ref.required_fields)
        assert(source[field] !== undefined && source[field] !== "",
          `${file}/${record.id} source lacks ${field}`);
      assert(/^[0-9a-f]{64}$/u.test(source.sha256)
        && source.game_version === "4.4",
      `${file}/${record.id} source receipt drift`);
    }
    for (const approximation of record.approximations ?? []) {
      for (const field of schema.types.approximation.required_fields)
        assert(approximation[field] !== undefined,
          `${file}/${record.id} approximation lacks ${field}`);
      assert(approximation.alternatives.length > 0
        && approximation.affected_fixture_ids.length > 0
        && approximation.replacement_condition.length > 0,
      `${file}/${record.id} approximation is not replaceable`);
    }
  }
}

const states = documents["king-states.json"].records;
assert(states.length === 2, "King state count drift");
const plight = states.find(({ id }) => id === "king-state.plight");
assert(plight.state_order === 10
  && plight.source_stage_id === "30508022"
  && plight.availability === "DirectAlternative"
  && plight.protection_state === "ActiveKnightProtection",
"Plight state drift");
const normal = states.find(({ id }) => id === "king-state.normal");
assert(normal.state_order === 20
  && normal.source_stage_id === "30508021"
  && normal.required_cleared_knight_count === 3
  && normal.protection_contribution_count === 0
  && normal.evidence_quality === "ApproximateFromReleasedText"
  && normal.approximations.length === 1,
"normal King state boundary drift");

const protection = documents["king-protection.json"].records;
assert(protection.length === 4, "King protection rule count drift");
assert(JSON.stringify(protection.map(
  ({ contribution_order: order }) => order,
)) === JSON.stringify([10, 20, 30, 40]),
"King protection ordering drift");
const composition = protection.find(
  ({ id }) => id === "king-protection.composition",
);
assert(composition.contribution_ids.length === 3
  && composition.numeric_effects === "Unavailable"
  && composition.evidence_quality === "ProjectPolicy",
"protection composition overclaims released evidence");
const clear = protection.find(
  ({ id }) => id === "king-protection.knight-clear-contribution",
);
assert(JSON.stringify(clear.evaluation_order) === JSON.stringify([
  "CommitKnightClear",
  "DeactivateMatchingTransmission",
  "RecomputeKingAvailability",
]), "Knight-clear protection order drift");
const reset = protection.find(
  ({ id }) => id === "king-protection.reset-and-teardown",
);
assert(JSON.stringify(reset.ordered_reset_projection) === JSON.stringify([
  "InvalidateCurrentKnightClear",
  "ReactivateMatchingKnightTransmission",
  "RevokeNormalKingAvailabilityIfAnyTransmissionActive",
]) && reset.best_battle_record_effect === "Unchanged",
"protection reset order drift");
const shortcut = protection.find(
  ({ id }) => id === "king-protection.direct-plight-shortcut",
);
assert(shortcut.trigger === "SuccessfulPlightKingClear"
  && shortcut.exact_projection.knight_stage_ids.length === 3
  && shortcut.exact_projection.stars_each === 3
  && shortcut.account_reward_projection === "Excluded"
  && shortcut.loadout_snapshot_projection === "NoSyntheticKnightTeamSnapshots"
  && shortcut.approximations.length === 1,
"direct Plight shortcut drift");
assert(JSON.stringify(shortcut.downstream_order) === JSON.stringify([
  "CommitPlightKingClear",
  "ProjectThreeStarKnightEquivalence",
  "EvaluateCurrentAndBestProgress",
  "SettleMechanicalResults",
]), "direct Plight downstream order drift");

const covered = new Set(Object.values(documents).flatMap(
  ({ records }) => records.flatMap(
    ({ manifest_record_ids: references }) => references,
  ),
));
const required = manifest.categories.king_state_transitions.records.map(
  ({ id }) => `king_state_transitions:${id}`,
);
assert(required.length === 6 && covered.size === 6,
  "King transition denominator drift");
for (const reference of required)
  assert(covered.has(reference), `uncovered King transition: ${reference}`);

console.log(
  `Anomaly Arbitration King states verified: ${files.map(
    (file) => `${file}=${digest(encoded[file])}`,
  ).join(" ")}`,
);
