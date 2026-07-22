import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const root = path.resolve(process.argv[2] ?? ".");
const project = path.join(root, "config");
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy", "sora-toolchain.json"), "utf8"));
const sora = path.join(root, policy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
const assert = (condition, message) => { if (!condition) throw new Error(message); };
const run = (args) => {
  const result = spawnSync(sora, args, { cwd: project, encoding: "utf8" });
  if (result.status !== 0) throw new Error(`${args.join(" ")} failed\n${result.stdout}\n${result.stderr}`);
  return result.stdout.trim();
};

assert(fs.existsSync(sora), `Sora ${policy.version} is not installed`);
assert(run(["--version"]) === `sora ${policy.version}`, "Sora version differs from policy");
run(["--serial", "check", "--project", "./project.toml"]);

const lock = JSON.parse(fs.readFileSync(path.join(project, "generated", "schema.lock"), "utf8"));
const universeTables = lock.schema.tables.filter((table) => table.name.startsWith("Universe"));
const tableByName = new Map(universeTables.map((table) => [table.name, table]));
assert(universeTables.length === 49, `expected 49 Universe tables, got ${universeTables.length}`);
for (const table of universeTables) {
  assert(["Universe.xlsx", "UniverseBindings.xlsx", "UniverseEvidence.xlsx"].includes(table.source.file), `${table.name} uses an unapproved workbook`);
  assert(table.source.sheet === table.name, `${table.name} sheet name differs from table name`);
  for (const field of table.fields) if (field.name.endsWith("_decimal")) {
    const type = field.ty?.Optional ?? field.ty;
    assert(type === "String", `${table.name}.${field.name} is not string-transported`);
    assert(JSON.stringify(field.length) === "[1,32]", `${table.name}.${field.name} has the wrong decimal length`);
  }
}

function assertRef(table, field, target) {
  const value = tableByName.get(table)?.fields.find((candidate) => candidate.name === field)?.ty;
  const ref = value?.Ref ?? value?.Optional?.Ref;
  assert(ref?.table === target, `${table}.${field} is not a typed ${target} reference`);
}
assertRef("UniverseDifficulty", "world_id", "UniverseWorld");
assertRef("UniverseRoom", "domain_id", "UniverseDomain");
assertRef("UniverseBlessing", "path_id", "UniversePath");
assertRef("UniverseBlessingLevel", "blessing_id", "UniverseBlessing");
assertRef("UniverseCurioState", "curio_id", "UniverseCurio");
assertRef("UniverseOccurrenceChoice", "variant_id", "UniverseOccurrenceVariant");
assertRef("UniverseEncounterMember", "group_id", "UniverseEncounterGroup");
assertRef("UniverseEncounterWave", "member_id", "UniverseEncounterMember");
assertRef("UniverseEncounterPool", "room_id", "UniverseRoom");
assertRef("UniverseActivityBinding", "profile_id", "UniverseProfile");

function assertString(table, field) {
  const value = tableByName.get(table)?.fields.find((candidate) => candidate.name === field)?.ty;
  assert(value === "String", `${table}.${field} is not a stable string binding`);
}
assertString("UniverseDifficultyEnemy", "enemy_variant_stable_key");
assertString("UniverseEncounterWaveEnemy", "enemy_variant_stable_key");
assertString("UniverseBlessingPrerequisite", "prerequisite_stable_key");

const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "starclock-universe-schema-"));
try {
  const directLock = path.join(temporary, "schema.lock");
  const directTemplates = path.join(temporary, "templates");
  const directRust = path.join(temporary, "rust");
  run(["--serial", "schema-lock", "--project", "./project.toml", "--out", directLock]);
  run(["--serial", "excel-template", "--project", "./project.toml", "--out", directTemplates]);
  run(["--serial", "gen", "--target", "rust", "--project", "./project.toml", "--out", directRust, "--format-code", "never"]);
  const rustFiles = fs.readdirSync(directRust).filter((file) => file.endsWith(".rs")).map((file) => path.join(directRust, file));
  const formatted = spawnSync("rustfmt", ["--edition", "2024", ...rustFiles], { cwd: root, encoding: "utf8" });
  assert(formatted.status === 0, `rustfmt failed for direct readers: ${formatted.stderr}`);
  assert(fs.readFileSync(directLock).equals(fs.readFileSync(path.join(project, "generated", "schema.lock"))), "committed schema lock drifted");
  for (const workbook of ["Universe.xlsx", "UniverseBindings.xlsx", "UniverseEvidence.xlsx"]) {
    assert(fs.statSync(path.join(directTemplates, workbook)).size > 1000, `${workbook} direct template is missing`);
    assert(fs.statSync(path.join(project, "generated", "templates", workbook)).size > 1000, `${workbook} committed template is missing`);
  }
  for (const table of universeTables) {
    const file = `${table.name.replace(/([a-z0-9])([A-Z])/gu, "$1_$2").toLowerCase()}.rs`;
    assert(fs.readFileSync(path.join(directRust, file)).equals(fs.readFileSync(path.join(project, "generated", "rust", file))), `${file} reader drifted`);
  }
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

console.log(`Universe Sora schema verified: ${universeTables.length} tables, three multi-sheet templates, generated lock/readers stable.`);
