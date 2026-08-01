#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";

const text = (path) => fs.readFileSync(path, "utf8");
const json = (path) => JSON.parse(text(path));
const digest = (path) => crypto.createHash("sha256").update(fs.readFileSync(path)).digest("hex");
const assert = (condition, message) => { if (!condition) throw new Error(message); };

const phase = json("evidence/gold-and-gears-runtime-v1/audits/phase8-b3.json");
const audit = json("evidence/gold-and-gears-runtime-v1/audits/release-audits.json");
const policy = json("policy/goal14-release-audits.json");
const generated = json("policy/generated-drift.json");

assert(phase.schema_revision === "starclock.gold-and-gears-phase8-b3-evidence.v1" &&
  phase.goal_id === "gold-and-gears-runtime-v1" && phase.batch === "G14-P8-B3" &&
  phase.result === "ReleaseAuditsAndCleanCheckoutPass", "P8-B3 evidence identity drift");
assert(audit.schema_revision === "starclock.goal14-release-audits-evidence.v1" &&
  audit.result === "dependency-license-architecture-native-source-provenance-generated-prior-release-audits-pass",
"P8-B3 release-audit evidence drift");

const receipt = phase.release_audits;
assert(receipt.prior_releases === audit.prior_releases.length && receipt.prior_releases === 8 &&
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
  receipt.exact_once_gaps === 0 && receipt.exact_once_duplicates === 0 &&
  receipt.orphan_rules === 0 && receipt.runtime_json_file_reads === 0,
"P8-B3 architecture, source or exact-once receipt drift");
assert(receipt.generated_drift_checks === generated.checks.length &&
  receipt.generated_drift_checks === 35 && receipt.source_cache_checks === 4 &&
  generated.checks.filter((check) => check.requires === "source-cache").length === 4,
"P8-B3 generated-drift receipt drift");
assert(receipt.policy_sha256 === digest("policy/goal14-release-audits.json") &&
  receipt.evidence_sha256 === digest("evidence/gold-and-gears-runtime-v1/audits/release-audits.json"),
"P8-B3 audit digest drift");

const cached = phase.source_cache_full_gate;
assert(cached.passed === true && cached.elapsed_seconds === "225.3" &&
  cached.workspace_harnesses === 34 && cached.generated_drift_checks === 35 &&
  cached.source_cache_checks_skipped === 0, "P8-B3 source-cache full-gate receipt drift");
const clean = phase.clean_checkout_gate;
assert(clean.passed === true && clean.staged_tree_verified === true &&
  clean.fresh_cargo_target === true && clean.cargo_incremental_disabled === true &&
  clean.source_cache_available === false && clean.full_repository_gate === true &&
  clean.workspace_harnesses === 34 && clean.post_gate_repository_clean === true,
"P8-B3 clean-checkout receipt drift");
const quick = phase.quick_gate;
assert(quick.passed === true && quick.elapsed_seconds === "5.4" &&
  quick.direct_packages === 0 && quick.downstream_packages === 0 && quick.deferred_inputs === 5,
"P8-B3 quick-gate receipt drift");

assert(generated.checks.some((check) => check.name === "Goal 14 release audits" &&
  JSON.stringify(check.command) === JSON.stringify(["node", "tools/goal14/verify-release-audits.mjs", "."])),
"P8-B3 release audit is absent from generated drift");
const cleanScript = text("tools/goal14/run-clean-checkout.mjs");
for (const literal of ["git\", [\"write-tree\"]", "CARGO_INCREMENTAL: \"0\"",
  "CARGO_TARGET_DIR: path.join(checkout, \"target\")", "run.mjs\", \"--full\"",
  "STARCLOCK_ARTIFACT_CHECK_ONLY: \"1\"", "\".cache\", \"tools\", \"downloads\"",
  "path.dirname(root) : os.tmpdir()", "status\", \"--porcelain\""])
  assert(cleanScript.includes(literal), `P8-B3 clean-checkout contract missing ${literal}`);

const ledger = text("docs/goals/14-gold-and-gears-runtime-status.md");
assert(ledger.includes("| Active batch | None |") &&
  ledger.includes("| Next unblocked batch | `G14-P8-B4` |") &&
  ledger.includes("| `G14-P8-B3` | `Complete` |"), "P8-B3 ledger state drift");
console.log("Goal 14 P8-B3 verified (8 releases, 5 protected roots, 136 packages, 35 generated checks, clean checkout)." );
