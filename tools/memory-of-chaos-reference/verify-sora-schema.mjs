#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";

const phaseArg = process.argv.find((argument) => argument.startsWith("--phase="));
const phase = Number(phaseArg?.slice("--phase=".length) ?? 4);
const expectedCounts = [0, 8, 14, 21, 27];
if (!Number.isInteger(phase) || phase < 1 || phase > 4) throw new Error("--phase must be 1..4");
const root = path.resolve(".");
const projectPath = path.join(root, "config/memory-of-chaos/project.toml");
const sora = process.env.STARCLOCK_SORA_BIN
  ?? path.join(root, ".cache/tools/sora-cli-0.3.0/bin/sora");
function assert(condition, message) { if (!condition) throw new Error(message); }
assert(execFileSync(sora, ["--version"], { encoding: "utf8" }).trim() === "sora 0.3.0",
  "pinned Sora version drift");
execFileSync(sora, ["check", "--project", projectPath], { cwd: root, stdio: "inherit" });
const project = await readFile(projectPath, "utf8");
assert(project.includes('package = "starclock_memory_of_chaos_reference_config"'), "package isolation drift");
for (const forbidden of ['out = "../generated/', 'out = "../universe-generated/', 'data_root = "../data"']) {
  assert(!project.includes(forbidden), `forbidden project output ${forbidden}`);
}
const schemas = ["core.toml", "systems.toml", "content.toml", "review.toml"].slice(0, phase);
const schemaTexts = await Promise.all(schemas.map((file) =>
  readFile(path.join(root, "config/memory-of-chaos/schema", file), "utf8")));
const tables = schemaTexts.join("\n").split("[[tables]]").length - 1;
assert(tables === expectedCounts[phase], `Sora table count drift ${tables}`);
for (const required of ["stable_key", "name_en", "name_zh_cn", "summary_en", "summary_zh_cn", "manifest_record_ids", "source_ref_ids", "payload_json", "runtime_executable"]) {
  assert(schemaTexts.every((schema) => schema.includes(`name = "${required}"`)), `missing common field ${required}`);
}
assert(schemaTexts[0].includes("ref<MocProfile.id>") && schemaTexts[0].includes("ref<MocStages.id>"),
  "core typed references missing");
console.log(`Goal 17 Sora schema verified: phase ${phase}, ${tables} isolated tables, pinned Sora 0.3.0.`);
