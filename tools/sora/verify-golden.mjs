import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const arguments_ = process.argv.slice(2);
assert(arguments_.every((argument) => argument === "--bless"), "usage: node tools/sora/verify-golden.mjs [--bless]");
const bless = arguments_.includes("--bless");
const fixture = path.join(root, "config", "sora-golden");
const work = path.join(root, ".cache", "sora-golden-work");
const expected = path.join(fixture, "expected");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/sora-golden-work", "unexpected Sora work path");
assert(path.relative(root, expected).replaceAll("\\", "/") === "config/sora-golden/expected", "unexpected Sora golden path");

const toolPolicy = readJson(path.join(root, "policy", "sora-toolchain.json"));
const readerPolicy = readJson(path.join(root, "policy", "sora-golden-reader-dependencies.json"));
const capability = readJson(path.join(fixture, "capability-lock.json"));
assert(toolPolicy.schema_revision === "starclock.sora-toolchain.v1", "unsupported Sora tool policy revision");
assert(capability.schema_revision === "starclock.sora-capability-lock.v1", "unsupported Sora capability lock revision");
assert(capability.sora_cli_version === toolPolicy.version, "capability and tool versions differ");
assert(capability.crate_archive_sha256 === toolPolicy.crate_sha256, "capability and archive checksums differ");
assert(capability.upstream_tag_object === toolPolicy.tag_object && capability.upstream_tag_commit === toolPolicy.tag_commit, "capability and upstream tag identities differ");
assert(readerPolicy.schema_revision === "starclock.sora-golden-reader-dependencies.v1", "unsupported golden-reader dependency policy revision");
for (const field of ["scope", "purpose", "deterministic_impact", "compile_cost", "rejected_alternatives"]) assert(readerPolicy[field] && (!Array.isArray(readerPolicy[field]) || readerPolicy[field].length > 0), `golden-reader policy lacks ${field}`);
assert(readerPolicy.source_url_template === "https://crates.io/crates/{name}/{version}" && readerPolicy.relationships, "golden-reader source/relationship policy is incomplete");
assert(sha256(path.join(fixture, "reader", "Cargo.lock")) === readerPolicy.lockfile_sha256, "golden-reader Cargo.lock digest differs from policy");

const sora = resolveSora(toolPolicy);
assert(runCapture(sora, ["--version"]).stdout.trim() === capability.version_output, `installed Sora is not ${capability.version_output}`);
const cachedArchive = path.join(root, ".cache", "tools", "downloads", `${toolPolicy.package}-${toolPolicy.version}.crate`);
assert(fs.existsSync(cachedArchive), `missing checksum-bound archive; run ${toolPolicy.install_command}`);
assert(sha256(cachedArchive) === toolPolicy.crate_sha256, "cached Sora archive checksum differs");

fs.rmSync(work, { recursive: true, force: true });
fs.mkdirSync(path.join(work, "schema"), { recursive: true });
fs.mkdirSync(path.join(work, "data"), { recursive: true });
fs.mkdirSync(path.join(work, "reader"), { recursive: true });
for (const name of ["project.toml", "project-before.toml", "project-unsigned.toml"]) fs.copyFileSync(path.join(fixture, name), path.join(work, name));
fs.cpSync(path.join(fixture, "schema"), path.join(work, "schema"), { recursive: true });
fs.cpSync(path.join(fixture, "data"), path.join(work, "data"), { recursive: true });
for (const name of ["main.rs", "Cargo.toml", "Cargo.lock"]) fs.copyFileSync(path.join(fixture, "reader", name), path.join(work, "reader", name));
verifyReaderDependencies(readerPolicy);

run(sora, ["--serial", "excel-template", "--project", "./project-before.toml", "--out", "data"]);
const workbook = path.join(work, "data", "Golden.xlsx");
assert(fs.existsSync(workbook), "excel-template did not create Golden.xlsx");
const beforeSync = sha256(workbook);
const preview = runCapture(sora, ["--serial", "excel-sync", "--project", "./project.toml", "--data-root", "data"]);
assert(sha256(workbook) === beforeSync, "excel-sync preview mutated the workbook");
assert(`${preview.stdout}\n${preview.stderr}`.includes("note"), "excel-sync preview did not report the added note field");
run(sora, ["--serial", "excel-sync", "--project", "./project.toml", "--data-root", "data", "--write"]);
assert(sha256(workbook) !== beforeSync, "excel-sync --write did not update the workbook");

run(sora, ["--serial", "check", "--project", "./project.toml"]);
run(sora, ["--serial", "build", "--project", "./project.toml", "--clean"]);
formatRust(path.join(work, "reader", "generated"));
assertWorkbookSemantics(path.join(work, "generated", "excel"));
const firstBuild = stableArtifactHashes(work);
assert(firstBuild.size > 5, "configured build did not emit the expected artifact families");
verifyUnsupportedAssumptions();

const direct = path.join(work, "direct");
run(sora, ["--serial", "schema-lock", "--project", "./project.toml", "--out", "direct/schema.lock"]);
run(sora, ["--serial", "excel-template", "--project", "./project.toml", "--out", "direct/excel"]);
run(sora, ["--serial", "gen", "--target", "rust", "--project", "./project.toml", "--out", "direct/rust", "--format-code", "never"]);
formatRust(path.join(direct, "rust"));
run(sora, ["--serial", "export", "--format", "binary", "--project", "./project.toml", "--data-root", "data", "--out", "direct/config.sora"]);
run(sora, ["--serial", "export", "--format", "json-debug", "--project", "./project.toml", "--data-root", "data", "--out", "direct/debug-json"]);
assertSameFile(path.join(work, "generated/schema.lock"), path.join(direct, "schema.lock"), "schema-lock direct/build drift");
assertTreeFileList(path.join(work, "generated/excel"), path.join(direct, "excel"), "Excel template direct/build file-list drift");
assertWorkbookSemantics(path.join(direct, "excel"));
assertSameTree(path.join(work, "reader/generated"), path.join(direct, "rust"), "Rust codegen direct/build drift");
assertSameFile(path.join(work, "generated/config.sora"), path.join(direct, "config.sora"), "binary export direct/build drift");
assertSameTree(path.join(work, "generated/debug-json"), path.join(direct, "debug-json"), "diagnostic export direct/build drift");

run(sora, ["--serial", "build", "--project", "./project.toml", "--clean"]);
formatRust(path.join(work, "reader", "generated"));
assertWorkbookSemantics(path.join(work, "generated", "excel"));
assertMapsEqual(firstBuild, stableArtifactHashes(work), "second configured build drifted");

const generatedRust = walk(path.join(work, "reader", "generated")).filter((file) => file.endsWith(".rs"));
run("rustfmt", ["--edition", "2024", "--check", path.join(work, "reader", "main.rs"), ...generatedRust]);

run("cargo", ["run", "--manifest-path", "reader/Cargo.toml", "--locked", "--quiet", "--", path.join(work, "generated", "config.sora")]);

const actualFiles = stableArtifactFiles(work);
if (bless) {
  fs.rmSync(expected, { recursive: true, force: true });
  for (const relative of actualFiles) {
    const destination = path.join(expected, relative);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.copyFileSync(path.join(work, relative), destination);
  }
  const files = Object.fromEntries(actualFiles.map((relative) => [relative, sha256(path.join(work, relative))]));
  const manifest = {
    schema_revision: "starclock.sora-golden-output.v1",
    sora_cli_version: toolPolicy.version,
    crate_archive_sha256: toolPolicy.crate_sha256,
    output_digest: digestFileMap(files),
    files,
  };
  fs.writeFileSync(path.join(fixture, "expected-manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`Blessed Sora ${toolPolicy.version} golden (${actualFiles.length} files; ${manifest.output_digest}).`);
} else {
  const manifest = readJson(path.join(fixture, "expected-manifest.json"));
  assert(manifest.schema_revision === "starclock.sora-golden-output.v1", "unsupported Sora expected-output revision");
  assert(manifest.sora_cli_version === toolPolicy.version && manifest.crate_archive_sha256 === toolPolicy.crate_sha256, "expected outputs use a different Sora tool");
  const expectedFiles = stableArtifactFiles(expected);
  assert(JSON.stringify(actualFiles) === JSON.stringify(expectedFiles), "Sora golden artifact file list drifted");
  const actualMap = Object.fromEntries(actualFiles.map((relative) => [relative, sha256(path.join(work, relative))]));
  const expectedMap = Object.fromEntries(expectedFiles.map((relative) => [relative, sha256(path.join(expected, relative))]));
  assert(JSON.stringify(actualMap) === JSON.stringify(expectedMap), "Sora golden artifact bytes drifted");
  assert(JSON.stringify(actualMap) === JSON.stringify(manifest.files), "Sora expected manifest file hashes drifted");
  assert(digestFileMap(actualMap) === manifest.output_digest, "Sora expected manifest digest drifted");
  console.log(`Sora ${toolPolicy.version} capability golden verified (${actualFiles.length} files; ${manifest.output_digest}).`);
}

function resolveSora(policy) {
  if (process.env.STARCLOCK_SORA_BIN) return path.resolve(process.env.STARCLOCK_SORA_BIN);
  const local = path.join(root, policy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
  assert(fs.existsSync(local), `Sora ${policy.version} is not installed; run ${policy.install_command}`);
  return local;
}
function verifyUnsupportedAssumptions() {
  const bareProject = runFailure(sora, ["--serial", "build", "--project", "project.toml", "--clean"]);
  assert(`${bareProject.stdout}\n${bareProject.stderr}`.includes("failed to resolve project directory"), "bare project.toml clean-path limitation changed");

  const debugJson = runFailure(sora, ["--serial", "export", "--format", "debug-json", "--project", "./project.toml", "--data-root", "data", "--out", "unsupported/debug-json"]);
  assert(`${debugJson.stdout}\n${debugJson.stderr}`.includes("unknown export format `debug-json`"), "debug-json unexpectedly became a supported export spelling");

  const requiredFormat = spawnSync(sora, ["--serial", "gen", "--target", "rust", "--project", "./project.toml", "--out", "unsupported/format-required", "--format-code", "required"], { cwd: work, encoding: "utf8" });
  if (process.platform === "win32") {
    assert(requiredFormat.status !== 0 && `${requiredFormat.stdout}\n${requiredFormat.stderr}`.includes("formatter command was not found in PATH"), "Windows rustfmt executable-suffix limitation changed");
  } else {
    assert(requiredFormat.status === 0, `format-code required failed on ${process.platform}: ${requiredFormat.stderr}`);
  }

  const unsignedRoot = path.join(work, "unsupported", "unsigned");
  run(sora, ["--serial", "gen", "--target", "rust", "--project", "./project-unsigned.toml", "--out", "unsupported/unsigned/generated", "--format-code", "never"]);
  for (const name of ["Cargo.toml", "Cargo.lock"]) fs.copyFileSync(path.join(work, "reader", name), path.join(unsignedRoot, name));
  fs.writeFileSync(path.join(unsignedRoot, "main.rs"), "mod generated;\nfn main() {}\n");
  const unsignedCompile = runFailure("cargo", ["check", "--manifest-path", "unsupported/unsigned/Cargo.toml", "--locked"]);
  assert(`${unsignedCompile.stdout}\n${unsignedCompile.stderr}`.includes("u32: SoraDecode"), "unsigned Sora Rust decode limitation changed");
}
function verifyReaderDependencies(policy) {
  const metadata = JSON.parse(runCapture("cargo", ["metadata", "--manifest-path", "reader/Cargo.toml", "--locked", "--format-version", "1"]).stdout);
  const registry = metadata.packages.filter((entry) => entry.source?.startsWith("registry+")).map((entry) => ({ name: entry.name, version: entry.version, license: entry.license })).sort((left, right) => left.name.localeCompare(right.name));
  const reviewed = policy.packages.map((entry) => ({ name: entry.name, version: entry.version, license: entry.license })).sort((left, right) => left.name.localeCompare(right.name));
  assert(JSON.stringify(registry) === JSON.stringify(reviewed), "golden-reader resolved package/license inventory differs from policy");
  const lockPackages = parseRegistryLock(path.join(work, "reader", "Cargo.lock"));
  const reviewedLock = policy.packages.map(({ name, version, checksum }) => ({ name, version, checksum })).sort((left, right) => left.name.localeCompare(right.name));
  assert(JSON.stringify(lockPackages) === JSON.stringify(reviewedLock), "golden-reader locked package/checksum inventory differs from policy");
  const readerPackage = metadata.packages.find((entry) => entry.name === "starclock-sora-golden-reader");
  const direct = readerPackage.dependencies.map((entry) => entry.name).sort();
  assert(JSON.stringify(direct) === JSON.stringify([...policy.direct_packages].sort()), "golden-reader direct dependency inventory differs from policy");
}
function parseRegistryLock(file) {
  return fs.readFileSync(file, "utf8").split("[[package]]").slice(1).map((block) => ({
    name: block.match(/^name = "([^"]+)"/m)?.[1],
    version: block.match(/^version = "([^"]+)"/m)?.[1],
    source: block.match(/^source = "([^"]+)"/m)?.[1],
    checksum: block.match(/^checksum = "([^"]+)"/m)?.[1],
  })).filter((entry) => entry.source?.startsWith("registry+")).map(({ name, version, checksum }) => ({ name, version, checksum })).sort((left, right) => left.name.localeCompare(right.name));
}
function run(command, args) {
  const result = spawnSync(command, args, { cwd: work, stdio: "inherit" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${relativeCommand(command)} ${args.join(" ")} exited with ${result.status}`);
}
function runCapture(command, args) {
  const result = spawnSync(command, args, { cwd: fs.existsSync(work) ? work : root, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${relativeCommand(command)} ${args.join(" ")} exited with ${result.status}: ${result.stderr}`);
  return result;
}
function runFailure(command, args) {
  const result = spawnSync(command, args, { cwd: work, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status !== 0, `${relativeCommand(command)} ${args.join(" ")} unexpectedly succeeded`);
  return result;
}
function artifactFiles(base, roots) {
  return roots.flatMap((relative) => walk(path.join(base, relative)).map((file) => path.relative(base, file).replaceAll("\\", "/"))).sort();
}
function formatRust(directory) {
  const files = walk(directory).filter((file) => file.endsWith(".rs"));
  run("rustfmt", ["--edition", "2024", ...files]);
}
function artifactHashes(base, roots) { return new Map(artifactFiles(base, roots).map((relative) => [relative, sha256(path.join(base, relative))])); }
function stableArtifactFiles(base) { return artifactFiles(base, ["generated", "reader/generated"]).filter((relative) => !relative.startsWith("generated/excel/")); }
function stableArtifactHashes(base) { return new Map(stableArtifactFiles(base).map((relative) => [relative, sha256(path.join(base, relative))])); }
function walk(directory) {
  assert(fs.existsSync(directory), `missing artifact directory ${directory}`);
  return fs.readdirSync(directory, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name)).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    return entry.isDirectory() ? walk(target) : entry.isFile() ? [target] : [];
  });
}
function assertSameFile(left, right, message) { assert(sha256(left) === sha256(right), message); }
function assertSameTree(left, right, message) {
  const leftMap = new Map(walk(left).map((file) => [path.relative(left, file).replaceAll("\\", "/"), sha256(file)]));
  const rightMap = new Map(walk(right).map((file) => [path.relative(right, file).replaceAll("\\", "/"), sha256(file)]));
  assertMapsEqual(leftMap, rightMap, message);
}
function assertTreeFileList(left, right, message) {
  const leftFiles = walk(left).map((file) => path.relative(left, file).replaceAll("\\", "/"));
  const rightFiles = walk(right).map((file) => path.relative(right, file).replaceAll("\\", "/"));
  assert(JSON.stringify(leftFiles) === JSON.stringify(rightFiles), message);
}
function assertWorkbookSemantics(directory) {
  const result = runCapture(sora, ["--serial", "excel-sync", "--project", "./project.toml", "--data-root", path.relative(work, directory)]);
  const report = `${result.stdout}\n${result.stderr}`;
  assert(report.includes("unchanged sheet: ExcelProbe"), `ExcelProbe template is not semantically synchronized: ${report}`);
  assert(!report.includes("add columns:") && !report.includes("remove columns:") && !report.includes("update columns:"), `Excel template has schema drift: ${report}`);
}
function assertMapsEqual(left, right, message) { assert(JSON.stringify([...left]) === JSON.stringify([...right]), message); }
function digestFileMap(files) {
  const canonical = Object.entries(files).sort(([left], [right]) => left.localeCompare(right)).map(([name, hash]) => `${name}\0${hash}\n`).join("");
  return crypto.createHash("sha256").update(canonical, "utf8").digest("hex");
}
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function relativeCommand(command) { return path.isAbsolute(command) ? path.relative(root, command).replaceAll("\\", "/") : command; }
function assert(condition, message) { if (!condition) throw new Error(message); }
