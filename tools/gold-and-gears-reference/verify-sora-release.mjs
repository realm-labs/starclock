import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const root = path.resolve(process.argv[2] ?? ".");
const policy = json("policy/sora-toolchain.json");
const sora = path.join(
  root,
  policy.install_root,
  "bin",
  process.platform === "win32" ? "sora.exe" : "sora",
);
const project = path.join(root, "config", "gold-and-gears", "project.toml");
const committed = path.join(root, "config", "gold-and-gears-generated");
const committedData = path.join(root, "config", "gold-and-gears", "data");
const python = process.env.STARCLOCK_PYTHON ?? "python3";
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "starclock-gold-gears-release-"));
const workbooks = [
  "GoldAndGears.xlsx",
  "GoldAndGearsProgression.xlsx",
  "GoldAndGearsContent.xlsx",
  "GoldAndGearsEvidence.xlsx",
];

try {
  assert(policy.version === "0.3.0" && fs.existsSync(sora), "pinned Sora 0.3.0 is unavailable");
  run(python, ["-c", "import openpyxl; assert openpyxl.__version__ == '3.1.5'"]);
  const firstData = path.join(temporary, "workbooks-a");
  const secondData = path.join(temporary, "workbooks-b");
  for (const output of [firstData, secondData]) {
    run(python, [
      "tools/gold-and-gears-reference/author_workbooks.py",
      "--root",
      root,
      "--output",
      output,
    ]);
  }
  for (const workbook of workbooks) {
    assertSame(path.join(firstData, workbook), path.join(secondData, workbook), `${workbook} double generation`);
    assertSame(path.join(firstData, workbook), path.join(committedData, workbook), `${workbook} committed drift`);
  }

  run(sora, ["--serial", "check", "--project", project]);
  const firstBundle = path.join(temporary, "first.sora");
  const secondBundle = path.join(temporary, "second.sora");
  const firstDebug = path.join(temporary, "debug-a");
  const secondDebug = path.join(temporary, "debug-b");
  run(sora, [
    "--serial", "export", "--format", "binary", "--project", project,
    "--data-root", firstData, "--out", firstBundle,
  ]);
  run(sora, [
    "--serial", "export", "--format", "binary", "--project", project,
    "--data-root", secondData, "--out", secondBundle,
  ]);
  run(sora, [
    "--serial", "export", "--format", "json-debug", "--project", project,
    "--data-root", firstData, "--out", firstDebug,
  ]);
  run(sora, [
    "--serial", "export", "--format", "json-debug", "--project", project,
    "--data-root", secondData, "--out", secondDebug,
  ]);
  assertSame(firstBundle, secondBundle, "Sora binary double export");
  assertSame(firstBundle, path.join(committed, "config.sora"), "committed Sora bundle drift");
  assertSameTree(firstDebug, secondDebug, "Sora debug double export");
  assertSameTree(firstDebug, path.join(committed, "debug-json"), "committed Sora debug drift");

  const tables = json("config/gold-and-gears-generated/schema.lock").schema.tables;
  let rowCount = 0;
  for (const table of tables) {
    const payload = JSON.parse(fs.readFileSync(path.join(firstDebug, `${table.name}.json`), "utf8"));
    rowCount += payload.table.rows.length;
  }
  assert(tables.length === 52 && rowCount === 29140, "Sora table/row denominator differs");
  run(
    "cargo",
    [
      "run",
      "--manifest-path",
      "tools/gold-and-gears-reference/bundle-loader/Cargo.toml",
      "--locked",
      "--quiet",
      "--",
      firstBundle,
      String(tables.length),
      String(rowCount),
    ],
    {
      ...process.env,
      CARGO_TARGET_DIR: path.join(root, ".cache", "gold-and-gears-bundle-loader-target"),
    },
  );
  console.log(
    `Gold and Gears Sora release verified (${tables.length} tables, ${rowCount} rows; ` +
    "byte-identical workbooks/bundle/debug export; every generated reader loaded).",
  );
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

function run(command, arguments_, env = process.env) {
  const result = spawnSync(command, arguments_, { cwd: root, encoding: "utf8", env });
  if (result.status !== 0) {
    throw new Error(`${command} ${arguments_.join(" ")} failed\n${result.stdout}\n${result.stderr}`);
  }
}

function assertSame(first, second, label) {
  assert(fs.readFileSync(first).equals(fs.readFileSync(second)), `${label} differs`);
}

function assertSameTree(first, second, label) {
  const firstFiles = fs.readdirSync(first).filter((name) => name.endsWith(".json")).sort();
  const secondFiles = fs.readdirSync(second).filter((name) => name.endsWith(".json")).sort();
  assert(JSON.stringify(firstFiles) === JSON.stringify(secondFiles), `${label} file set differs`);
  for (const file of firstFiles) assertSame(path.join(first, file), path.join(second, file), `${label}/${file}`);
}

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
