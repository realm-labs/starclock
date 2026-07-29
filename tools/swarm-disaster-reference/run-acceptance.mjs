#!/usr/bin/env node

import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const arguments_ = process.argv.slice(2);
const withSourceCache = arguments_.includes("--with-source-cache");
assert(
  arguments_.every((argument) =>
    argument === "--with-source-cache" || !argument.startsWith("--")),
  "usage: run-acceptance.mjs [root] [--with-source-cache]",
);
const root = path.resolve(
  arguments_.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const python = process.env.STARCLOCK_PYTHON ?? "python3";
const sourceCommands = [
  ["node", "tools/swarm-disaster-reference/verify-foundation.mjs"],
  ["node", "tools/swarm-disaster-reference/inventory.mjs", "--check"],
  ["node", "tools/swarm-disaster-reference/verify-inventory.mjs"],
  ["node", "tools/swarm-disaster-reference/manifest.mjs", "--check"],
  ["node", "tools/swarm-disaster-reference/verify-manifest.mjs"],
  ["node", "tools/swarm-disaster-reference/verify-contracts.mjs"],
  ...[
    "topology",
    "domains",
    "countdown",
    "audience-dice",
    "dice-faces",
    "communing",
    "communing-trail",
    "pathstrider",
    "paths",
    "blessings",
    "curios",
    "occurrences",
    "services",
    "encounters",
  ].flatMap((name) => [
    ["node", `tools/swarm-disaster-reference/import-${name}.mjs`, "--check"],
    ["node", `tools/swarm-disaster-reference/verify-${name}.mjs`],
  ]),
  ["node", "tools/swarm-disaster-reference/finalize-pack.mjs", "--check"],
  ["node", "tools/swarm-disaster-reference/verify-pack.mjs"],
];
const artifactCommands = [
  ["node", "tools/swarm-disaster-reference/audit-release.mjs"],
  ["node", "tools/swarm-disaster-reference/verify-semantic-fixtures.mjs"],
  ["node", "tools/swarm-disaster-reference/verify-sora-schema.mjs", root],
  [
    python,
    "tools/swarm-disaster-reference/verify_workbooks.py",
    "--root",
    root,
  ],
  ["node", "tools/swarm-disaster-reference/verify-visual-review.mjs", root],
  ["node", "tools/swarm-disaster-reference/verify-sora-release.mjs", root],
  ["node", "tools/swarm-disaster-reference/audit-integration.mjs", root],
  ["node", "tools/dependency-policy/verify.mjs"],
  ["node", "tools/workspace/verify-dependencies.mjs"],
  ["git", "diff", "--check"],
];
const commands = [
  ...(withSourceCache ? sourceCommands : []),
  ...artifactCommands,
];

for (const command of commands) run(command);
console.log(
  `Swarm Disaster ${withSourceCache ? "source-cache and " : ""}` +
  `artifact acceptance passed (${commands.length} commands).`,
);

function run(command) {
  console.log(`\n==> ${command.join(" ")}`);
  const result = spawnSync(command[0], command.slice(1), {
    cwd: root,
    env: {
      ...process.env,
      PYTHONDONTWRITEBYTECODE: "1",
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
