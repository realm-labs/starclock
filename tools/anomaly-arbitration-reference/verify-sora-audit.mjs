#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdtemp, readFile, readdir } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const projectPath = path.join(
  root,
  "config/anomaly-arbitration/project.toml",
);
const schemaPath = path.join(
  root,
  "config/anomaly-arbitration/schema/audit.toml",
);
const generatedRoot = path.join(
  root,
  "config/anomaly-arbitration-generated",
);
const sora = path.join(
  root,
  ".cache/tools/sora-cli-0.3.0/bin/sora",
);

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function files(directory) {
  const output = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) output.push(...await files(target));
    else output.push(target);
  }
  return output.sort();
}

function run(args) {
  execFileSync(sora, args, { cwd: root, stdio: "inherit" });
}

function zipMembers(workbook) {
  return execFileSync("unzip", ["-Z1", workbook], { encoding: "utf8" })
    .trim().split("\n").filter(Boolean).sort();
}

function zipMember(workbook, member) {
  const escapedMember = member.replaceAll("[", "\\[").replaceAll("]", "\\]");
  return execFileSync("unzip", ["-p", workbook, escapedMember], {
    encoding: null,
    maxBuffer: 32 * 1024 * 1024,
  });
}

execFileSync(process.execPath, [
  path.join(
    root,
    "tools/anomaly-arbitration-reference/generate-sora-audit-schema.mjs",
  ),
  "--check",
], { stdio: "inherit" });
const [project, schema] = await Promise.all([
  readFile(projectPath, "utf8"),
  readFile(schemaPath, "utf8"),
]);
run(["check", "--project", projectPath]);
assert(project.includes('"schema/audit.toml"'),
  "audit schema include drift");
const expected = new Map([
  ["ArbMechanicRules", "MechanicRules"],
  ["ArbSources", "Sources"],
  ["ArbReconciliation", "Reconciliation"],
  ["ArbCoverage", "Coverage"],
  ["ArbResearchGaps", "ResearchGaps"],
  ["ArbReviewFixtures", "ReviewFixtures"],
  ["ArbManifestReceipt", "ManifestReceipt"],
  ["ArbPackIndex", "PackIndex"],
]);
const tableBlocks = schema.split("[[tables]]").slice(1);
assert(tableBlocks.length === expected.size, "audit table count drift");
for (const block of tableBlocks) {
  const name = /name = "([^"]+)"/u.exec(block)?.[1];
  const sheet = /sheet = "([^"]+)"/u.exec(block)?.[1];
  assert(expected.get(name) === sheet, `${name} sheet contract drift`);
  assert(block.includes('file = "AnomalyArbitrationReview.xlsx"')
    && block.includes('name = "stable_key"')
    && block.includes('name = "payload_json"')
    && block.includes('name = "runtime_executable"'),
  `${name} authoring fields drift`);
}
assert(schema.includes("list<ref<ArbSources.id>>")
  && schema.includes("ref<ArbProfiles.id>"),
"audit typed-reference drift");

const lock = JSON.parse(await readFile(
  path.join(generatedRoot, "schema.lock"),
));
assert(lock.schema.tables.length === 37, "generated table count drift");
const tableNameBySheet = new Map(lock.schema.tables.map((table) =>
  [table.source.sheet, table.name]));
const temporary = await mkdtemp(path.join(
  os.tmpdir(),
  "starclock-g13-sora-audit-",
));
run([
  "schema-lock", "--project", projectPath,
  "--out", path.join(temporary, "schema.lock"),
]);
run([
  "excel-template", "--project", projectPath,
  "--out", path.join(temporary, "templates"),
]);
run([
  "gen", "--target", "rust", "--project", projectPath,
  "--out", path.join(temporary, "readers/rust"), "--format-code", "never",
]);
for (const relative of [
  "schema.lock",
  ...((await files(path.join(generatedRoot, "readers/rust"))).map((file) =>
    path.relative(generatedRoot, file))),
]) {
  const committed = await readFile(path.join(generatedRoot, relative));
  const regenerated = await readFile(path.join(temporary, relative));
  assert(committed.equals(regenerated),
    `${relative} generated artifact drift`);
}

const workbookSheets = {
  "AnomalyArbitration.xlsx": [
    "Profiles", "Periods", "Stages", "TerminalOutcomes",
    "ParticipantPolicies", "TeamSlots", "LoadoutRecords",
    "ProgressRecords", "KingStates", "KingProtection", "Clocks",
    "QuadrantOptions", "QuadrantSelections", "Targets", "Objectives",
    "StageResults", "Aggregations",
  ],
  "AnomalyArbitrationBindings.xlsx": [
    "PoolAudits", "Traits", "MazeBuffBindings", "BattleEvents",
    "Encounters", "EncounterWaves", "EnemySlots", "Enemies",
    "EnemySkills", "EnemyStatuses", "AbilityBindings",
    "MechanicContributions",
  ],
  "AnomalyArbitrationReview.xlsx": [
    "MechanicRules", "Sources", "Reconciliation", "Coverage",
    "ResearchGaps", "ReviewFixtures", "ManifestReceipt", "PackIndex",
  ],
};
for (const [name, sheets] of Object.entries(workbookSheets)) {
  const committed = path.join(generatedRoot, "templates", name);
  const regenerated = path.join(temporary, "templates", name);
  const members = zipMembers(committed);
  assert(JSON.stringify(members) === JSON.stringify(zipMembers(regenerated)),
    `${name} ZIP member drift`);
  for (const member of members) {
    if (member === "docProps/core.xml") continue;
    assert(zipMember(committed, member).equals(zipMember(regenerated, member)),
      `${name}:${member} template drift`);
  }
  const workbookXml = zipMember(committed, "xl/workbook.xml")
    .toString("utf8");
  const actualSheets = [...workbookXml.matchAll(
    /<sheet name="([^"]+)"/gu,
  )].map((match) => match[1]);
  assert(JSON.stringify(actualSheets) === JSON.stringify(sheets),
    `${name} sheet order drift`);
  const sharedStrings = zipMember(committed, "xl/sharedStrings.xml")
    .toString("utf8");
  assert(sharedStrings.includes("<t>@table</t>"),
    `${name} lacks Sora table metadata`);
  for (let index = 1; index <= sheets.length; index++) {
    const worksheet = zipMember(
      committed,
      `xl/worksheets/sheet${index}.xml`,
    ).toString("utf8");
    assert(sharedStrings.includes(
      `<t>${tableNameBySheet.get(sheets[index - 1])}</t>`,
    ) && worksheet.includes('topLeftCell="B8"'),
    `${name}/${sheets[index - 1]} metadata drift`);
  }
}
const generatedFiles = await files(generatedRoot);
const coreFiles = generatedFiles.filter((file) => {
  const relative = path.relative(generatedRoot, file);
  return relative === "schema.lock"
    || relative.startsWith("templates/")
    || relative.startsWith("readers/");
});
const debugFiles = generatedFiles.filter((file) =>
  path.relative(generatedRoot, file).startsWith("debug-json/"));
assert(coreFiles.length === 49, "generated schema/template/reader count drift");
assert(debugFiles.length === 37, "generated debug-table count drift");
assert(generatedFiles.length === 87
  && generatedFiles.some((file) =>
    path.relative(generatedRoot, file) === "config.sora"),
"generated export artifact count drift");
const treeDigest = createHash("sha256");
for (const file of generatedFiles) {
  treeDigest.update(path.relative(generatedRoot, file));
  treeDigest.update("\0");
  treeDigest.update(await readFile(file));
  treeDigest.update("\0");
}
console.log(
  "Anomaly Arbitration Sora audit/generated artifacts verified: "
    + `8 audit tables, 37 total tables, 49 core and 38 export files, `
    + `schema=${digest(Buffer.from(schema))}, `
    + `tree=${treeDigest.digest("hex")}.`,
);
