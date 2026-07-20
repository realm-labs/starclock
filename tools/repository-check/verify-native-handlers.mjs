import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = readJson("policy/native-handler-audit.json");
assert(policy.schema_revision === "starclock.native-handler-audit.v1", "unsupported native-handler audit revision");
assert(policy.registry_revision === "native-registry-v1", "unexpected native-handler registry revision");
assert(Array.isArray(policy.admitted_handlers), "admitted_handlers must be an array");
assert(Array.isArray(policy.v1a_reviews), "v1a_reviews must be an array");

const productionRows = readJson("config/generated/debug-json/NativeHandler.json").table.rows;
assert(Array.isArray(productionRows), "production NativeHandler diagnostic has no row list");
assert(productionRows.length === policy.admitted_handlers.length, "production NativeHandler rows disagree with the admitted-handler audit");

const registrySource = read("crates/starclock-rules/src/registry.rs");
assert(registrySource.includes(`PRODUCTION_REGISTRY_REVISION: &str = "${policy.registry_revision}"`), "compiled registry revision disagrees with policy");
const declaredCount = Number(registrySource.match(/PRODUCTION_BATTLE_HANDLERS:\s*\[BattleHandlerRegistration;\s*(\d+)\]/)?.[1]);
assert(Number.isInteger(declaredCount), "cannot locate the compiled production handler list");
assert(declaredCount === policy.admitted_handlers.length, "compiled production handler count disagrees with policy");

const probeRoot = absolute("config/probes/v1a");
const probeNames = fs.readdirSync(probeRoot, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort();
const reviews = [...policy.v1a_reviews].sort((left, right) => left.probe.localeCompare(right.probe));
assert(new Set(reviews.map((review) => review.probe)).size === reviews.length, "duplicate V1a native-handler review");
assert(JSON.stringify(reviews.map((review) => review.probe)) === JSON.stringify(probeNames), "V1a review inventory does not exactly cover the probe directories");
for (const review of reviews) {
  assert(review.outcome === "ir-sufficient", `${review.probe}: only a reviewed IR-sufficient outcome is allowed without a handler`);
  assert(nonEmpty(review.decision), `${review.probe}: missing IR-sufficiency decision`);
  assert(Array.isArray(review.forms) && review.forms.length > 0 && review.forms.every(nonEmpty), `${review.probe}: missing reviewed form list`);
  assert(Array.isArray(review.evidence) && review.evidence.length > 0, `${review.probe}: missing evidence list`);
  for (const evidence of review.evidence) assert(fs.statSync(absolute(evidence), { throwIfNoEntry: false })?.isFile(), `${review.probe}: missing evidence ${evidence}`);
  assert(Array.isArray(review.admitted_handler_ids) && review.admitted_handler_ids.length === 0, `${review.probe}: handler admission requires a non-IR-sufficient review`);
  auditProbeRows(review.probe);
}

const branchPolicy = policy.content_branch_audit;
for (const auditRoot of branchPolicy.roots) assert(fs.statSync(absolute(auditRoot), { throwIfNoEntry: false })?.isDirectory(), `${auditRoot}: branch-audit root does not exist`);
const tracked = execFileSync("git", ["ls-files", "--", ...branchPolicy.roots.map((auditRoot) => `${auditRoot}/*.rs`), ...branchPolicy.roots.map((auditRoot) => `${auditRoot}/**/*.rs`)], { cwd: root, encoding: "utf8" })
  .split(/\r?\n/).filter(Boolean).map(normalize).sort();
assert(tracked.length > 0, "content-branch audit selected no Rust sources");
for (const relative of tracked) auditRust(relative, branchPolicy);

console.log(`Native-handler audit verified (${reviews.length} V1a scopes, ${policy.admitted_handlers.length} admitted handlers, ${tracked.length} core/registry Rust files).`);

function auditProbeRows(probe) {
  const rowsRoot = absolute(`config/probes/v1a/${probe}/rows`);
  const nativePath = path.join(rowsRoot, "NativeHandler.tsv");
  if (fs.existsSync(nativePath)) assert(parseTsv(fs.readFileSync(nativePath, "utf8")).length === 0, `${probe}: probe admits a NativeHandler row`);
  const rulePath = path.join(rowsRoot, "RuleDefinition.tsv");
  if (fs.existsSync(rulePath)) {
    for (const row of parseTsv(fs.readFileSync(rulePath, "utf8"))) assert(!nonEmpty(row.native_handler_id), `${probe}: rule ${row.id} binds a native handler`);
  }
  const operationPath = path.join(rowsRoot, "Operation.tsv");
  if (fs.existsSync(operationPath)) {
    for (const row of parseTsv(fs.readFileSync(operationPath, "utf8"))) assert(!/InvokeNativeHandler/.test(row.payload ?? ""), `${probe}: operation ${row.id} invokes a native handler`);
  }
}

function auditRust(relative, audit) {
  const source = read(relative);
  const syntax = stripRustTrivia(source);
  for (const token of audit.forbidden_content_type_tokens) assert(!new RegExp(`\\b${escapeRegex(token)}\\b`).test(source), `${relative}: core source names content type ${token}`);
  for (const symbol of audit.forbidden_v1a_symbols) assert(!new RegExp(`\\b${escapeRegex(symbol)}\\b`, "i").test(source), `${relative}: core source names V1a content symbol ${symbol}`);
  const idType = `(?:${audit.definition_id_types.map(escapeRegex).join("|")})`;
  for (const expression of controlExpressions(syntax)) {
    assert(!new RegExp(`\\b${idType}\\s*::\\s*(?:new|try_from)\\s*\\(\\s*\\d+`).test(expression), `${relative}: control flow branches on a hard-coded definition ID`);
    assert(!/\b(?:ability|effect|handler|rule|source|unit_definition)(?:_id)?\s*\.\s*get\s*\(\s*\)\s*(?:==|!=|<=|>=|<|>)\s*\d+/.test(expression), `${relative}: control flow branches on a numeric content identity`);
    assert(!/\d+\s*(?:==|!=|<=|>=|<|>)\s*\b(?:ability|effect|handler|rule|source|unit_definition)(?:_id)?\s*\.\s*get\s*\(\s*\)/.test(expression), `${relative}: control flow branches on a numeric content identity`);
  }
}

function controlExpressions(source) {
  const output = [];
  const pattern = /\b(?:if|match)\b/g;
  for (let found; (found = pattern.exec(source));) {
    let round = 0;
    let square = 0;
    let index = pattern.lastIndex;
    for (; index < source.length; index += 1) {
      const character = source[index];
      if (character === "(") round += 1;
      else if (character === ")") round = Math.max(0, round - 1);
      else if (character === "[") square += 1;
      else if (character === "]") square = Math.max(0, square - 1);
      else if (character === "{" && round === 0 && square === 0) break;
      else if (character === ";" && round === 0 && square === 0) break;
    }
    output.push(source.slice(found.index, index));
  }
  return output;
}

function stripRustTrivia(source) {
  let output = "";
  let state = "code";
  let blockDepth = 0;
  for (let index = 0; index < source.length; index += 1) {
    const here = source[index];
    const next = source[index + 1];
    if (state === "code" && here === "/" && next === "/") { state = "line"; output += "  "; index += 1; continue; }
    if (state === "code" && here === "/" && next === "*") { state = "block"; blockDepth = 1; output += "  "; index += 1; continue; }
    if (state === "block" && here === "/" && next === "*") { blockDepth += 1; output += "  "; index += 1; continue; }
    if (state === "block" && here === "*" && next === "/") { blockDepth -= 1; if (blockDepth === 0) state = "code"; output += "  "; index += 1; continue; }
    if (state === "code" && here === '"') { state = "string"; output += " "; continue; }
    if (state === "string" && here === "\\") { output += "  "; index += 1; continue; }
    if (state === "string" && here === '"') { state = "code"; output += " "; continue; }
    if (state === "line" && (here === "\n" || here === "\r")) { state = "code"; output += here; continue; }
    output += state === "code" ? here : (here === "\n" || here === "\r" ? here : " ");
  }
  return output;
}

function parseTsv(value) {
  const lines = value.trim().split(/\r?\n/);
  if (lines.length === 0 || !lines[0]) return [];
  const headers = lines[0].split("\t");
  return lines.slice(1).filter((line) => line.length > 0).map((line) => Object.fromEntries(headers.map((header, index) => [header, line.split("\t")[index] ?? ""])));
}
function readJson(relative) { return JSON.parse(read(relative)); }
function read(relative) { return fs.readFileSync(absolute(relative), "utf8"); }
function absolute(relative) { return path.join(root, relative); }
function normalize(value) { return value.replaceAll("\\", "/"); }
function nonEmpty(value) { return typeof value === "string" && value.trim().length > 0; }
function escapeRegex(value) { return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
