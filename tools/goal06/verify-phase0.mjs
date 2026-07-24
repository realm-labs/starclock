import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
run("node", ["tools/goal06/verify-foundation.mjs"]);
run("node", ["tools/goal06/verify-debt-probes.mjs"]);

const replay = json("policy/goal06-replay-compatibility.json");
assert(replay.schema_revision === "starclock.goal06-replay-compatibility.v1",
  "unsupported replay compatibility policy");
assert(replay.historical_v2.format_version === 2
  && replay.historical_v2.schema_version === 1, "historical replay-v2 identity differs");
assert(replay.target_v3.format_version === 3
  && replay.target_v3.schema_version === 1, "target replay-v3 identity differs");
assert(replay.target_v3.required_nested_battle_identity.length === 6,
  "replay-v3 identity field denominator differs");
assert(new Set(replay.target_v3.first_divergence_order).size
  === replay.target_v3.first_divergence_order.length, "divergence order is ambiguous");
assert(Object.values(replay.migration_contracts).every((value) => value === true),
  "replay migration contract is not accepted");

const formatV2 = text("crates/starclock-replay/src/format_v2.rs");
for (const marker of [
  "pub const REPLAY_FORMAT_VERSION_V2: u32 = 2;",
  "pub const REPLAY_SCHEMA_VERSION_V2: u32 = 1;",
  "pub fn decode_replay_v2",
]) assert(formatV2.includes(marker), `historical replay marker is missing: ${marker}`);
assert(text("crates/starclock-mode-universe/src/universe_replay_v2.rs")
  .includes("verify_standard_universe_replay_v2"), "Universe replay-v2 verifier is missing");

const performance = json("policy/goal06-performance.json");
assert(performance.focused_wall_budget_seconds >= 60
  && performance.focused_wall_budget_seconds <= 180, "focused budget is outside 1–3 minutes");
assert(performance.workloads.length === 6, "performance workload denominator differs");
assert(new Set(performance.workloads.map((entry) => entry.id)).size
  === performance.workloads.length, "performance workload IDs are not unique");
assert(performance.terminal_limits.catalog_compositions_per_battle === 0,
  "per-battle catalog rebuild is allowed");
assert(performance.terminal_limits.cache_authoritative_fields === 0,
  "cache fields are allowed into authoritative state");

const dependencies = json("policy/goal06-dependency-baseline.json");
assert(dependencies.cargo_lock_sha256 === sha256("Cargo.lock"), "Cargo.lock baseline differs");
assert(dependencies.workspace_manifest_sha256 === sha256("Cargo.toml"),
  "workspace manifest baseline differs");
assert(dependencies.new_registry_packages === 0 && dependencies.new_direct_dependencies === 0,
  "Goal 06 added a dependency during Phase 0");
assert(Object.values(dependencies.contracts).every((value) => value === true),
  "dependency baseline contract is not accepted");

const release = json("policy/goal06-release-contract.json");
assert(release.schema_revision === "starclock.goal06-release-contract-scaffold.v1"
  && release.state === "Scaffold", "Goal 06 release scaffold differs");
assert(release.planned_batches === 18 && release.planned_phases === 5,
  "Goal 06 release denominator differs");
assert(release.required_prior_contracts.at(-1) === "standard-universe-end-to-end-v1",
  "Goal 05 is not the direct release prerequisite");

const status = text("docs/goals/06-combat-identity-and-dynamic-assembly-status.md");
assert((status.match(/^\| `G06-P[0-4]-B\d+` \|/gmu) ?? []).length === 18,
  "Goal 06 status batch denominator differs");
assert((status.match(/^\| `G06-P0-B[1-3]` \| `Complete` \|/gmu) ?? []).length === 3,
  "Goal 06 Phase 0 is not complete");
assert(status.includes("| Next unblocked batch | `G06-P1-B1` |"),
  "Goal 06 next batch differs");

console.log("Goal 06 Phase 0 verified (v2/v3 compatibility, 6 workloads, no dependency drift).");

function run(command, args) {
  execFileSync(command, args, { cwd: root, stdio: "inherit" });
}
function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
