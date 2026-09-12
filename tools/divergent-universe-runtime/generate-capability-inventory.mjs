#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import {
  classifySourceType,
  isPresentationOperation,
  mapExpression,
  mapLifecycle,
  mapOperation,
  mapRecordShape,
  mapSelector,
  mapState,
  mapTrigger,
  mapTypedCondition,
  sourceDomain,
} from "./capability-map.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/divergent-universe-runtime-v1/capability-inventory.json";
const dispositionInput =
  "content-manifests/divergent-universe-runtime-v1/mechanic-dispositions.json";
const rulesInput = "content-reference/divergent-universe-v1/mechanic-rules.json";
const sourceInventoryInput =
  "content-manifests/divergent-universe-v1/source-inventory.json";
const sourceRevision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const defaultSourceCache = ".cache/content-reference/turnbasedgamedata";
const typeFields = ["$type", "Type", "type", "ActionType"];
const selectorValueKeys = new Set(["Alias", "TargetType", "Team", "AliveStateMask"]);
const stateValueKeys = new Set([
  "DynamicKey", "Property", "PropertyType", "ValueType", "DynamicValueKey",
]);
const lifecycleKeyPattern = /^On[A-Z_]/;

const options = parseOptions(process.argv.slice(2));
const artifact = buildCapabilityInventory(options.sourceCache);
const serialized = pretty(artifact);
if (options.check) {
  const actual = fs.readFileSync(path.join(root, output), "utf8");
  assert(actual === serialized, `${output} is stale`);
  console.log(`Divergent Universe capability inventory is current (${summaryLine(artifact)}).`);
} else {
  fs.writeFileSync(path.join(root, output), serialized);
  console.log(`Generated ${output} (${summaryLine(artifact)}).`);
}

export function buildCapabilityInventory(sourceCacheArgument = defaultSourceCache) {
  const sourceCache = path.resolve(root, sourceCacheArgument);
  verifyRevision(sourceCache);
  const disposition = json(dispositionInput);
  const rules = json(rulesInput);
  const sourceInventory = json(sourceInventoryInput);
  const inventoryByPath = new Map(sourceInventory.records
    .filter(({ repository }) => repository === "turnbasedgamedata")
    .map((record) => [record.path, record]));
  const ruleById = new Map(rules.map((rule) => [rule.id, rule]));
  assert(ruleById.size === rules.length, "mechanic rule identity is not unique");
  const sourcePaths = disposition.programs.map((program) => {
    const rule = required(ruleById.get(program.mechanic_id), `rule ${program.mechanic_id}`);
    const source = required(rule.source_refs?.[0], `source reference ${program.mechanic_id}`);
    assert(source.path === program.source_id, `source path drift: ${program.mechanic_id}`);
    assert(source.locator === "root", `non-root mechanic source: ${program.mechanic_id}`);
    assert(source.revision === sourceRevision, `source revision drift: ${program.mechanic_id}`);
    return source.path;
  });
  const sourceBytes = readGitBlobs(sourceCache, sourcePaths);
  const accumulators = makeAccumulators();
  const programs = disposition.programs.map((program) => {
    const rule = required(ruleById.get(program.mechanic_id), `rule ${program.mechanic_id}`);
    const source = rule.source_refs[0];
    const bytes = required(sourceBytes.get(source.path), `source bytes ${source.path}`);
    const inventoryRecord = required(inventoryByPath.get(source.path),
      `source inventory row ${source.path}`);
    assert(hashBytes(bytes) === inventoryRecord.sha256,
      `raw source digest drift: ${program.mechanic_id}`);
    assert(bytes.length === inventoryRecord.bytes,
      `raw source byte count drift: ${program.mechanic_id}`);
    const value = JSON.parse(bytes.toString("utf8")
      .replace(/("Hash"\s*:\s*)(-?\d{16,})/gu, '$1"$2"'));
    assert(hashBytes(Buffer.from(JSON.stringify(value))) === source.sha256,
      `source evidence digest drift: ${program.mechanic_id}`);
    const domain = sourceDomain(program);
    const local = makeLocalInventory();
    const audit = { order: [], counts: new Map() };
    const recordShape = addShape(accumulators.recordShapes, "record", {
      domain,
      scope: program.scope,
      trigger: program.trigger,
      state_lifetime: program.state_lifetime,
      execution_disposition: program.execution_disposition,
      value_shape: valueShape(value),
      fields: isObject(value) ? Object.keys(value).sort() : [],
      mapping: mapRecordShape(program),
    }, program.mechanic_id);
    addProgramBoundaryShapes(program, domain, accumulators, local);
    walk(value, {
      authoritative: domain !== "Metadata",
      domain,
      parentKey: "$root",
      program,
    }, accumulators, local, audit);
    if (audit.order.length === 0)
      addFallbackStructureShapes(value, program, domain, accumulators, local, audit);
    verifyOperationAudit(program, rule, audit);
    const assignedShapeIds = sorted(new Set([
      ...local.expressions,
      ...local.selectors,
      ...local.triggers,
      ...local.operations,
      ...local.states,
      ...local.lifecycles,
    ]));
    return {
      mechanic_id: program.mechanic_id,
      scope: program.scope,
      trigger: program.trigger,
      state_lifetime: program.state_lifetime,
      execution_disposition: program.execution_disposition,
      execution_owner: program.execution_owner,
      execution_partition: program.execution_partition,
      domain,
      source_path: source.path,
      source_locator: source.locator,
      source_sha256: source.sha256,
      record_shape_id: recordShape,
      operation_occurrence_count: sum([...audit.counts.values()]),
      extracted_shape_ids: {
        expressions: sorted(local.expressions),
        selectors: sorted(local.selectors),
        triggers: sorted(local.triggers),
        operations: sorted(local.operations),
        states: sorted(local.states),
        lifecycles: sorted(local.lifecycles),
      },
      extracted_shape_counts: {
        expressions: local.expressions.size,
        selectors: local.selectors.size,
        triggers: local.triggers.size,
        operations: local.operations.size,
        states: local.states.size,
        lifecycles: local.lifecycles.size,
      },
      extracted_shape_set_sha256: hashBytes(Buffer.from(assignedShapeIds.join("\n"))),
    };
  });
  assert(programs.length === 669, "mechanic program denominator drift");
  assert(new Set(programs.map(({ mechanic_id: id }) => id)).size === programs.length,
    "mechanic program inventory is not exact-once");

  const expressions = finish(accumulators.expressions);
  const selectors = finish(accumulators.selectors);
  const triggers = finish(accumulators.triggers);
  const operations = finish(accumulators.operations);
  const states = finish(accumulators.states);
  const lifecycles = finish(accumulators.lifecycles);
  const recordShapes = finish(accumulators.recordShapes);
  const opcodeBytes = expressionOpcodeBytes(expressions);
  const allShapes = [
    expressions, selectors, triggers, operations, states, lifecycles, recordShapes,
  ];
  const missingCapabilities = collectMissingCapabilities(allShapes, opcodeBytes);
  return {
    schema_revision: "starclock.divergent-universe-capability-inventory.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P2-B1",
    source_repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    source_revision: sourceRevision,
    source_access_date: "2026-07-22",
    input_digests: {
      mechanic_dispositions: {
        path: dispositionInput,
        sha256: sha256File(path.join(root, dispositionInput)),
      },
      mechanic_rules: {
        path: rulesInput,
        sha256: sha256File(path.join(root, rulesInput)),
      },
      source_inventory: {
        path: sourceInventoryInput,
        sha256: sha256File(path.join(root, sourceInventoryInput)),
      },
    },
    inventory_contract: {
      operation_shapes:
        "Every string discriminator at $type, Type, type or ActionType, matching the released reference-pack operation audit exactly.",
      expression_shapes:
        "Every postfix OpCodes/FixedValues/DynamicHashes expression, fixed IsDynamic/FixedValue expression and typed By*/Predicate condition.",
      selector_shapes:
        "Every typed selector plus scalar Alias, TargetType, Team and AliveStateMask selector token.",
      trigger_shapes:
        "Every program trigger and every raw Event string.",
      state_shapes:
        "Every typed state configuration, modifier definition, ReadInfo object and named dynamic/property state reference.",
      lifecycle_shapes:
        "Every program state lifetime and raw On* object/array lifecycle hook.",
    },
    evidence_notes: [
      "Every source file is read as a Git blob from the pinned revision and checked against its authored SHA-256 before extraction.",
      "ExistingPrimitive identifies a shared IR candidate only; it does not claim a source program has been lowered or executed.",
      "MissingCapability is deliberately conservative when source field semantics do not map exactly to the current shared vocabulary.",
      "NonAuthoritative shapes are presentation, null-token or source-layout boundaries and cannot mutate authoritative state.",
      "Postfix byte values remain unresolved; no opcode semantics are inferred from byte identity or ordering.",
    ],
    summary: {
      mechanic_programs: programs.length,
      executable_programs: programs.filter(({ execution_disposition: value }) =>
        value === "ExactExecutable").length,
      metadata_only_programs: programs.filter(({ execution_disposition: value }) =>
        value === "MetadataOnly").length,
      excluded_programs: programs.filter(({ execution_disposition: value }) =>
        value === "ExcludedWithProof").length,
      unique_source_files: sourceBytes.size,
      source_scopes: countBy(programs, ({ scope }) => scope),
      domains: countBy(programs, ({ domain }) => domain),
      expression_shapes: expressions.length,
      postfix_opcode_sequences: new Set(expressions
        .filter(({ encoding }) => encoding === "PostfixBase64")
        .map(({ opcodes }) => opcodes)).size,
      postfix_opcode_bytes: opcodeBytes.length,
      selector_shapes: selectors.length,
      trigger_shapes: triggers.length,
      operation_types: new Set(operations.map(({ qualified_name: name }) => name)).size,
      operation_shapes: operations.length,
      operation_occurrences: sum(programs.map(
        ({ operation_occurrence_count: count }) => count)),
      state_shapes: states.length,
      lifecycle_shapes: lifecycles.length,
      record_shapes: recordShapes.length,
      missing_capabilities: missingCapabilities.length,
      dispositions: countBy(allShapes.flat(), ({ mapping }) => mapping.disposition),
    },
    postfix_opcode_bytes: opcodeBytes,
    missing_capabilities: missingCapabilities,
    expression_shapes: expressions,
    selector_shapes: selectors,
    trigger_shapes: triggers,
    operation_shapes: operations,
    state_shapes: states,
    lifecycle_shapes: lifecycles,
    record_shapes: recordShapes,
    programs,
  };
}

function walk(value, context, accumulators, local, audit) {
  if (Array.isArray(value)) {
    for (const item of value) walk(item, context, accumulators, local, audit);
    return;
  }
  if (!isObject(value)) return;

  const fields = Object.keys(value).filter((key) => key !== "$type").sort();
  const typed = [];
  for (const field of typeFields) {
    const token = value[field];
    if (typeof token !== "string") continue;
    addAuditToken(audit, token);
    const mapping = mapOperation(token, context.domain);
    const id = addShape(accumulators.operations, "operation", {
      domain: context.domain,
      qualified_name: token,
      discriminator_field: field,
      shape_kind: classifySourceType(token),
      parent_key: context.parentKey,
      fields,
      mapping,
    }, context.program.mechanic_id);
    local.operations.add(id);
    typed.push({ token, mapping });
  }
  const authoritative = context.authoritative
    && !typed.some(({ token }) => isPresentationOperation(token, context.domain));

  for (const { token } of typed) {
    const kind = classifySourceType(token);
    if (kind === "Selector") {
      const id = addShape(accumulators.selectors, "selector", {
        domain: context.domain,
        selector_kind: "TypedSelector",
        token,
        parent_key: context.parentKey,
        fields,
        mapping: mapSelector(context.domain, token, authoritative),
      }, context.program.mechanic_id);
      local.selectors.add(id);
    } else if (kind === "Condition") {
      const id = addShape(accumulators.expressions, "expression", {
        domain: context.domain,
        encoding: "TypedCondition",
        token,
        parent_key: context.parentKey,
        fields,
        mapping: mapTypedCondition(context.domain, token, authoritative),
      }, context.program.mechanic_id);
      local.expressions.add(id);
    } else if (kind === "State") {
      const id = addShape(accumulators.states, "state", {
        domain: context.domain,
        state_kind: "TypedConfiguration",
        token,
        parent_key: context.parentKey,
        fields,
        mapping: mapState(context.domain, "TypedConfiguration", authoritative),
      }, context.program.mechanic_id);
      local.states.add(id);
    }
  }

  if (typeof value.OpCodes === "string"
      && Array.isArray(value.FixedValues) && Array.isArray(value.DynamicHashes)) {
    const id = addShape(accumulators.expressions, "expression", {
      domain: context.domain,
      encoding: "PostfixBase64",
      opcodes: value.OpCodes,
      opcode_bytes: [...Buffer.from(value.OpCodes, "base64")],
      fixed_value_count: value.FixedValues.length,
      dynamic_hash_count: value.DynamicHashes.length,
      mapping: mapExpression(context.domain, true, authoritative),
    }, context.program.mechanic_id);
    local.expressions.add(id);
  }
  if (value.IsDynamic === false && value.FixedValue !== undefined) {
    const id = addShape(accumulators.expressions, "expression", {
      domain: context.domain,
      encoding: "FixedValue",
      value_shape: valueShape(value.FixedValue),
      mapping: mapExpression(context.domain, false, authoritative),
    }, context.program.mechanic_id);
    local.expressions.add(id);
  }
  if (typeof value.Event === "string") {
    const id = addShape(accumulators.triggers, "trigger", {
      domain: context.domain,
      trigger_kind: "SourceEvent",
      trigger: value.Event,
      callback_fields: fields,
      mapping: mapTrigger(context.domain, value.Event, authoritative),
    }, context.program.mechanic_id);
    local.triggers.add(id);
  }
  if (isObject(value.ReadInfo)) {
    const id = addShape(accumulators.states, "state", {
      domain: context.domain,
      state_kind: "DynamicValueReadInfo",
      fields: Object.keys(value.ReadInfo).sort(),
      read_type: scalarToken(value.ReadInfo.Type),
      mapping: mapState(context.domain, "DynamicValueReadInfo", authoritative),
    }, context.program.mechanic_id);
    local.states.add(id);
  }

  for (const [key, item] of Object.entries(value)) {
    if (selectorValueKeys.has(key) && typeof item === "string") {
      const id = addShape(accumulators.selectors, "selector", {
        domain: context.domain,
        selector_kind: key,
        token: item,
        parent_key: context.parentKey,
        fields: [],
        mapping: mapSelector(context.domain, `${key}:${item}`, authoritative),
      }, context.program.mechanic_id);
      local.selectors.add(id);
    }
    if (stateValueKeys.has(key) && typeof item === "string") {
      const id = addShape(accumulators.states, "state", {
        domain: context.domain,
        state_kind: key,
        token: item,
        parent_key: context.parentKey,
        mapping: mapState(context.domain, key, authoritative),
      }, context.program.mechanic_id);
      local.states.add(id);
    }
    if (lifecycleKeyPattern.test(key) && (Array.isArray(item) || isObject(item))) {
      const id = addShape(accumulators.lifecycles, "lifecycle", {
        domain: context.domain,
        lifecycle_kind: "SourceHook",
        hook: key,
        value_shape: valueShape(item),
        mapping: mapLifecycle(context.domain, key, authoritative),
      }, context.program.mechanic_id);
      local.lifecycles.add(id);
    }
    if (key === "Modifiers" && isObject(item)) {
      for (const definition of Object.values(item)) {
        if (!isObject(definition)) continue;
        const id = addShape(accumulators.states, "state", {
          domain: context.domain,
          state_kind: "ModifierDefinition",
          fields: Object.keys(definition).sort(),
          mapping: mapState(context.domain, "ModifierDefinition", authoritative),
        }, context.program.mechanic_id);
        local.states.add(id);
      }
    }
    walk(item, { ...context, authoritative, parentKey: key }, accumulators, local, audit);
  }
}

function addProgramBoundaryShapes(program, domain, accumulators, local) {
  const trigger = addShape(accumulators.triggers, "trigger", {
    domain,
    trigger_kind: "ProgramBoundary",
    trigger: program.trigger,
    callback_fields: [],
    mapping: mapTrigger(domain, program.trigger, domain !== "Metadata"),
  }, program.mechanic_id);
  local.triggers.add(trigger);
  const lifecycle = addShape(accumulators.lifecycles, "lifecycle", {
    domain,
    lifecycle_kind: "ProgramStateLifetime",
    hook: program.state_lifetime,
    snapshot_policy: program.snapshot_policy,
    value_shape: "ProgramBoundary",
    mapping: mapLifecycle(domain, program.state_lifetime, domain !== "Metadata"),
  }, program.mechanic_id);
  local.lifecycles.add(lifecycle);
}

function addFallbackStructureShapes(value, program, domain, accumulators, local, audit) {
  const tokens = Array.isArray(value)
    ? ["Structure:Array"]
    : Object.keys(value).sort().map((key) => `Structure:${key}`);
  for (const token of tokens.length > 0 ? tokens : ["Structure:EmptyObject"]) {
    addAuditToken(audit, token);
    const id = addShape(accumulators.operations, "operation", {
      domain,
      qualified_name: token,
      discriminator_field: "SyntheticStructureAudit",
      shape_kind: "Structure",
      parent_key: "$root",
      fields: isObject(value) ? Object.keys(value).sort() : [],
      mapping: mapOperation(token, domain),
    }, program.mechanic_id);
    local.operations.add(id);
  }
}

function verifyOperationAudit(program, rule, audit) {
  const expected = rule.ordered_operations;
  assert(audit.order.length === expected.length,
    `operation type count drift: ${program.mechanic_id}`);
  for (let index = 0; index < expected.length; index += 1) {
    assert(audit.order[index] === expected[index].operation_type,
      `operation order drift at ${index + 1}: ${program.mechanic_id}`);
    assert(audit.counts.get(audit.order[index]) === expected[index].source_occurrences,
      `operation occurrence drift at ${index + 1}: ${program.mechanic_id}`);
  }
  assert(sum([...audit.counts.values()]) === program.operation_shape_count,
    `operation denominator drift: ${program.mechanic_id}`);
}

function addAuditToken(audit, token) {
  if (!audit.counts.has(token)) audit.order.push(token);
  audit.counts.set(token, (audit.counts.get(token) ?? 0) + 1);
}

function addShape(accumulator, prefix, shape, mechanicId) {
  const key = JSON.stringify(shape);
  let entry = accumulator.get(key);
  if (entry === undefined) {
    entry = { id: stableId(prefix, key), shape, occurrences: 0, programs: new Set() };
    accumulator.set(key, entry);
  }
  entry.occurrences += 1;
  entry.programs.add(mechanicId);
  return entry.id;
}

function finish(accumulator) {
  return [...accumulator.values()].map(({ id, shape, occurrences, programs }) => {
    const mechanicIds = sorted(programs);
    return {
      shape_id: id,
      ...shape,
      occurrence_count: occurrences,
      program_count: programs.size,
      sample_mechanic_ids: mechanicIds.slice(0, 3),
      ...(shape.mapping.disposition === "MissingCapability"
        ? { mechanic_ids: mechanicIds } : {}),
    };
  }).sort((left, right) => left.shape_id.localeCompare(right.shape_id));
}

function expressionOpcodeBytes(expressions) {
  const counts = new Map();
  const reachable = expressions.some(({ encoding, domain }) =>
    encoding === "PostfixBase64" && domain !== "Metadata");
  for (const expression of expressions) {
    if (expression.encoding !== "PostfixBase64") continue;
    for (const byte of expression.opcode_bytes)
      counts.set(byte, (counts.get(byte) ?? 0) + expression.occurrence_count);
  }
  return [...counts].sort(([left], [right]) => left - right)
    .map(([byte, occurrenceCount]) => ({
      byte,
      hexadecimal: `0x${byte.toString(16).padStart(2, "0")}`,
      occurrence_count: occurrenceCount,
      semantic_status: "UnresolvedExactByte",
      missing_capability: reachable
        ? "shared.version-4.4-postfix-opcode-semantics" : null,
    }));
}

function collectMissingCapabilities(groups, opcodeBytes) {
  const capabilities = new Map();
  for (const shape of groups.flat()) {
    if (shape.mapping.disposition !== "MissingCapability") continue;
    const capability = shape.mapping.missing_capability;
    const entry = capabilities.get(capability) ?? {
      capability, shapeIds: new Set(), mechanicIds: new Set(),
    };
    entry.shapeIds.add(shape.shape_id);
    for (const id of shape.mechanic_ids ?? shape.sample_mechanic_ids)
      entry.mechanicIds.add(id);
    capabilities.set(capability, entry);
  }
  for (const opcode of opcodeBytes) {
    if (opcode.missing_capability === null) continue;
    if (!capabilities.has(opcode.missing_capability))
      capabilities.set(opcode.missing_capability, {
        capability: opcode.missing_capability,
        shapeIds: new Set(), mechanicIds: new Set(),
      });
  }
  return [...capabilities.values()].map((entry) => ({
    capability: entry.capability,
    shape_count: entry.shapeIds.size,
    program_count: entry.mechanicIds.size,
    shape_ids: sorted(entry.shapeIds),
    sample_mechanic_ids: sorted(entry.mechanicIds).slice(0, 3),
  })).sort((left, right) => left.capability.localeCompare(right.capability));
}

function readGitBlobs(sourceCache, paths) {
  const uniquePaths = sorted(new Set(paths));
  const input = `${uniquePaths.map((value) => `${sourceRevision}:${value}`).join("\n")}\n`;
  const result = spawnSync("git", ["-C", sourceCache, "cat-file", "--batch"], {
    input,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
    maxBuffer: 512 * 1024 * 1024,
    windowsHide: true,
  });
  assert(result.status === 0,
    `git cat-file failed: ${result.stderr?.toString("utf8").trim() ?? "unknown error"}`);
  const outputBytes = result.stdout;
  const blobs = new Map();
  let offset = 0;
  for (const sourcePath of uniquePaths) {
    const lineEnd = outputBytes.indexOf(0x0a, offset);
    assert(lineEnd >= 0, `missing Git blob header: ${sourcePath}`);
    const header = outputBytes.subarray(offset, lineEnd).toString("utf8");
    const fields = header.split(" ");
    assert(fields.length === 3 && fields[1] === "blob",
      `Git object is unavailable or not a blob: ${sourcePath} (${header})`);
    const size = Number(fields[2]);
    assert(Number.isSafeInteger(size) && size >= 0, `invalid Git blob size: ${sourcePath}`);
    const start = lineEnd + 1;
    const end = start + size;
    assert(end < outputBytes.length && outputBytes[end] === 0x0a,
      `truncated Git blob: ${sourcePath}`);
    blobs.set(sourcePath, Buffer.from(outputBytes.subarray(start, end)));
    offset = end + 1;
  }
  assert(offset === outputBytes.length, "unexpected trailing Git batch output");
  return blobs;
}

function makeAccumulators() {
  return {
    expressions: new Map(), selectors: new Map(), triggers: new Map(),
    operations: new Map(), states: new Map(), lifecycles: new Map(),
    recordShapes: new Map(),
  };
}

function makeLocalInventory() {
  return {
    expressions: new Set(), selectors: new Set(), triggers: new Set(),
    operations: new Set(), states: new Set(), lifecycles: new Set(),
  };
}

function parseOptions(args) {
  let check = false;
  let sourceCache = defaultSourceCache;
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--check") check = true;
    else if (argument === "--source-cache") {
      sourceCache = required(args[index + 1], "--source-cache value");
      index += 1;
    } else throw new Error(`unknown argument: ${argument}`);
  }
  return { check, sourceCache };
}

function verifyRevision(sourceCache) {
  const revision = execFileSync("git", ["-C", sourceCache, "rev-parse", "HEAD"], {
    encoding: "utf8", windowsHide: true,
  }).trim();
  assert(revision === sourceRevision,
    `source cache must be detached at ${sourceRevision}; found ${revision}`);
}

function valueShape(value) {
  if (Array.isArray(value)) return "Array";
  if (value === null) return "Null";
  if (typeof value === "object") return "Object";
  return typeof value;
}

function scalarToken(value) {
  return ["string", "number", "boolean"].includes(typeof value) ? String(value) : null;
}

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function stableId(prefix, value) {
  return `divergent-universe.capability-${prefix}.${hashBytes(Buffer.from(value)).slice(0, 24)}`;
}

function countBy(values, selector) {
  const counts = {};
  for (const value of values) {
    const key = selector(value);
    counts[key] = (counts[key] ?? 0) + 1;
  }
  return Object.fromEntries(Object.entries(counts).sort(([left], [right]) =>
    left.localeCompare(right)));
}

function sorted(values) {
  return [...values].sort();
}

function sum(values) {
  return values.reduce((total, value) => total + value, 0);
}

function required(value, label) {
  assert(value !== undefined && value !== null, `missing ${label}`);
  return value;
}

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function sha256File(file) {
  return hashBytes(fs.readFileSync(file));
}

function hashBytes(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function pretty(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function summaryLine(value) {
  return `${value.summary.mechanic_programs} programs; `
    + `${value.summary.operation_types} operation types; `
    + `${value.summary.missing_capabilities} named gaps`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
