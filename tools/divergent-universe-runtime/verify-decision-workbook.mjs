// Rebuild current artifacts without overwriting designer-authored input.
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const project = path.join(root, "config/divergent-universe-decisions.toml");
const generated = path.join(root, "config/divergent-universe-decisions-generated");
const data = path.join(root, "config/divergent-universe-decisions/data");
const sora = path.join(root, ".cache/tools/sora-cli-0.6.1/bin", process.platform === "win32" ? "sora.exe" : "sora");
const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "starclock-du-decisions-"));
const python = process.env.STARCLOCK_PYTHON ?? "python";
execFileSync(process.execPath, ["tools/divergent-universe-runtime/generate-decision-schema.mjs", "--check"], { cwd: root, stdio: "inherit" });
const run = (args) => execFileSync(sora, ["--serial", ...args], { cwd: root, stdio: "inherit" });
const equal = (left, right) => {
  if (!fs.readFileSync(left).equals(fs.readFileSync(right))) throw Error(`generated drift: ${left}`);
};

run(["schema-lock", "--project", project, "--out", path.join(scratch, "schema.lock")]);
equal(path.join(generated, "schema.lock"), path.join(scratch, "schema.lock"));
run(["gen", "--target", "rust", "--project", project, "--out", path.join(scratch, "reader"), "--format-code", "never"]);
const names = (folder) => fs.readdirSync(folder).sort();
if (JSON.stringify(names(path.join(generated, "reader"))) !== JSON.stringify(names(path.join(scratch, "reader")))) throw Error("reader file set drift");
for (const file of names(path.join(generated, "reader"))) equal(path.join(generated, "reader", file), path.join(scratch, "reader", file));
for (const [format, directory] of [["binary", "config.sora"], ["json-debug", "debug-json"]]) {
  run(["export", "--format", format, "--project", project, "--data-root", data, "--out", path.join(scratch, directory)]);
  if (format === "binary") equal(path.join(generated, directory), path.join(scratch, directory));
  else {
    if (JSON.stringify(names(path.join(generated, directory))) !== JSON.stringify(names(path.join(scratch, directory)))) throw Error("debug export file set drift");
    for (const file of names(path.join(generated, directory))) equal(path.join(generated, directory, file), path.join(scratch, directory, file));
  }
}
const freshData = path.join(scratch, "reauthored");
execFileSync(python, ["tools/divergent-universe-runtime/author_decision_workbook.py", "--root", root,
  "--output", path.join(freshData, "DivergentUniverseDecisions.xlsx")], { cwd: root, stdio: "inherit" });
run(["export", "--format", "binary", "--project", project, "--data-root", freshData, "--out", path.join(scratch, "reauthored.sora")]);
equal(path.join(generated, "config.sora"), path.join(scratch, "reauthored.sora"));
console.log(`Decision workbook, schema, readers and exports verified; fresh artifacts retained at ${scratch}. No gameplay execution claim.`);
