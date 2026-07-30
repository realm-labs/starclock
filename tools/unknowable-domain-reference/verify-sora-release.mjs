import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const root = path.resolve(process.argv[2] ?? ".");
const policy = json("policy/sora-toolchain.json");
const sora = locateSora();
const projectRelative = path.join("config", "unknowable-domain", "project.toml");
const committedGenerated = path.join(
  root,
  "config",
  "unknowable-domain-generated",
);
const committedData = path.join(root, "config", "unknowable-domain", "data");
const loader = path.join(
  root,
  "tools",
  "unknowable-domain-reference",
  "bundle-loader",
);
const ephemeralGenerated = path.join(loader, "src", "generated");
const python = process.env.STARCLOCK_PYTHON ?? "python3";
const pythonEnvironment = {
  ...process.env,
  PYTHONDONTWRITEBYTECODE: "1",
};
const temporary = fs.mkdtempSync(
  path.join(os.tmpdir(), "starclock-unknowable-domain-release-"),
);
const workbooks = [
  "UnknowableDomain.xlsx",
  "UnknowableDomainBindings.xlsx",
  "UnknowableDomainReview.xlsx",
];

try {
  assert(
    policy.version === "0.3.0" && fs.existsSync(sora),
    "pinned Sora 0.3.0 is unavailable",
  );
  assert(
    !fs.existsSync(ephemeralGenerated),
    "ephemeral generated-reader directory already exists",
  );
  run(python, [
    "-c",
    "import openpyxl; assert openpyxl.__version__ == '3.1.5'",
  ], pythonEnvironment);
  run(sora, [
    "--serial",
    "check",
    "--project",
    path.join(root, projectRelative),
  ]);

  const builds = ["a", "b"].map((label) => {
    const buildRoot = path.join(temporary, `build-${label}`);
    const projectRoot = path.join(buildRoot, "config", "unknowable-domain");
    fs.mkdirSync(projectRoot, { recursive: true });
    fs.copyFileSync(
      path.join(root, projectRelative),
      path.join(projectRoot, "project.toml"),
    );
    fs.cpSync(
      path.join(root, "config", "unknowable-domain", "schema"),
      path.join(projectRoot, "schema"),
      { recursive: true },
    );
    const data = path.join(projectRoot, "data");
    run(python, [
      "tools/unknowable-domain-reference/author_workbooks.py",
      "--root",
      root,
      "--output",
      data,
    ], pythonEnvironment);
    run(sora, [
      "--serial",
      "build",
      "--project",
      path.join(projectRoot, "project.toml"),
    ]);
    const generated = path.join(
      buildRoot,
      "config",
      "unknowable-domain-generated",
    );
    formatRust(path.join(generated, "rust"));
    return {
      data,
      generated,
    };
  });

  for (const workbook of workbooks) {
    const first = path.join(builds[0].data, workbook);
    const second = path.join(builds[1].data, workbook);
    assertSame(first, second, `${workbook} double generation`);
    assertSame(first, path.join(committedData, workbook), `${workbook} drift`);
  }
  assertSameGenerated(builds[0].generated, builds[1].generated);
  assertSameGenerated(builds[0].generated, committedGenerated, true);

  const debugRoot = path.join(builds[0].generated, "debug-json");
  const tables =
    JSON.parse(
      fs.readFileSync(
        path.join(builds[0].generated, "schema.lock"),
        "utf8",
      ),
    ).schema.tables;
  let rowCount = 0;
  let emptyCount = 0;
  for (const table of tables) {
    const payload = JSON.parse(
      fs.readFileSync(path.join(debugRoot, `${table.name}.json`), "utf8"),
    );
    const rows = payload.table.rows.length;
    rowCount += rows;
    if (rows === 0) {
      emptyCount += 1;
    }
  }
  assert(
    tables.length === 65 && rowCount === 17_149 && emptyCount === 4,
    "Sora table/row/empty-table denominator differs",
  );

  fs.cpSync(
    path.join(builds[0].generated, "rust"),
    ephemeralGenerated,
    { recursive: true },
  );
  run(
    "cargo",
    [
      "run",
      "--manifest-path",
      path.join(loader, "Cargo.toml"),
      "--locked",
      "--quiet",
      "--",
      path.join(builds[0].generated, "config.sora"),
      String(tables.length),
      String(rowCount),
      String(emptyCount),
    ],
    {
      ...process.env,
      CARGO_TARGET_DIR: path.join(
        root,
        ".cache",
        "unknowable-domain-bundle-loader-target",
      ),
    },
  );
  verifyVisualReview(tables, rowCount, emptyCount);
  console.log(
    `Unknowable Domain Sora release verified (${tables.length} tables, ` +
      `${rowCount} rows, ${emptyCount} verified-empty tables; ` +
      `bundle ${sha256(path.join(committedGenerated, "config.sora"))}; ` +
      "byte-identical workbooks/build/export; every generated reader loaded).",
  );
} finally {
  fs.rmSync(ephemeralGenerated, { recursive: true, force: true });
  fs.rmSync(temporary, { recursive: true, force: true });
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
  const executable = process.platform === "win32" ? "sora.exe" : "sora";
  const local = path.join(root, policy.install_root, "bin", executable);
  if (fs.existsSync(local)) {
    return local;
  }
  const worktrees = spawnSync(
    "git",
    ["worktree", "list", "--porcelain"],
    { cwd: root, encoding: "utf8" },
  );
  if (worktrees.status === 0) {
    for (const line of worktrees.stdout.split(/\r?\n/u)) {
      if (line.startsWith("worktree ")) {
        const candidate = path.join(
          line.slice("worktree ".length),
          policy.install_root,
          "bin",
          executable,
        );
        if (fs.existsSync(candidate)) {
          return candidate;
        }
      }
    }
  }
  return local;
}

function formatRust(directory) {
  const files = fs
    .readdirSync(directory)
    .filter((name) => name.endsWith(".rs"))
    .map((name) => path.join(directory, name));
  run("rustfmt", ["--edition", "2024", ...files]);
}

function verifyVisualReview(tables, rowCount, emptyCount) {
  const review = json(
    "evidence/unknowable-domain-reference-v1/visual-review.json",
  );
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
    review.contact_sheet_sha256.length === 9 &&
      Object.values(review.checks).every((value) => value === true) &&
      Array.isArray(review.defects) &&
      review.defects.length === 0,
    "visual-review checks are incomplete",
  );
}

function debugTreeDigest(files) {
  const digest = crypto.createHash("sha256");
  for (const file of files.toSorted()) {
    digest.update(file);
    digest.update("\0");
    digest.update(
      fs.readFileSync(
        path.join(committedGenerated, "debug-json", file),
      ),
    );
    digest.update("\0");
  }
  return digest.digest("hex");
}

function assertSame(first, second, label) {
  assert(
    fs.readFileSync(first).equals(fs.readFileSync(second)),
    `${label} differs`,
  );
}

function assertSameGenerated(first, second, committed = false) {
  const required = [
    "schema.lock",
    "config.sora",
    ...workbooks.map((name) => path.join("templates", name)),
    ...listFiles(path.join(first, "debug-json")).map((name) =>
      path.join("debug-json", name)
    ),
    ...listFiles(path.join(first, "rust"))
      .filter((name) => name !== "mod.rs")
      .map((name) => path.join("rust", name)),
  ];
  const expected = required.toSorted();
  const actual = listFiles(second)
    .filter((name) => name !== path.join("rust", "mod.rs"))
    .toSorted();
  assert(
    JSON.stringify(expected) === JSON.stringify(actual),
    `${committed ? "committed" : "double-build"} generated file set differs`,
  );
  for (const relative of expected) {
    if (relative.startsWith(`templates${path.sep}`)) {
      assert(
        fs.statSync(path.join(first, relative)).size > 1000 &&
          fs.statSync(path.join(second, relative)).size > 1000,
        `${committed ? "committed" : "double-build"}/${relative} is missing`,
      );
      continue;
    }
    assertSame(
      path.join(first, relative),
      path.join(second, relative),
      `${committed ? "committed" : "double-build"}/${relative}`,
    );
  }
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

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}
