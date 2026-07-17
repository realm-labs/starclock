import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const policy = JSON.parse(fs.readFileSync(path.join(root, "policy/sora-toolchain.json"), "utf8"));
const force = process.argv.slice(2).includes("--force");
assert(process.argv.slice(2).every((argument) => argument === "--force"), "usage: node tools/sora/install.mjs [--force]");
const installRoot = path.resolve(root, policy.install_root);
assert(installRoot.startsWith(path.join(root, ".cache", "tools") + path.sep), "Sora install root escaped the repository tool cache");
const binary = path.join(installRoot, "bin", process.platform === "win32" ? "sora.exe" : "sora");

const downloads = path.join(root, ".cache", "tools", "downloads");
fs.mkdirSync(downloads, { recursive: true });
const archive = path.join(downloads, `${policy.package}-${policy.version}.crate`);
if (!fs.existsSync(archive) || sha256(archive) !== policy.crate_sha256) {
  const response = await fetch(policy.crate_url);
  assert(response.ok, `failed to download ${policy.crate_url}: HTTP ${response.status}`);
  fs.writeFileSync(archive, Buffer.from(await response.arrayBuffer()));
}
assert(sha256(archive) === policy.crate_sha256, `Sora archive checksum mismatch: ${sha256(archive)}`);

if (!force && fs.existsSync(binary) && runCapture(binary, ["--version"]) === `sora ${policy.version}`) {
  console.log(`Checksum-bound Sora ${policy.version} already installed at ${relative(binary)}.`);
  process.exit(0);
}

run("cargo", ["install", policy.package, "--version", `=${policy.version}`, "--locked", "--root", installRoot, "--force"]);
assert(fs.existsSync(binary), `cargo install did not create ${relative(binary)}`);
assert(runCapture(binary, ["--version"]) === `sora ${policy.version}`, "installed Sora version differs from policy");
console.log(`Installed checksum-bound Sora ${policy.version} at ${relative(binary)}.`);

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} exited with ${result.status}`);
}
function runCapture(command, args) {
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8" });
  if (result.error) throw result.error;
  assert(result.status === 0, `${command} exited with ${result.status}: ${result.stderr}`);
  return result.stdout.trim();
}
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function relative(file) { return path.relative(root, file).replaceAll("\\", "/"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
