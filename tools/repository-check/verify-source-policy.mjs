import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = readJson(path.join(root, "policy/repository-checks.json"));
assert(policy.schema_revision === "starclock.repository-checks.v1", "unsupported repository-check policy revision");
const rules = policy.rust_source;
assert(Number.isInteger(rules.maximum_handwritten_lines) && rules.maximum_handwritten_lines > 0, "invalid handwritten line limit");
assert(Number.isInteger(rules.maximum_facade_lines) && rules.maximum_facade_lines > 0, "invalid facade line limit");

const exclusions = new Map();
for (const exclusion of rules.excluded_roots) {
  const relative = validateRelativePath(exclusion.path, "excluded root");
  assert(!exclusions.has(relative), `${relative}: duplicate excluded root`);
  assert(rules.roots.some((sourceRoot) => relative.startsWith(`${normalize(sourceRoot)}/`)), `${relative}: exclusion must be narrower than a Rust source root`);
  assert(fs.statSync(path.join(root, relative), { throwIfNoEntry: false })?.isDirectory(), `${relative}: excluded root must be an existing directory`);
  assert(["generated", "vendor"].includes(exclusion.kind), `${relative}: exclusion kind must be generated or vendor`);
  assert(nonEmpty(exclusion.reason), `${relative}: exclusion requires a review reason`);
  exclusions.set(relative, exclusion);
}

const exceptions = new Map();
for (const exception of rules.line_limit_exceptions) {
  const relative = validateRelativePath(exception.path, "line-limit exception");
  assert(!exceptions.has(relative), `${relative}: duplicate line-limit exception`);
  assert(nonEmpty(exception.reason), `${relative}: line-limit exception requires a review reason`);
  exceptions.set(relative, exception);
}

const reexportAllowlist = new Map();
for (const entry of rules.allowed_public_reexports) {
  const relative = validateRelativePath(entry.path, "public re-export allowlist");
  assert(!reexportAllowlist.has(relative), `${relative}: duplicate public re-export allowlist entry`);
  assert(nonEmpty(entry.reason), `${relative}: public re-export allowlist entry requires a review reason`);
  reexportAllowlist.set(relative, entry.reason);
}

const rustFiles = [];
for (const sourceRoot of rules.roots) {
  const relative = validateRelativePath(sourceRoot, "Rust source root");
  walk(path.join(root, relative), rustFiles);
}
rustFiles.sort((left, right) => left.localeCompare(right));

let publicReexportCount = 0;
const reexportFiles = new Set();
for (const relative of rustFiles) {
  const absolute = path.join(root, relative);
  const source = fs.readFileSync(absolute, "utf8");
  const lines = physicalLineCount(source);
  const basename = path.basename(relative);
  const limit = basename === "lib.rs" || basename === "mod.rs"
    ? rules.maximum_facade_lines
    : rules.maximum_handwritten_lines;
  if (lines > limit) {
    const exception = exceptions.get(relative);
    assert(exception, `${relative}: ${lines} physical lines exceeds the ${limit}-line limit without a reviewed exception`);
    assert(source.startsWith("//! Line-limit exception:"), `${relative}: reviewed exception requires a module-level explanation`);
  }

  const segments = relative.split("/");
  assert(!segments.includes("prelude") && basename !== "prelude.rs", `${relative}: project prelude modules are forbidden`);

  const syntax = stripLineComments(source);
  assert(!/^\s*pub\s+(?:\([^)]*\)\s+)?mod\s+prelude\b/m.test(syntax), `${relative}: public prelude modules are forbidden`);
  const publicUses = syntax.match(/^\s*pub\s+use\b[\s\S]*?;/gm) ?? [];
  if (publicUses.length > 0) {
    publicReexportCount += publicUses.length;
    reexportFiles.add(relative);
    assert(reexportAllowlist.has(relative), `${relative}: public re-exports require an explicit reviewed allowlist entry`);
  }
  for (const declaration of publicUses) {
    assert(!/::\s*\*|\{\s*\*|,\s*\*|\*\s*\}/.test(declaration), `${relative}: wildcard public re-exports are forbidden`);
    const sourceModule = declaration.match(/\bpub\s+use\s+(?:crate::)?([A-Za-z_][A-Za-z0-9_]*)::/)?.[1];
    if (sourceModule) {
      const publicModule = new RegExp(`^\\s*pub\\s+mod\\s+${escapeRegex(sourceModule)}\\s*;`, "m");
      assert(!publicModule.test(syntax), `${relative}: ${sourceModule} would have two canonical public paths`);
    }
  }

  const publicDeclarations = syntax.match(/\bpub\s+(?:const|enum|fn|static|struct|trait|type|use)\b[^;{]*(?:;|\{)/g) ?? [];
  for (const declaration of publicDeclarations) {
    for (const token of rules.forbidden_public_api_tokens) {
      assert(!declaration.includes(token), `${relative}: public declaration exposes forbidden implementation token ${token}`);
    }
  }
}

for (const allowedPath of reexportAllowlist.keys()) {
  assert(rustFiles.includes(allowedPath), `${allowedPath}: public re-export allowlist path does not exist`);
  assert(reexportFiles.has(allowedPath), `${allowedPath}: stale public re-export allowlist entry`);
}
for (const exceptionPath of exceptions.keys()) {
  assert(rustFiles.includes(exceptionPath), `${exceptionPath}: stale line-limit exception`);
}

console.log(`Rust source policy verified (${rustFiles.length} handwritten files, ${publicReexportCount} explicit public re-export declarations, ${exclusions.size} explicit generated/vendor exclusions).`);

function walk(directory, output) {
  assert(fs.existsSync(directory), `${normalize(path.relative(root, directory))}: source root does not exist`);
  for (const entry of fs.readdirSync(directory, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name))) {
    const absolute = path.join(directory, entry.name);
    const relative = normalize(path.relative(root, absolute));
    if (entry.isDirectory()) {
      if (!isExcluded(relative)) walk(absolute, output);
    } else if (entry.isFile() && entry.name.endsWith(".rs") && !isExcluded(relative)) {
      output.push(relative);
    }
  }
}

function isExcluded(relative) {
  return [...exclusions.keys()].some((excluded) => relative === excluded || relative.startsWith(`${excluded}/`));
}
function physicalLineCount(value) {
  if (value.length === 0) return 0;
  const count = value.split(/\r\n|\n|\r/).length;
  return /(?:\r\n|\n|\r)$/.test(value) ? count - 1 : count;
}
function stripLineComments(value) { return value.replace(/\/\/.*$/gm, ""); }
function normalize(value) { return value.replaceAll("\\", "/"); }
function nonEmpty(value) { return typeof value === "string" && value.trim().length > 0; }
function escapeRegex(value) { return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function validateRelativePath(value, label) {
  assert(nonEmpty(value), `${label} must be a non-empty path`);
  const relative = normalize(value);
  assert(!path.isAbsolute(value) && relative !== "." && !relative.startsWith("../") && !relative.includes("/../"), `${label} must remain inside the repository`);
  return relative;
}
function assert(condition, message) { if (!condition) throw new Error(message); }
