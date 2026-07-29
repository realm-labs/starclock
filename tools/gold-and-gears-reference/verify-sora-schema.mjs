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
const schema = path.join(root, "config", "gold-and-gears", "schema", "core.toml");
const normalizedRoot = path.join(root, "content-reference", "gold-and-gears-v1");
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "starclock-gold-gears-schema-"));

const expected = new Map([
  ["GoldGearsProfile", "profiles.json"],
  ["GoldGearsArea", "areas.json"],
  ["GoldGearsDifficultySegment", "difficulty-segments.json"],
  ["GoldGearsPlane", "planes.json"],
  ["GoldGearsChessboard", "chessboards.json"],
  ["GoldGearsMapColumn", "map-columns.json"],
  ["GoldGearsMapNode", "map-nodes.json"],
  ["GoldGearsMapEdge", "map-edges.json"],
  ["GoldGearsMapEvent", "map-events.json"],
  ["GoldGearsBlockCreateRule", "block-create-rules.json"],
  ["GoldGearsRoom", "rooms.json"],
  ["GoldGearsDomain", "domains.json"],
  ["GoldGearsBeacon", "beacons.json"],
  ["GoldGearsBossChoice", "boss-choices.json"],
  ["GoldGearsCognitionRange", "cognition-ranges.json"],
  ["GoldGearsModeConstant", "mode-constants.json"],
  ["GoldGearsDiceCategory", "dice-categories.json"],
  ["GoldGearsDiceDefinition", "dice-definitions.json"],
  ["GoldGearsDicePathValue", "dice-path-values.json"],
  ["GoldGearsDiceSlot", "dice-slots.json"],
  ["GoldGearsDiceFace", "dice-faces.json"],
  ["GoldGearsDiceFaceTag", "dice-face-tags.json"],
  ["GoldGearsKnowledgeRule", "knowledge-rules.json"],
]);

try {
  assert(policy.version === "0.3.0", "Sora version policy differs");
  assert(fs.existsSync(sora), "pinned Sora 0.3.0 is not installed");
  assert(fs.existsSync(project) && fs.existsSync(schema), "isolated Gold and Gears schema is missing");
  const projectText = fs.readFileSync(project, "utf8");
  for (const forbidden of ["config/data", "config/generated", "config/universe-generated"]) {
    assert(!projectText.includes(forbidden), `isolated project references forbidden output ${forbidden}`);
  }
  const before = fs.readFileSync(schema);
  run("node", ["tools/gold-and-gears-reference/generate-sora-schema.mjs", root]);
  assert(before.equals(fs.readFileSync(schema)), "Gold and Gears core schema generation drifted");
  run(sora, ["--serial", "check", "--project", project]);
  const lock = path.join(temporary, "schema.lock");
  run(sora, ["--serial", "schema-lock", "--project", project, "--out", lock]);
  const parsed = JSON.parse(fs.readFileSync(lock, "utf8")).schema;
  assert(parsed.package === "starclock_gold_and_gears_reference_config", "schema package differs");
  const tables = new Map(parsed.tables.map((table) => [table.name, table]));
  assert(tables.size === expected.size, `expected ${expected.size} core tables, found ${tables.size}`);
  for (const [tableName, normalized] of expected) {
    assert(tables.has(tableName), `missing table ${tableName}`);
    assert(Array.isArray(json(path.join("content-reference/gold-and-gears-v1", normalized))), `${normalized} is not an array`);
    const stable = tables.get(tableName).fields.find((field) => field.name === "stable_key");
    assert(stable?.ty === "String", `${tableName}.stable_key is not typed as string`);
  }
  for (const [tableName, fieldName, target] of [
    ["GoldGearsChessboard", "start_node_id", "GoldGearsMapNode"],
    ["GoldGearsMapNode", "chessboard_id", "GoldGearsChessboard"],
    ["GoldGearsMapNode", "column_id", "GoldGearsMapColumn"],
    ["GoldGearsMapEdge", "source_node_id", "GoldGearsMapNode"],
    ["GoldGearsMapEdge", "target_node_id", "GoldGearsMapNode"],
    ["GoldGearsBlockCreateRule", "domain_id", "GoldGearsDomain"],
    ["GoldGearsDiceDefinition", "category_id", "GoldGearsDiceCategory"],
    ["GoldGearsDicePathValue", "dice_id", "GoldGearsDiceDefinition"],
    ["GoldGearsKnowledgeRule", "dice_face_id", "GoldGearsDiceFace"],
  ]) {
    const field = tables.get(tableName).fields.find((candidate) => candidate.name === fieldName);
    assert(
      field?.ty?.Ref?.table === target && field.ty.Ref.field === "id",
      `${tableName}.${fieldName} is not ref<${target}.id>`,
    );
  }
  console.log(`Gold and Gears core Sora schema verified (${tables.size} isolated tables, typed topology/dice/Knowledge references).`);
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

function run(command, arguments_) {
  const result = spawnSync(command, arguments_, { cwd: root, encoding: "utf8" });
  if (result.status !== 0) {
    throw new Error(`${command} ${arguments_.join(" ")} failed\n${result.stdout}\n${result.stderr}`);
  }
}

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
