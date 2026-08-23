#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = json("policy/sora-toolchain.json");
const authoring = json(
  "content-manifests/currency-wars-v1/authoring-contract.json",
);
const lock = json("config/currency-wars-generated/schema.lock");
const projectPath = "config/currency-wars-project.toml";
const legacyProjectPath = "config/currency-wars/project.toml";

assert(policy.package === "sora-cli" && policy.version === "0.6.1",
  "Goal 21 requires exactly sora-cli 0.6.1");
assert(policy.install_root === ".cache/tools/sora-cli-0.6.1"
  && /^[a-f0-9]{64}$/u.test(policy.crate_sha256),
"Sora 0.6.1 checksum/install policy drift");
const sora = path.join(root, policy.install_root, "bin", "sora");
assert(fs.existsSync(sora), `Sora 0.6.1 is unavailable; run ${policy.install_command}`);
assert(execFileSync(sora, ["--version"], { encoding: "utf8" }).trim()
  === "sora 0.6.1", "installed Sora is not 0.6.1");

assert(fs.existsSync(path.join(root, projectPath)),
  "Currency Wars Sora 0.6.1 project is missing");
assert(!fs.existsSync(path.join(root, legacyProjectPath)),
  "obsolete traversal-based Currency Wars project remains");
const project = text(projectPath);
for (const declaration of [
  'project = { id = "starclock_currency_wars_reference" }',
  "groups = { common = { default = true } }",
  'views = { default = { contract = "starclock_currency_wars_reference/default", groups = ["common"] } }',
  'view = "default"',
]) assert(project.includes(declaration), `project lacks ${declaration}`);
assert(!project.includes("../") && !/^package\s*=/mu.test(project),
  "Currency Wars project uses a legacy identity or traversal output");

const schemaFiles = ["core.toml", "systems.toml", "content.toml", "audit.toml"];
const schema = schemaFiles.map((file) =>
  text(`config/currency-wars/schema/${file}`)).join("\n");
const tables = [...schema.matchAll(/^\[\[tables\]\]$/gmu)].length;
const tableIds = [...schema.matchAll(/^id = "([^"]+)"$/gmu)]
  .map((match) => match[1]);
assert(tables === 111 && tableIds.length === tables
  && new Set(tableIds).size === tables,
"Currency Wars tables lack exact-once Sora 0.6.1 IDs");

assert(authoring.authority.schema_exporter_version === "0.6.1"
  && authoring.isolation.project === projectPath,
"Currency Wars authoring authority did not migrate to Sora 0.6.1");
assert(lock.version === 3
  && lock.project_id === "starclock_currency_wars_reference"
  && lock.contract_id === "starclock_currency_wars_reference/default"
  && lock.view === "default"
  && lock.schema.tables.length === 111,
"Currency Wars generated schema lock is not the 0.6.1 project/view contract");

const activeInputs = [
  "policy/currency-wars-workbook-toolchain.json",
  "tools/currency-wars-reference/contracts.mjs",
  "tools/currency-wars-reference/generate-sora-schema.mjs",
  "tools/currency-wars-reference/run-clean-checkout.mjs",
  "tools/currency-wars-reference/verify-contracts.mjs",
  "tools/currency-wars-reference/verify-sora-generated.mjs",
  "tools/currency-wars-reference/verify-sora-reader.mjs",
  "tools/currency-wars-reference/verify-sora-schema.mjs",
];
for (const file of activeInputs) {
  const value = text(file);
  assert(!value.includes("sora-cli-0.3.0")
    && !value.includes("sora-cli==0.3.0")
    && !value.includes("Sora 0.3.0"),
  `${file} retains a Goal 21 Sora 0.3.0 fallback`);
}

console.log(
  "Currency Wars Sora migration verified (0.6.1; project/view; 111 table IDs; no active fallback).",
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
