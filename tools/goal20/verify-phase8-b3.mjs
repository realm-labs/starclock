#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";

const text = (file) => fs.readFileSync(file, "utf8");
const json = (file) => JSON.parse(text(file));
const digest = (file) => crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const assert = (condition, message) => { if (!condition) throw new Error(message); };

const phase = json("evidence/swarm-disaster-runtime-v1/audits/phase8-b3.json");
const audit = json("evidence/swarm-disaster-runtime-v1/audits/release-audits.json");
const policy = json("policy/goal20-release-audits.json");
const generated = json("policy/generated-drift.json");

assert(phase.schema_revision === "starclock.swarm-disaster-phase8-b3-evidence.v1" &&
  phase.goal_id === "swarm-disaster-runtime-v1" && phase.batch === "G20-P8-B3" &&
  phase.result === "ReleaseAuditsAndCleanCheckoutPass", "P8-B3 evidence identity drift");
assert(audit.schema_revision === "starclock.goal20-release-audits-evidence.v1" &&
  audit.result === "dependency-license-architecture-native-source-provenance-generated-prior-release-audits-pass",
"P8-B3 release-audit evidence drift");

const receipt = phase.release_audits;
assert(receipt.prior_releases === audit.prior_releases.length && receipt.prior_releases === 10 &&
  receipt.protected_reference_roots === audit.protected_reference_roots.length &&
  receipt.protected_reference_roots === 5 &&
  receipt.reviewed_registry_packages === audit.dependency_license.reviewed_registry_packages &&
  receipt.reviewed_registry_packages === 136 && receipt.new_registry_packages === 0 &&
  audit.dependency_license.new_registry_packages.length === 0,
"P8-B3 prior-release or dependency receipt drift");
assert(receipt.native_handler_scopes === audit.architecture_native_source.native_handler_scopes &&
  receipt.native_handler_scopes === 8 && receipt.admitted_native_handlers === 0 &&
  receipt.source_obligations === audit.completeness.source_obligations &&
  receipt.mechanic_rules === audit.completeness.mechanic_rules &&
  receipt.semantic_fixture_families === audit.completeness.semantic_fixture_families &&
  receipt.policy_boundaries === audit.completeness.policy_boundaries &&
  receipt.exact_once_gaps === 0 && receipt.exact_once_duplicates === 0 &&
  receipt.orphan_rules === 0 && receipt.runtime_json_file_reads === 0,
"P8-B3 architecture, source or exact-once receipt drift");
assert(receipt.generated_drift_checks === generated.checks.length &&
  receipt.generated_drift_checks === 41 && receipt.source_cache_checks === 4 &&
  generated.checks.filter((check) => check.requires === "source-cache").length === 4,
"P8-B3 generated-drift receipt drift");
assert(receipt.policy_sha256 === digest("policy/goal20-release-audits.json") &&
  receipt.evidence_sha256 === digest("evidence/swarm-disaster-runtime-v1/audits/release-audits.json"),
"P8-B3 audit digest drift");

const cached = phase.source_cache_full_gate;
assert(cached.passed === true && cached.elapsed_seconds === "147.7" &&
  cached.workspace_harnesses === 35 && cached.generated_drift_checks === 41 &&
  cached.source_cache_checks_skipped === 0, "P8-B3 source-cache full-gate receipt drift");
const clean = phase.clean_checkout_gate;
assert(clean.passed === true && clean.failed_attempts === 1 &&
  clean.first_failure === "shallow-fetch-omitted-goal08-ancestry" &&
  clean.staged_tree_verified === true &&
  clean.fresh_cargo_target === true && clean.cargo_incremental_disabled === true &&
  clean.source_cache_available === false && clean.full_repository_gate === true &&
  clean.workspace_harnesses === 35 && clean.post_gate_repository_clean === true,
"P8-B3 clean-checkout receipt drift");
const quick = phase.quick_gate;
assert(quick.passed === true && quick.elapsed_seconds === "3.5" &&
  quick.direct_packages === 0 && quick.downstream_packages === 0 && quick.deferred_inputs === 5,
"P8-B3 quick-gate receipt drift");

assert(generated.checks.some((check) => check.name === "Goal 20 release audits" &&
  JSON.stringify(check.command) === JSON.stringify(["node", "tools/goal20/verify-release-audits.mjs", "."])),
"P8-B3 release audit is absent from generated drift");
const cleanScript = text("tools/goal20/run-clean-checkout.mjs");
for (const literal of ["git\", [\"write-tree\"]", "CARGO_INCREMENTAL: \"0\"",
  "CARGO_TARGET_DIR: path.join(checkout, \"target\")", "run.mjs\", \"--full\"",
  "STARCLOCK_ARTIFACT_CHECK_ONLY: \"1\"", "\".cache\", \"tools\", \"downloads\"",
  "path.dirname(root) : os.tmpdir()", "status\", \"--porcelain\""])
  assert(cleanScript.includes(literal), `P8-B3 clean-checkout contract missing ${literal}`);

const ledger = text("docs/goals/20-swarm-disaster-runtime-status.md");
assert(ledger.includes("| Active batch | `G20-P8-B4` |") &&
  ledger.includes("| Next unblocked batch | None |") &&
  ledger.includes("| `G20-P8-B3` | `Complete` |"), "P8-B3 ledger state drift");
console.log("Goal 20 P8-B3 verified (10 releases, 5 protected roots, 136 packages, 41 generated checks, clean checkout).");
