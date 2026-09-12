#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/sora-toolchain.json");
const authoring = json(
  "content-manifests/divergent-universe-v1/authoring-contract.json",
);
const lock = json("config/divergent-universe-generated/schema.lock");
const projectPath = "config/divergent-universe-project.toml";
const legacyProjectPath = "config/divergent-universe/project.toml";
const binary = process.platform === "win32" ? "sora.exe" : "sora";
const sora = path.join(root, policy.install_root, "bin", binary);

assert(policy.package === "sora-cli" && policy.version === "0.6.1",
  "Goal 22 requires exactly sora-cli 0.6.1");
assert(policy.install_root === ".cache/tools/sora-cli-0.6.1"
  && /^[a-f0-9]{64}$/u.test(policy.crate_sha256),
"Sora 0.6.1 checksum/install policy drift");
assert(fs.existsSync(sora),
  `Sora 0.6.1 is unavailable; run ${policy.install_command}`);
assert(execFileSync(sora, ["--version"], { encoding: "utf8" }).trim()
  === "sora 0.6.1", "installed Sora is not 0.6.1");

assert(fs.existsSync(path.join(root, projectPath)),
  "Divergent Universe Sora 0.6.1 project is missing");
assert(!fs.existsSync(path.join(root, legacyProjectPath)),
  "obsolete traversal-based Divergent Universe project remains");
const project = text(projectPath);
for (const declaration of [
  'project = { id = "starclock_divergent_universe_reference" }',
  "groups = { common = { default = true } }",
  'views = { default = { contract = "starclock_divergent_universe_reference/default", groups = ["common"] } }',
  'view = "default"',
]) assert(project.includes(declaration), `project lacks ${declaration}`);
assert(!project.includes("../") && !/^package\s*=/gmu.test(project),
  "Divergent Universe project uses a legacy identity or traversal output");

const schemaFiles = ["core.toml", "systems.toml", "content.toml", "evidence.toml"];
const schema = schemaFiles.map((file) =>
  text(`config/divergent-universe/schema/${file}`)).join("\n");
const tables = [...schema.matchAll(/^\[\[tables\]\]$/gmu)].length;
const tableIds = [...schema.matchAll(/^id = "([^"]+)"$/gmu)]
  .map((match) => match[1]);
assert(tables === 81 && tableIds.length === tables
  && new Set(tableIds).size === tables,
"Divergent Universe tables lack exact-once Sora 0.6.1 IDs");

assert(authoring.authority.schema_exporter_version === "0.6.1"
  && authoring.isolation.project === projectPath,
"Divergent Universe authoring authority did not migrate to Sora 0.6.1");
assert(lock.version === 3
  && lock.project_id === "starclock_divergent_universe_reference"
  && lock.contract_id === "starclock_divergent_universe_reference/default"
  && lock.view === "default"
  && lock.schema.tables.length === 81,
"Divergent Universe generated schema lock lacks the 0.6.1 project/view contract");

const activeInputs = [
  "content-manifests/divergent-universe-v1/README.md",
  "tools/divergent-universe-reference/contracts.mjs",
  "tools/divergent-universe-reference/generate-sora-artifacts.mjs",
  "tools/divergent-universe-reference/generate-sora-schema.mjs",
  "tools/divergent-universe-reference/run-clean-checkout.mjs",
  "tools/divergent-universe-reference/verify-contracts.mjs",
  "tools/divergent-universe-reference/verify-sora-release.mjs",
  "tools/divergent-universe-reference/verify-sora-schema.mjs",
  "tools/divergent-universe-reference/workbook_authoring.py",
];
for (const file of activeInputs) {
  const value = text(file);
  assert(!value.includes("sora-cli-0.3.0")
    && !value.includes("sora-cli==0.3.0")
    && !value.includes("Sora 0.3.0"),
  `${file} retains a Goal 22 Sora 0.3.0 fallback`);
}

console.log(
  "Divergent Universe Sora migration verified " +
  "(0.6.1; project/view; 81 table IDs; no active fallback).",
);

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function json(relative) {
  return JSON.parse(text(relative));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
