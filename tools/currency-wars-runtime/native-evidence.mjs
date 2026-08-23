#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policyPath = path.join(root, "policy/currency-wars-native-evidence.json");
const target = argument("--target");
const output = optionalArgument("--output");
const write = process.argv.includes("--write");
assert(target !== null,
  "usage: native-evidence.mjs --target <triple> [--write] [--output <path>]");
assert(write || process.argv.includes("--check"), "select --check or --write");

const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "starclock-currency-wars-native-"));
try {
  const equality = {
    matrix_sha256: sha256File(
      "content-manifests/currency-wars-runtime-v1/coverage-and-release.json",
    ),
    runtime_contract_sha256: sha256File(
      "content-manifests/currency-wars-runtime-v1/runtime-contract.json",
    ),
    exact_coverage_sha256: sha256File(
      "content-manifests/currency-wars-runtime-v1/exact-runtime-coverage-audit.json",
    ),
    standard: executeGolden("standard", path.join(temporary, "standard.scrp")),
    overclock: executeGolden("overclock", path.join(temporary, "overclock.scrp")),
  };
  if (write) {
    const policy = {
      schema_revision: "starclock.currency-wars-native-evidence.v1",
      goal_id: "currency-wars-runtime-v1",
      runtime_targets: [
        "x86_64-pc-windows-msvc",
        "x86_64-unknown-linux-gnu",
        "aarch64-apple-darwin",
      ],
      compile_only_targets: [
        "aarch64-pc-windows-msvc",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
      ],
      equality,
    };
    fs.writeFileSync(policyPath, pretty(policy));
  }
  const policy = JSON.parse(fs.readFileSync(policyPath, "utf8"));
  assert(policy.runtime_targets.includes(target), `unreviewed native target: ${target}`);
  assert(JSON.stringify(equality) === JSON.stringify(policy.equality),
    `${target} Currency Wars native evidence differs from the frozen golden`);
  const report = {
    schema_revision: "starclock.currency-wars-native-run.v1",
    target,
    result: "Pass",
    equality,
  };
  if (output !== null) {
    const destination = path.resolve(root, output);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, pretty(report));
  }
  console.log(
    `Currency Wars native evidence verified for ${target} `
      + `(${equality.standard.replay_sha256.slice(0, 16)}).`,
  );
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

function executeGolden(gambit, replayPath) {
  const run = execute([
    "currency-wars", "run",
    "--route", "801",
    "--difficulty", "1",
    "--gambit", gambit,
    "--seed", "31000501",
    "--controller", "baseline",
    "--replay-out", replayPath,
    "--json",
  ]);
  const verify = execute(["replay", "verify", replayPath, "--json"]);
  return {
    run_report_sha256: canonicalSha256(JSON.parse(run)),
    replay_sha256: sha256Bytes(fs.readFileSync(replayPath)),
    verification_report_sha256: canonicalSha256(JSON.parse(verify)),
  };
}

function execute(args) {
  const result = spawnSync(
    "cargo",
    ["run", "--quiet", "--release", "-p", "starclock-cli", "--", ...args],
    { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  );
  if (result.error) throw result.error;
  assert(result.status === 0,
    `starclock ${args.join(" ")} failed:\n${result.stderr}`);
  return result.stdout.trim();
}

function canonicalSha256(value) {
  return sha256Bytes(Buffer.from(JSON.stringify(canonical(value))));
}

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(Object.keys(value).sort()
      .map((key) => [key, canonical(value[key])]));
  }
  return value;
}

function sha256File(relativePath) {
  return sha256Bytes(fs.readFileSync(path.join(root, relativePath)));
}

function sha256Bytes(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function argument(name) {
  const index = process.argv.indexOf(name);
  return index < 0 ? null : process.argv[index + 1] ?? null;
}

function optionalArgument(name) {
  return argument(name);
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
