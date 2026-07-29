#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const arguments_ = process.argv.slice(2);
const withSourceCache = arguments_.includes("--with-source-cache");
assert(arguments_.every((argument) =>
  argument === "--with-source-cache" || !argument.startsWith("--")),
"usage: run-acceptance.mjs [root] [--with-source-cache]");
const root = path.resolve(arguments_.find((argument) =>
  !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const python = process.env.STARCLOCK_PYTHON
  ?? "/Users/mikai/.cache/codex-runtimes/codex-primary-runtime/" +
    "dependencies/python/bin/python3";
const imports = [
  "flow",
  "squad-boundary",
  "economy",
  "position-empowerment",
  "bonds",
  "stars",
  "build-equipment",
  "investment-systems",
  "rank-progression",
  "blessing-closure",
  "curio-hex-closure",
  "events",
  "services",
  "encounters",
];
const sourceCommands = [
  ["node", "tools/currency-wars-reference/verify-foundation.mjs"],
  ["node", "tools/currency-wars-reference/inventory.mjs", "--check"],
  ["node", "tools/currency-wars-reference/verify-inventory.mjs"],
  ["node", "tools/currency-wars-reference/manifest.mjs", "--check"],
  ["node", "tools/currency-wars-reference/verify-manifest.mjs"],
  ["node", "tools/currency-wars-reference/contracts.mjs", "--check"],
  ["node", "tools/currency-wars-reference/verify-contracts.mjs"],
  ...imports.flatMap((name) => [
    ["node", `tools/currency-wars-reference/import-${name}.mjs`, "--check"],
    ["node", `tools/currency-wars-reference/verify-${name}.mjs`],
  ]),
  ["node", "tools/currency-wars-reference/generate-pack.mjs", "--check"],
  ["node", "tools/currency-wars-reference/verify-pack.mjs"],
  ["node", "tools/currency-wars-reference/reconcile-goals.mjs", "--check"],
];
const artifactCommands = [
  [
    "node",
    "tools/currency-wars-reference/audit-ownership.mjs",
    "--check",
    "--batch",
    "G12-P4-B3",
    "--output",
    "evidence/currency-wars-reference-v1/p4b3-ownership-audit.json",
  ],
  [
    "node",
    "tools/currency-wars-reference/execute-semantic-fixtures.mjs",
    "--check",
  ],
  [
    python,
    "tools/currency-wars-reference/verify-workbooks.py",
    "--root",
    root,
    "--directory",
    path.join(root, "config/currency-wars/data"),
  ],
  [
    "node",
    "tools/currency-wars-reference/verify-sora-schema.mjs",
    "--through",
    "P3-B4",
  ],
  ["node", "tools/currency-wars-reference/verify-sora-generated.mjs"],
  [
    "node",
    "tools/currency-wars-reference/verify-sora-reader.mjs",
    "config/currency-wars-generated/config.sora",
  ],
  [
    "node",
    "tools/currency-wars-reference/verify-release-acceptance.mjs",
    ...(withSourceCache
      ? [
        "--source-cache-root",
        "/Users/mikai/CLionProjects/starclock/.cache/content-reference",
      ]
      : []),
  ],
  [
    "node",
    "tools/currency-wars-reference/verify-release.mjs",
  ],
  ["node", "tools/dependency-policy/verify.mjs"],
  ["node", "tools/workspace/verify-dependencies.mjs"],
  ["node", "tools/repository-check/run.mjs"],
  ["git", "diff", "--check"],
];
const commands = [
  ...(withSourceCache ? sourceCommands : []),
  ...artifactCommands,
];
for (const command of commands) run(command);
console.log(
  `Currency Wars ${withSourceCache ? "source-cache and " : ""}` +
  `artifact acceptance passed (${commands.length} commands).`,
);

function run(command) {
  console.log(`\n==> ${command.join(" ")}`);
  const result = spawnSync(command[0], command.slice(1), {
    cwd: root,
    env: {
      ...process.env,
      PYTHONDONTWRITEBYTECODE: "1",
      STARCLOCK_PYTHON: python,
    },
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  assert(result.status === 0,
    `${command.join(" ")} exited with ${result.status}`);
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
