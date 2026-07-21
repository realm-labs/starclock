import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = parseArguments(process.argv.slice(2));
const policyPath = path.join(root, "policy/ci-matrix.json");
const policyBytes = fs.readFileSync(policyPath);
const policy = JSON.parse(policyBytes.toString("utf8"));
const profiles = [...policy.native_profiles, ...policy.compile_only_profiles];
const profile = profiles.find((candidate) => candidate.id === arguments_.profile);
assert(profile, `unknown CI profile: ${arguments_.profile}`);

const rustVerbose = capture("rustc", ["-vV"]);
const rustHost = /^host: (.+)$/mu.exec(rustVerbose)?.[1];
assert(rustHost === profile.host, `rustc host ${rustHost} does not match ${profile.host}`);
assert(process.platform === profile.node_platform, `Node platform ${process.platform} does not match ${profile.node_platform}`);
assert(process.arch === profile.node_arch, `Node architecture ${process.arch} does not match ${profile.node_arch}`);

if (process.env.GITHUB_ACTIONS === "true") {
  assert(process.env.RUNNER_OS === profile.runner_os, `RUNNER_OS ${process.env.RUNNER_OS} does not match ${profile.runner_os}`);
  assert(process.env.RUNNER_ARCH === profile.runner_arch, `RUNNER_ARCH ${process.env.RUNNER_ARCH} does not match ${profile.runner_arch}`);
}

const expectedManifest = JSON.parse(fs.readFileSync(path.join(root, "config/sora-golden/expected-manifest.json"), "utf8"));
const evidence = {
  schema_revision: "starclock.ci-evidence.v1",
  profile: profile.id,
  execution_mode: profile.execution_mode,
  tests_executed: profile.tests_executed,
  runner: {
    label: profile.runner,
    os: process.env.RUNNER_OS ?? os.platform(),
    architecture: process.env.RUNNER_ARCH ?? os.arch(),
    image_os: process.env.ImageOS ?? null,
    image_version: process.env.ImageVersion ?? null
  },
  revision: {
    commit: process.env.GITHUB_SHA ?? capture("git", ["rev-parse", "HEAD"]),
    workflow_run_id: process.env.GITHUB_RUN_ID ?? null,
    workflow_run_attempt: process.env.GITHUB_RUN_ATTEMPT ?? null
  },
  toolchain: {
    rust_host: rustHost,
    rustc: firstLine(rustVerbose),
    cargo: capture("cargo", ["--version"]),
    node: process.version
  },
  target: profile.target,
  evidence_origin: process.env.GITHUB_ACTIONS === "true" ? "hosted-ci" : "local-smoke",
  policy_sha256: sha256(Buffer.from(policyBytes.toString("utf8").replaceAll("\r\n", "\n"))),
  sora_golden_output_digest: expectedManifest.output_digest,
  golden_suite_contract_sha256: sha256(Buffer.from(JSON.stringify(policy.golden_suites))),
  golden_suites: policy.golden_suites.map((suite) => ({
    id: suite.id,
    disposition: profile.execution_mode === "native" ? "executed" : "compiled-not-executed"
  }))
};

if (profile.execution_mode === "native") {
  const soraPolicy = JSON.parse(fs.readFileSync(path.join(root, "policy/sora-toolchain.json"), "utf8"));
  const soraBinary = path.join(root, soraPolicy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
  evidence.gate = policy.repository_gate;
  evidence.toolchain.sora = capture(soraBinary, ["--version"]);
} else {
  const installedTargets = capture("rustup", ["target", "list", "--installed", "--toolchain", "1.97.0"])
    .split(/\r?\n/u)
    .filter(Boolean);
  assert(installedTargets.includes(profile.target), `compile target ${profile.target} is not installed`);
  evidence.gate = `cargo check --workspace --all-targets --all-features --target ${profile.target}`;
  evidence.installed_target = true;
}

const output = path.resolve(root, arguments_.output);
assert(output.startsWith(path.join(root, ".ci-evidence") + path.sep), "evidence output must remain under .ci-evidence");
fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(output, `${JSON.stringify(evidence, null, 2)}\n`);
console.log(`Wrote ${path.relative(root, output).replaceAll("\\", "/")}.`);

function parseArguments(values) {
  const parsed = {};
  for (let index = 0; index < values.length; index += 2) {
    const name = values[index];
    const value = values[index + 1];
    assert(["--profile", "--output"].includes(name) && value, "usage: write-evidence.mjs --profile ID --output .ci-evidence/FILE.json");
    parsed[name.slice(2)] = value;
  }
  assert(parsed.profile && parsed.output, "usage: write-evidence.mjs --profile ID --output .ci-evidence/FILE.json");
  return parsed;
}

function capture(command, args) {
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} exited with ${result.status}: ${result.stderr}`);
  return result.stdout.trim();
}

function firstLine(value) { return value.split(/\r?\n/u)[0]; }
function sha256(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
