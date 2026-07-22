import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const root = path.resolve(process.argv[2] ?? ".");
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy", "sora-toolchain.json"), "utf8"));
const sora = path.join(root, policy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
const project = path.join(root, "config", "universe-project.toml");
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "starclock-universe-production-"));
const universeFiles = new Set(["Universe.xlsx", "UniverseBindings.xlsx", "UniverseEvidence.xlsx"]);
const assert = (condition, message) => { if (!condition) throw new Error(message); };

function run(command, args) {
  const environment = { ...process.env, PYTHONDONTWRITEBYTECODE: "1" };
  if (command === "cargo") environment.CARGO_TARGET_DIR = path.join(root, ".cache", "universe-bundle-loader-target");
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8", env: environment });
  if (result.status !== 0) throw new Error(`${command} ${args.join(" ")} failed\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function hashTree(directory) {
  return Object.fromEntries(fs.readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .sort((left, right) => left.name.localeCompare(right.name))
    .map((entry) => [entry.name, sha256(path.join(directory, entry.name))]));
}

function build(label) {
  const data = path.join(temporary, `${label}-data`);
  const debug = path.join(temporary, `${label}-debug`);
  const bundle = path.join(temporary, `${label}.sora`);
  fs.mkdirSync(data);
  run("python", ["tools/universe-reference/author_workbooks.py", "--root", root, "--output", data]);
  run(sora, ["--serial", "export", "--format", "binary", "--project", project, "--data-root", data, "--out", bundle]);
  run(sora, ["--serial", "export", "--format", "json-debug", "--project", project, "--data-root", data, "--out", debug]);
  run("python", ["tools/universe-reference/verify_workbooks.py", "--root", root, "--data-root", data, "--debug-root", debug]);
  run("cargo", ["run", "--manifest-path", "tools/universe-bundle-loader/Cargo.toml", "--locked", "--quiet", "--", bundle, "1", "9", "9", "2645"]);
  return { data, debug, bundle };
}

try {
  const first = build("a");
  const second = build("b");
  const firstWorkbooks = Object.fromEntries([...universeFiles].sort().map((name) => [name, sha256(path.join(first.data, name))]));
  const secondWorkbooks = Object.fromEntries([...universeFiles].sort().map((name) => [name, sha256(path.join(second.data, name))]));
  assert(JSON.stringify(firstWorkbooks) === JSON.stringify(secondWorkbooks), "double-generated workbook bytes differ");
  assert(fs.readFileSync(first.bundle).equals(fs.readFileSync(second.bundle)), "double-generated Sora bundles differ");
  assert(JSON.stringify(hashTree(first.debug)) === JSON.stringify(hashTree(second.debug)), "double-generated debug tables differ");
  assert(fs.readFileSync(first.bundle).equals(fs.readFileSync(path.join(root, "config", "universe-generated", "config.sora"))), "committed Universe Sora bundle differs from regeneration");
  console.log(`Production Universe workbooks verified: ${JSON.stringify(firstWorkbooks)}; bundle ${sha256(first.bundle)}.`);
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}
