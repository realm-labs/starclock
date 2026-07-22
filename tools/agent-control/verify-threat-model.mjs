import { readFile } from "node:fs/promises";

const model = JSON.parse(await readFile("policy/agent-control-threat-model.json", "utf8"));
const fail = (message) => { throw new Error(`agent-control threat model: ${message}`); };
if (model.schema_revision !== "starclock.agent-control-threat-model.v1") fail("revision drift");
if (model.threats.length !== 19) fail("expected 19 enumerated threats");
const ids = model.threats.map((threat) => threat.id);
if (new Set(ids).size !== ids.length || ids.some((id, index) => id !== `G02-T${String(index + 1).padStart(2, "0")}`)) fail("threat IDs are missing, duplicate or unordered");
for (const threat of model.threats) {
  if (!threat.threat || threat.controls.length < 2 || threat.verification.length === 0) fail(`${threat.id} lacks controls or verification`);
  if (threat.verification.some((batch) => !/^G02-P[0-5]-B[1-7]$/.test(batch))) fail(`${threat.id} has invalid verification owner`);
}
const scopes = new Set(model.authorization_scopes);
for (const scope of ["starclock:battle:read", "starclock:battle:act", "starclock:battle:replay", "starclock:debug:omniscient"]) if (!scopes.has(scope)) fail(`missing scope ${scope}`);
const remote = model.startup_profiles.find((profile) => profile.profile === "http_remote");
for (const required of ["TLS or explicitly attested TLS-terminating proxy", "token signature issuer audience and expiry validation", "nonempty exact Origin allowlist", "tenant and principal extraction", "scope enforcement", "rate limits", "security audit sink"]) if (!remote.required.includes(required)) fail(`remote startup omits ${required}`);
for (const forbidden of ["anonymous requests", "wildcard Origin", "token passthrough", "startup with any missing required control"]) if (!remote.forbidden.includes(forbidden)) fail(`remote startup does not forbid ${forbidden}`);
for (const [name, value] of Object.entries(model.operational_limits)) if (!Number.isSafeInteger(value) || value <= 0) fail(`invalid operational limit ${name}`);
if (model.operational_limits.replay_records !== 1_000_000 || model.operational_limits.replay_record_payload_bytes !== 16 * 1024 * 1024) fail("replay limits disagree with Goal 01 codec");
console.log("agent-control threat model verified: 19 threats, 3 startup profiles, 8 scopes");
