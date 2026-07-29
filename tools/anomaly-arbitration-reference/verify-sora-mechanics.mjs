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
  "config/anomaly-arbitration/schema/mechanics.toml",
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
execFileSync(sora, ["check", "--project", projectPath], {
  cwd: root,
  stdio: "inherit",
});
assert(project.includes('"schema/system.toml"')
  && project.includes('"schema/mechanics.toml"'),
"Sora schema include order drift");
const expected = new Map([
  ["AAKingStates", "KingStates"],
  ["AAKingProtection", "KingProtection"],
  ["AAClocks", "Clocks"],
  ["AAQuadrantOptions", "QuadrantOptions"],
  ["AAQuadrantSelections", "QuadrantSelections"],
  ["AATargets", "Targets"],
  ["AAObjectives", "Objectives"],
  ["AAStageResults", "StageResults"],
  ["AAAggregations", "Aggregations"],
]);
const tableBlocks = schema.split("[[tables]]").slice(1);
assert(tableBlocks.length === expected.size, "mechanics table count drift");
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
assert(schema.includes("ref<AAProfiles.id>")
  && schema.includes("ref<AAStages.id>"),
"mechanics typed-reference drift");
assert((schema.match(/name = "source_numeric_id"/gu) ?? []).length === 2,
  "source numeric ID authoring boundary drift");

console.log(
  "Anomaly Arbitration Sora mechanics schema verified: "
    + `9 tables, schema=${createHash("sha256").update(schema).digest("hex")}, `
    + `project=${createHash("sha256").update(project).digest("hex")}.`,
);
