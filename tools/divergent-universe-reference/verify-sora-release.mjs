import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const root = path.resolve(process.argv[2] ?? ".");
const projectRelative = path.join(
  "config",
  "divergent-universe",
  "project.toml",
);
const generatedRelative = path.join(
  "config",
  "divergent-universe-generated",
);
const committedGenerated = path.join(root, generatedRelative);
const committedData = path.join(
  root,
  "config",
  "divergent-universe",
  "data",
);
const loader = path.join(
  root,
  "tools",
  "divergent-universe-reference",
  "bundle-loader",
);
const ephemeralGenerated = path.join(loader, "src", "generated");
const python = process.env.STARCLOCK_PYTHON ?? "python3";
const sora = locateSora();
const temporary = fs.mkdtempSync(
  path.join(os.tmpdir(), "starclock-divergent-universe-release-"),
);
const workbooks = [
  "DivergentUniverse.xlsx",
  "DivergentUniverseBindings.xlsx",
  "DivergentUniverseReview.xlsx",
];

try {
  assert(
    !fs.existsSync(ephemeralGenerated),
    "ephemeral generated-reader directory already exists",
  );
  run(python, [
    "-c",
    "import openpyxl; assert openpyxl.__version__ == '3.1.5'",
  ]);
  run(sora, [
    "--serial",
    "check",
    "--project",
    path.join(root, projectRelative),
  ]);
  const builds = ["a", "b"].map((label) => build(label));
  for (const workbook of workbooks) {
    const first = path.join(builds[0].data, workbook);
    const second = path.join(builds[1].data, workbook);
    assertSame(first, second, `${workbook} double generation`);
    assertSame(first, path.join(committedData, workbook), `${workbook} drift`);
  }
  assertSameGenerated(builds[0].generated, builds[1].generated);
  assertSameGenerated(builds[0].generated, committedGenerated, true);

  const schema = jsonAt(path.join(builds[0].generated, "schema.lock")).schema;
  let rowCount = 0;
  let emptyCount = 0;
  for (const table of schema.tables) {
    const payload = jsonAt(path.join(
      builds[0].generated,
      "debug-json",
      `${table.name}.json`,
    ));
    const count = payload.table.rows.length;
    rowCount += count;
    if (count === 0) emptyCount += 1;
  }
  assert(
    schema.tables.length === 80 &&
      rowCount === 26_985 &&
      emptyCount === 3,
    "Sora table/row/empty-table denominator differs",
  );

  fs.cpSync(
    path.join(builds[0].generated, "reader"),
    ephemeralGenerated,
    { recursive: true },
  );
  run("cargo", [
    "run",
    "--manifest-path",
    path.join(loader, "Cargo.toml"),
    "--locked",
    "--quiet",
    "--",
    path.join(builds[0].generated, "config.sora"),
    String(schema.tables.length),
    String(rowCount),
    String(emptyCount),
  ], {
    ...process.env,
    CARGO_TARGET_DIR: path.join(
      root,
      ".cache",
      "divergent-universe-bundle-loader-target",
    ),
  });
  verifyVisualReview(schema.tables, rowCount, emptyCount);
  console.log(
    `Divergent Universe Sora release verified (${schema.tables.length} ` +
      `tables, ${rowCount} rows, ${emptyCount} verified-empty tables; ` +
      `bundle ${sha256(path.join(committedGenerated, "config.sora"))}; ` +
      "byte-identical workbooks/build/export; every reader loaded).",
  );
} finally {
  fs.rmSync(ephemeralGenerated, { recursive: true, force: true });
  fs.rmSync(temporary, { recursive: true, force: true });
}

function build(label) {
  const buildRoot = path.join(temporary, `build-${label}`);
  const projectRoot = path.join(
    buildRoot,
    "config",
    "divergent-universe",
  );
  fs.mkdirSync(projectRoot, { recursive: true });
  fs.copyFileSync(
    path.join(root, projectRelative),
    path.join(projectRoot, "project.toml"),
  );
  fs.cpSync(
    path.join(root, "config", "divergent-universe", "schema"),
    path.join(projectRoot, "schema"),
    { recursive: true },
  );
  const data = path.join(projectRoot, "data");
  run(python, [
    "tools/divergent-universe-reference/author_workbooks.py",
    root,
    "--output",
    data,
  ], {
    ...process.env,
    PYTHONDONTWRITEBYTECODE: "1",
  });
  run(sora, [
    "--serial",
    "build",
    "--project",
    path.join(projectRoot, "project.toml"),
  ]);
  return {
    data,
    generated: path.join(
      buildRoot,
      "config",
      "divergent-universe-generated",
    ),
  };
}

function verifyVisualReview(tables, rowCount, emptyCount) {
  const review = jsonAt(path.join(
    root,
    "evidence",
    "divergent-universe-reference-v1",
    "visual-review.json",
  ));
  const expectedWorkbooks = workbooks.map((file) => ({
    file,
    sheets: tables
      .filter((table) => table.source.file === file)
      .map((table) => table.source.sheet),
  }));
  assert(
    review.sheet_count === tables.length &&
      JSON.stringify(review.workbooks) === JSON.stringify(expectedWorkbooks),
    "visual-review workbook/sheet denominator differs",
  );
  for (const workbook of workbooks) {
    assert(
      review.workbook_sha256[workbook] ===
        sha256(path.join(committedData, workbook)),
      `${workbook} visual-review digest differs`,
    );
  }
  const bundle = path.join(committedGenerated, "config.sora");
  assert(
    review.sora_bundle.bytes === fs.statSync(bundle).size &&
      review.sora_bundle.sha256 === sha256(bundle) &&
      review.sora_bundle.tables === tables.length &&
      review.sora_bundle.rows === rowCount &&
      review.sora_bundle.verified_empty_tables === emptyCount,
    "visual-review Sora bundle identity differs",
  );
  const debugFiles = listFiles(
    path.join(committedGenerated, "debug-json"),
  ).filter((name) => name.endsWith(".json"));
  assert(
    review.debug_export.files === debugFiles.length &&
      review.debug_export.sha256 === debugTreeDigest(debugFiles),
    "visual-review debug-export identity differs",
  );
  assert(
    review.contact_sheet_sha256.length === 10 &&
      Object.values(review.checks).every((value) => value === true) &&
      Array.isArray(review.defects) &&
      review.defects.length === 0,
    "visual-review checks are incomplete",
  );
}

function assertSameGenerated(first, second, committed = false) {
  const templates = workbooks.map((name) => path.join("templates", name));
  const required = [
    "schema.lock",
    "config.sora",
    ...templates,
    ...listFiles(path.join(first, "debug-json")).map((name) =>
      path.join("debug-json", name)
    ),
    ...listFiles(path.join(first, "reader")).map((name) =>
      path.join("reader", name)
    ),
  ].toSorted();
  const actual = listFiles(second).toSorted();
  assert(
    JSON.stringify(required) === JSON.stringify(actual),
    `${committed ? "committed" : "double-build"} generated file set differs`,
  );
  for (const relative of required) {
    if (relative.startsWith(`templates${path.sep}`)) {
      assert(
        fs.statSync(path.join(first, relative)).size > 1_000 &&
          fs.statSync(path.join(second, relative)).size > 1_000,
        `${relative} template is missing`,
      );
    } else {
      assertSame(
        path.join(first, relative),
        path.join(second, relative),
        `${committed ? "committed" : "double-build"}/${relative}`,
      );
    }
  }
}

function debugTreeDigest(files) {
  const digest = crypto.createHash("sha256");
  for (const file of files.toSorted()) {
    digest.update(file);
    digest.update("\0");
    digest.update(fs.readFileSync(path.join(
      committedGenerated,
      "debug-json",
      file,
    )));
    digest.update("\0");
  }
  return digest.digest("hex");
}

function listFiles(directory, prefix = "") {
  const result = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const relative = path.join(prefix, entry.name);
    if (entry.isDirectory()) {
      result.push(...listFiles(path.join(directory, entry.name), relative));
    } else {
      result.push(relative);
    }
  }
  return result.toSorted();
}

function run(command, arguments_, env = process.env) {
  const result = spawnSync(command, arguments_, {
    cwd: root,
    encoding: "utf8",
    env,
  });
  if (result.status !== 0) {
    throw new Error(
      `${command} ${arguments_.join(" ")} failed\n` +
        `${result.stdout}\n${result.stderr}`,
    );
  }
}

function locateSora() {
  const policy = jsonAt(path.join(root, "policy/sora-toolchain.json"));
  const candidates = [
    path.join(root, policy.install_root, "bin", "sora"),
    path.join(
      "/Users/mikai/CLionProjects/starclock",
      policy.install_root,
      "bin",
      "sora",
    ),
  ];
  const result = candidates.find((candidate) => fs.existsSync(candidate));
  if (!result) throw new Error("pinned Sora 0.3.0 is unavailable");
  return result;
}

function assertSame(first, second, label) {
  assert(
    fs.readFileSync(first).equals(fs.readFileSync(second)),
    `${label} differs`,
  );
}

function jsonAt(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
