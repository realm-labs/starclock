#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(".");
const projectPath = path.join(
  root,
  "config/anomaly-arbitration/project.toml",
);
const schemaPath = path.join(
  root,
  "config/anomaly-arbitration/schema/system.toml",
);
const sora = path.join(
  root,
  ".cache/tools/sora-cli-0.3.0/bin/sora",
);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const [project, schema] = await Promise.all([
  readFile(projectPath, "utf8"),
  readFile(schemaPath, "utf8"),
]);
assert(execFileSync(sora, ["--version"], { encoding: "utf8" }).trim()
  === "sora 0.3.0", "pinned Sora version drift");
execFileSync(sora, ["check", "--project", projectPath], {
  cwd: root,
  stdio: "inherit",
});
assert(project.includes(
  'package = "starclock_anomaly_arbitration_reference_config"',
) && project.includes('"schema/system.toml"')
  && project.includes('data_root = "data"')
  && project.includes(
    'schema_lock = "../anomaly-arbitration-generated/schema.lock"',
  )
  && project.includes(
    'out = "../anomaly-arbitration-generated/config.sora"',
  ),
"isolated project contract drift");
for (const forbidden of [
  'out = "../generated/',
  'out = "../universe-generated/',
  'data_root = "../data"',
])
  assert(!project.includes(forbidden), `forbidden project output: ${forbidden}`);

const expected = new Map([
  ["AnomalyArbitrationProfiles", "Profiles"],
  ["AnomalyArbitrationPeriods", "Periods"],
  ["AnomalyArbitrationStages", "Stages"],
  ["AnomalyArbitrationTerminalOutcomes", "TerminalOutcomes"],
  ["AnomalyArbitrationParticipantPolicies", "ParticipantPolicies"],
  ["AnomalyArbitrationTeamSlots", "TeamSlots"],
  ["AnomalyArbitrationLoadoutRecords", "LoadoutRecords"],
  ["AnomalyArbitrationProgressRecords", "ProgressRecords"],
]);
const tableBlocks = schema.split("[[tables]]").slice(1);
assert(tableBlocks.length === expected.size, "system table count drift");
for (const block of tableBlocks) {
  const name = /name = "([^"]+)"/u.exec(block)?.[1];
  const sheet = /sheet = "([^"]+)"/u.exec(block)?.[1];
  assert(expected.get(name) === sheet, `${name} sheet contract drift`);
  assert(block.includes('file = "AnomalyArbitration.xlsx"')
    && block.includes('name = "stable_key"')
    && block.includes('name = "payload_json"')
    && block.includes('name = "runtime_executable"')
    && block.includes('name = "manifest_record_ids"')
    && block.includes('name = "source_ref_ids"'),
  `${name} common authoring fields drift`);
}
for (const typedReference of [
  "ref<AnomalyArbitrationProfiles.id>",
  "ref<AnomalyArbitrationPeriods.id>",
  "ref<AnomalyArbitrationStages.id>",
])
  assert(schema.includes(typedReference),
    `missing typed reference ${typedReference}`);
for (const enumName of [
  "AnomalyArbitrationOwnership",
  "AnomalyArbitrationCoverageState",
  "AnomalyArbitrationEvidenceQuality",
  "AnomalyArbitrationMechanismQuality",
  "AnomalyArbitrationStageKind",
  "AnomalyArbitrationDifficulty",
])
  assert(schema.includes(`name = "${enumName}"`),
    `missing enum ${enumName}`);

console.log(
  "Anomaly Arbitration Sora system schema verified: "
    + `8 tables, schema=${createHash("sha256").update(schema).digest("hex")}, `
    + `project=${createHash("sha256").update(project).digest("hex")}.`,
);
