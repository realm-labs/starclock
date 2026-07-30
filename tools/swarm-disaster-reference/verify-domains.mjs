#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  process.argv[2]
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const outputRoot = path.join(root, "content-reference/swarm-disaster-v1");
const json = async (relative) =>
  JSON.parse(await fs.readFile(path.join(root, relative), "utf8"));
const normalized = async (name) =>
  JSON.parse(await fs.readFile(path.join(outputRoot, name), "utf8"));
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function unique(values) {
  return values.length === new Set(values).size;
}

execFileSync(
  process.execPath,
  ["tools/swarm-disaster-reference/import-domains.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const manifest = await json(
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
const rooms = await normalized("rooms.json");
const domains = await normalized("domains.json");
const beacons = await normalized("beacons.json");
const bosses = await normalized("boss-choices.json");
const consequences = await normalized("topology-consequences.json");

assert(rooms.length === 861, `expected 861 rooms, found ${rooms.length}`);
assert(domains.length === 12, `expected 12 domains, found ${domains.length}`);
assert(beacons.length === 4, `expected 4 beacons, found ${beacons.length}`);
assert(bosses.length === 2, `expected 2 boss choices, found ${bosses.length}`);
assert(consequences.length === 13,
  `expected 13 topology consequences, found ${consequences.length}`);

for (const records of [rooms, domains, beacons, bosses, consequences]) {
  assert(unique(records.map(({ id }) => id)), "duplicate normalized ID");
  for (const row of records) {
    assert(row.coverage_state === "DataReady", `${row.id} is not DataReady`);
    assert(row.source_refs.length > 0, `${row.id} has no evidence`);
  }
}

const expectedRoomIds = new Set(manifest.categories.room_bindings.records
  .map(({ id }) => `swarm-disaster.room.${id}`));
assert(rooms.every(({ id }) => expectedRoomIds.delete(id))
  && expectedRoomIds.size === 0, "room manifest exact-once mismatch");
for (const room of rooms) {
  assert(room.sub_mode === "ChessRogue", `${room.id} sub-mode drift`);
  assert(room.section_ids.length > 0, `${room.id} has no section`);
  assert(room.domain_id === "" && room.encounter_pool_ids.length === 0,
    `${room.id} invented a room binding`);
  assert(room.encounter_binding_state === "DeferredToG09-P2-B5",
    `${room.id} encounter deferral drift`);
}

const expectedDomainIds = new Set(manifest.categories.domains.records
  .map(({ id }) => `swarm-disaster.domain.${id.toLowerCase()}`));
assert(domains.every(({ id }) => expectedDomainIds.delete(id))
  && expectedDomainIds.size === 0, "domain manifest exact-once mismatch");
for (const domain of domains) {
  assert(domain.selection_policy.candidate_order === "StableNodeId"
    && domain.replacement_policy.mutation_order === "StableNodeId",
  `${domain.id} unstable selection policy`);
  assert(domain.evidence_quality === "ProjectPolicy",
    `${domain.id} policy evidence label drift`);
}

const expectedBeaconIds = new Set(manifest.categories.beacons.records
  .map(({ id }) => `swarm-disaster.beacon.${id}`));
assert(beacons.every(({ id }) => expectedBeaconIds.delete(id))
  && expectedBeaconIds.size === 0, "beacon manifest exact-once mismatch");
for (const beacon of beacons) {
  assert(beacon.application_stage === "TopologyMutationResolution",
    `${beacon.id} application stage drift`);
  assert(beacon.copy_policy.includes("Explicitly")
    && beacon.blanking_policy.includes("Explicitly"),
  `${beacon.id} implicit copy/blanking policy`);
}

const expectedBossIds = new Set(manifest.categories.boss_choices.records
  .map(({ id }) => `swarm-disaster.boss-choice.${id}`));
assert(bosses.every(({ id }) => expectedBossIds.delete(id))
  && expectedBossIds.size === 0, "boss-choice manifest exact-once mismatch");
for (const boss of bosses) {
  assert(boss.weakness_consequence.elements.length > 0,
    `${boss.id} has no intrinsic weaknesses`);
  assert(
    boss.later_boss_consequence.resolution_state === "DeferredToG09-P1-B3",
    `${boss.id} boss-decay deferral drift`,
  );
}

const consequenceIds = new Set(consequences.map(({ source_id: id }) => id));
for (const required of [
  "202", "203", "204", "301", "302", "303", "304", "305",
  "504", "602", "801", "802", "804",
])
  assert(consequenceIds.delete(required), `missing dice consequence ${required}`);
assert(consequenceIds.size === 0, "unexpected topology consequence");
for (const consequence of consequences) {
  assert(consequence.trigger_kind === "AudienceDiceFace"
    && consequence.scope === "CurrentPlane",
  `${consequence.id} trigger/scope drift`);
  assert(consequence.ordered_operations.length === 1
    && consequence.ordered_operations[0].target_order === "StableNodeId"
    && consequence.ordered_operations[0].no_legal_target === "NoOp",
  `${consequence.id} operation policy drift`);
}

console.log(
  `Swarm Disaster domain verification passed: ${rooms.length} rooms, ` +
  `${domains.length} domains, ${beacons.length} beacons, ` +
  `${bosses.length} bosses, ${consequences.length} consequences.`,
);
