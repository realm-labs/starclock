import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.cwd());
const coverageRoot = path.join(root, "evidence", "core-combat-v1", "coverage");
const index = read("coverage-index.json");
for (const entry of index.files) assert(sha(path.join(coverageRoot, entry.name)) === entry.sha256, `${entry.name} hash mismatch`);
assert(index.coverage_sha256 === hashText(index.files.map((entry) => `${entry.name}\0${entry.sha256}\n`).join("")), "coverage digest mismatch");
const report = read("goal-coverage.json");
assert(report.summary.required === 283 && report.summary.accounted === 283, "required goal accounting mismatch");
assert(report.summary.data_ready === 134 && report.summary.golden_verified === 134 && report.summary.enabled_incomplete === 149, "runtime readiness count differs");
assert(report.summary.terminal_state_counts.Cataloged === 149, "Cataloged terminal count mismatch");
assert(report.summary.terminal_state_counts.Documented === 0, "Documented terminal count mismatch");
assert(report.summary.terminal_state_counts.Researching === 0, "Researching terminal count mismatch");
assert(report.summary.terminal_state_counts.GoldenVerified === 134, "GoldenVerified terminal count mismatch");
assert(report.categories.length === 6 && report.entries.length === 283, "category entry mismatch");
assert(new Set(report.entries.map((entry) => `${entry.manifest_kind}\0${entry.id}`)).size === 283, "duplicate goal entry");
assert(report.entries.filter((entry) => entry.milestones.DataReady && entry.milestones.GoldenVerified && entry.data_ready_blockers.length === 0).length === 134, "production readiness differs");
assert(report.entries.filter((entry) => !entry.milestones.DataReady && !entry.milestones.GoldenVerified && entry.data_ready_blockers.length === 4).length === 149, "pending-content readiness differs");
assert(report.disabled_audit.length === 2 && report.disabled_audit.every((entry) => !entry.enabled && !entry.denominator), "disabled audit mismatch");
assert(report.accounting.missing_manifest_ids.length === 0 && report.accounting.extra_runtime_ids.length === 0 && report.accounting.duplicate_manifest_ids.length === 0 && report.accounting.stale_version_ids.length === 0, "manifest accounting issue present");
assert(report.accounting.missing_data_ready_ids.length === 149, "missing DataReady ID list mismatch");
assert(report.accounting.unowned_research_case_ids.length === 0 && report.accounting.orphaned_provenance_mappings.length === 0 && report.accounting.missing_provenance_mappings.length === 0, "evidence accounting issue present");
assert(report.documentation_assertions.all_match, "documentation counters differ from manifests");
console.log(`Goal coverage verified (${index.coverage_sha256}; 283/283 accounted; 134 DataReady; 2 disabled audit-only).`);

function read(name) { return JSON.parse(fs.readFileSync(path.join(coverageRoot, name), "utf8")); }
function sha(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function hashText(value) { return crypto.createHash("sha256").update(value, "utf8").digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
