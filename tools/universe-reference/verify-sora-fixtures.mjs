import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const root = path.resolve(process.argv[2] ?? ".");
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy", "sora-toolchain.json"), "utf8"));
const sora = path.join(root, policy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
const project = path.join(root, "config", "universe-project.toml");
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "starclock-universe-fixture-"));
const assert = (condition, message) => { if (!condition) throw new Error(message); };

function run(command, args, environment = process.env) {
  const env = command === "cargo"
    ? { ...environment, CARGO_TARGET_DIR: path.join(root, ".cache", "universe-bundle-loader-target") }
    : environment;
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8", env });
  if (result.status !== 0) throw new Error(`${command} ${args.join(" ")} failed\n${result.stdout}\n${result.stderr}`);
}

function build(name, mode) {
  const data = path.join(temporary, `${name}-data`);
  const bundle = path.join(temporary, `${name}.sora`);
  const debug = path.join(temporary, `${name}-debug`);
  run("python", ["tools/universe-reference/fixture_workbooks.py", "--root", root, "--output", data, "--mode", mode]);
  run(sora, ["--serial", "export", "--format", "binary", "--project", project, "--data-root", data, "--out", bundle]);
  run(sora, ["--serial", "export", "--format", "json-debug", "--project", project, "--data-root", data, "--out", debug]);
  return { bundle, debug };
}

function rowCount(debug, table) {
  return JSON.parse(fs.readFileSync(path.join(debug, `${table}.json`), "utf8")).table.rows.length;
}

try {
  assert(fs.existsSync(sora), `Sora ${policy.version} is not installed`);
  const empty = build("empty", "empty");
  const first = build("representative-a", "representative");
  const second = build("representative-b", "representative");
  assert(fs.readFileSync(first.bundle).equals(fs.readFileSync(second.bundle)), "representative Sora export is not deterministic");
  assert(rowCount(empty.debug, "UniverseProfile") === 0, "empty workbook exported a profile row");
  assert(rowCount(first.debug, "UniverseProfile") === 1, "representative profile row is missing");
  assert(rowCount(first.debug, "UniverseActivityBinding") === 1, "representative Activity binding is missing");
  assert(rowCount(first.debug, "UniverseSourceRecord") === 1, "representative evidence row is missing");
  run("cargo", ["run", "--manifest-path", "tools/universe-bundle-loader/Cargo.toml", "--locked", "--quiet", "--", empty.bundle, "0", "0", "0", "0"]);
  run("cargo", ["run", "--manifest-path", "tools/universe-bundle-loader/Cargo.toml", "--locked", "--quiet", "--", first.bundle, "1", "1", "1", "1"]);
  console.log("Universe Sora fixtures verified: empty and representative exports load; representative double generation is identical.");
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}
