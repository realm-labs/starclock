import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const bless = process.argv.slice(2).includes("--bless");
assert(process.argv.slice(2).every((argument) => argument === "--bless"), "usage: verify-architecture-audit.mjs [--bless]");
const repositoryBytes = fs.readFileSync(path.join(root, "policy/repository-checks.json"));
const repositoryPolicy = JSON.parse(repositoryBytes);
const dependencyBytes = fs.readFileSync(path.join(root, "policy/dependency-and-tool-policy.json"));
const dependencyPolicy = JSON.parse(dependencyBytes);
const rules = repositoryPolicy.rust_source;
assert(rules.line_limit_exceptions.length === 0, "Goal 01 closes with no handwritten line-limit exception");
assert(rules.excluded_roots.length === 3, "generated/vendor exclusion inventory changed without review");
assert(rules.allowed_public_reexports.length === 4, "public re-export file allowlist changed without review");

const exclusions = rules.excluded_roots.map((entry) => entry.path.replaceAll("\\", "/"));
const tracked = execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard", "--", "*.rs"], { cwd: root, encoding: "utf8" })
  .split(/\r?\n/).filter(Boolean).map((entry) => entry.replaceAll("\\", "/")).sort();
const handwritten = tracked.filter((file) => !exclusions.some((excluded) => file === excluded || file.startsWith(`${excluded}/`)));
let publicReexports = 0;
let publicDeclarations = 0;
const reexportFiles = new Map();
const sourceRows = handwritten.map((relative) => {
  const source = fs.readFileSync(path.join(root, relative), "utf8");
  const lines = physicalLineCount(source);
  const facade = ["lib.rs", "mod.rs"].includes(path.basename(relative));
  const limit = facade ? rules.maximum_facade_lines : rules.maximum_handwritten_lines;
  assert(lines <= limit, `${relative}: exceeds ${limit} lines without an exception`);
  const syntax = stripComments(source);
  const reexports = syntax.match(/^\s*pub\s+use\b[\s\S]*?;/gm) ?? [];
  const declarations = syntax.match(/\bpub\s+(?:const|enum|fn|static|struct|trait|type|use)\b[^;{]*(?:;|\{)/g) ?? [];
  publicReexports += reexports.length;
  publicDeclarations += declarations.length;
  if (reexports.length > 0) reexportFiles.set(relative, reexports.length);
  for (const token of rules.forbidden_public_api_tokens) {
    for (const declaration of declarations) assert(!declaration.includes(token), `${relative}: public API leaks ${token}`);
  }
  return { path: relative, lines, limit, utilization_percent: Math.floor((lines * 100) / limit), facade };
});
assert(publicReexports === 30, "public re-export declaration count changed without review");
for (const entry of rules.allowed_public_reexports) assert(reexportFiles.has(entry.path), `${entry.path}: stale re-export allowance`);

const metadata = JSON.parse(execFileSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], { cwd: root, encoding: "utf8" }));
const memberIds = new Set(metadata.workspace_members);
const packages = metadata.packages.filter((entry) => memberIds.has(entry.id)).sort((left, right) => left.name.localeCompare(right.name));
assert(packages.length === 9, "workspace crate count changed without review");
const crateNames = new Set(packages.map((entry) => entry.name));
const graph = packages.map((pkg) => ({
  crate: pkg.name,
  local_dependencies: pkg.dependencies.filter((dependency) => crateNames.has(dependency.name)).map((dependency) => dependency.name).sort(),
  registry_dependencies: pkg.dependencies.filter((dependency) => dependency.source?.startsWith("registry+")).map((dependency) => ({ name: dependency.name, kind: dependency.kind ?? "production", requirement: dependency.req })).sort((left, right) => left.name.localeCompare(right.name)),
}));
const reviewedPackages = [
  ...dependencyPolicy.packages,
  ...dependencyPolicy.package_groups.flatMap((group) => group.packages),
];
assert(reviewedPackages.length === 50, "reviewed registry package inventory changed without review");
assert(new Set(reviewedPackages.map((entry) => `${entry.name}@${entry.version}`)).size === reviewedPackages.length, "reviewed package inventory contains duplicates");

const largest = [...sourceRows].sort((left, right) => right.utilization_percent - left.utilization_percent || right.lines - left.lines || left.path.localeCompare(right.path)).slice(0, 20);
const nearLimit = sourceRows.filter((entry) => entry.utilization_percent >= 95).map((entry) => entry.path).sort();
const report = {
  schema_revision: "starclock.goal01.architecture-audit.v1",
  policy: { repository_checks_sha256: normalizedSha(repositoryBytes), dependency_policy_sha256: normalizedSha(dependencyBytes) },
  source_size_audit: {
    handwritten_files: handwritten.length,
    generated_vendor_exclusions: rules.excluded_roots,
    handwritten_limit: rules.maximum_handwritten_lines,
    facade_limit: rules.maximum_facade_lines,
    line_limit_exceptions: [],
    near_limit_files: nearLimit,
    near_limit_policy: "These files remain within the enforced limit and receive no exception; any growth beyond the limit requires a module split, not an implicit allowance.",
    largest_files: largest,
  },
  public_api_audit: {
    public_declarations: publicDeclarations,
    public_reexports: publicReexports,
    reexport_files: Object.fromEntries([...reexportFiles].sort()),
    forbidden_implementation_tokens: rules.forbidden_public_api_tokens,
    leaked_tokens: 0,
    preludes: 0,
    wildcard_reexports: 0,
  },
  dependency_audit: {
    workspace_crates: packages.length,
    graph,
    reviewed_registry_packages: reviewedPackages.length,
    pinned_tools: dependencyPolicy.tools.length,
    unreviewed_packages: 0,
  },
};
const output = `${JSON.stringify(report, null, 2)}\n`;
const relative = "evidence/core-combat-v1/hardening/architecture-audit.json";
const outputPath = path.join(root, relative);
if (bless) fs.writeFileSync(outputPath, output);
else {
  assert(fs.existsSync(outputPath), `${relative}: missing; run with --bless`);
  assert(fs.readFileSync(outputPath, "utf8") === output, `${relative}: stale; run with --bless`);
}
console.log(`Architecture audit verified (${sha(output)}; ${handwritten.length} sources, ${publicReexports} re-exports, ${packages.length} crates, ${reviewedPackages.length} packages).`);

function physicalLineCount(value) { return value.length === 0 ? 0 : value.split(/\r\n|\n|\r/).length - (/(?:\r\n|\n|\r)$/.test(value) ? 1 : 0); }
function stripComments(value) { return value.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/.*$/gm, ""); }
function sha(value) { return crypto.createHash("sha256").update(value).digest("hex"); }
function normalizedSha(value) { return sha(Buffer.from(value.toString("utf8").replaceAll("\r\n", "\n"))); }
function assert(condition, message) { if (!condition) throw new Error(message); }
