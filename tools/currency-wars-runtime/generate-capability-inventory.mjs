#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  classifyConfigurationType,
  mapConfigurationType,
  mapExpression,
  mapLifecycle,
  mapRecordShape,
  mapSelector,
  mapState,
  mapTrigger,
  sourceDomain,
} from "./capability-map.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const output = "content-manifests/currency-wars-runtime-v1/capability-inventory.json";
const dispositionInput = "content-manifests/currency-wars-runtime-v1/mechanic-dispositions.json";
const sourceRevision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const defaultSourceCache = ".cache/content-reference/turnbasedgamedata";
const lifecycleKeyPattern = /^On[A-Z_]/;
const statePropertyKeys = new Set(["Property", "PropertyType"]);
const selectorValueKeys = new Set(["Alias", "TargetType", "Team", "AliveStateMask"]);

const options = parseOptions(process.argv.slice(2));
const artifact = buildCapabilityInventory(options.sourceCache);
const serialized = `${JSON.stringify(artifact, null, 2)}\n`;
if (options.check) {
  const actual = fs.readFileSync(path.join(root, output), "utf8");
  assert(actual === serialized, `${output} is stale`);
  console.log(`Currency Wars capability inventory is current (${summaryLine(artifact)}).`);
} else {
  fs.writeFileSync(path.join(root, output), serialized);
  console.log(`Generated ${output} (${summaryLine(artifact)}).`);
}

export function buildCapabilityInventory(sourceCacheArgument = defaultSourceCache) {
  const sourceCache = path.resolve(root, sourceCacheArgument);
  verifyRevision(sourceCache);
  const disposition = json(dispositionInput);
  const accumulators = makeAccumulators();
  const sourceFiles = new Map();
  const programs = disposition.programs.map((program) => {
    const source = readSource(sourceCache, program, sourceFiles);
    const domain = sourceDomain(program);
    const local = makeLocalInventory();
    const recordShape = addRecordShape(accumulators.recordShapes, program, source.value, domain);
    if (program.target_execution !== "MetadataOnly")
      walk(source.value, {
        authoritative: true, domain, parentKey: "$root", program,
      }, accumulators, local);
    const assignedShapeIds = [
      ...local.configurationTypes,
      ...local.expressions,
      ...local.selectors,
      ...local.triggers,
      ...local.states,
      ...local.lifecycles,
    ].sort();
    return {
      mechanic_id: program.mechanic_id,
      target_execution: program.target_execution,
      scope: program.scope,
      capability: program.capability,
      domain,
      source_path: program.source_path,
      source_locator: program.source_locator,
      source_sha256: program.source_sha256,
      record_shape_id: recordShape,
      extracted_shape_counts: {
        configuration_types: local.configurationTypes.size,
        expressions: local.expressions.size,
        selectors: local.selectors.size,
        triggers: local.triggers.size,
        states: local.states.size,
        lifecycles: local.lifecycles.size,
      },
      extracted_shape_set_sha256: hashBytes(Buffer.from(assignedShapeIds.join("\n"))),
    };
  });
  assert(programs.length === 2_367, "mechanic program denominator drift");
  assert(new Set(programs.map(({ mechanic_id: id }) => id)).size === programs.length,
    "mechanic program inventory is not exact-once");

  const configurationTypes = finish(accumulators.configurationTypes);
  const expressions = finish(accumulators.expressions);
  const selectors = finish(accumulators.selectors);
  const triggers = finish(accumulators.triggers);
  const states = finish(accumulators.states);
  const lifecycles = finish(accumulators.lifecycles);
  const recordShapes = finish(accumulators.recordShapes);
  const opcodeBytes = expressionOpcodeBytes(expressions);
  const missingCapabilities = collectMissingCapabilities([
    configurationTypes, expressions, selectors, triggers, states, lifecycles, recordShapes,
  ], opcodeBytes);
  return {
    schema_revision: "starclock.currency-wars-capability-inventory.v1",
    goal_id: "currency-wars-runtime-v1",
    batch: "G21-P2-B1",
    source_repository: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    source_revision: sourceRevision,
    source_access_date: "2026-08-13",
    independent_cross_checks: [{
      url: "https://www.luogu.com/article/zcvu6fp7",
      access_date: "2026-08-13",
      evidence_quality: "IndependentPublicCrossCheck",
      scope: "Supports only postfix ordering and the historical byte-0 fixed-value / byte-1 dynamic-hash operand convention; Version 4.4 opcode semantics remain unresolved.",
    }],
    input_digests: {
      mechanic_dispositions: {
        path: dispositionInput,
        sha256: sha256File(path.join(root, dispositionInput)),
      },
    },
    evidence_notes: [
      "Every source record is read at the pinned revision and checked against its authored SHA-256 before shape extraction.",
      "Postfix expression byte values are inventoried without assigning unverified semantics. Public independent analysis supports byte 0/1 operand references and postfix ordering, but does not cover the Version 4.4 opcode set.",
      "ExistingPrimitive means the shared IR has a candidate semantic primitive; it does not claim that the source program has been lowered or executed.",
      "NonAuthoritative shapes are presentation or metadata boundaries and cannot mutate authoritative Activity or battle state.",
    ],
    summary: {
      mechanic_programs: programs.length,
      executable_programs: programs.filter(({ target_execution: target }) =>
        target !== "MetadataOnly").length,
      metadata_only_programs: programs.filter(({ target_execution: target }) =>
        target === "MetadataOnly").length,
      unique_source_files: sourceFiles.size,
      source_scopes: countBy(programs, ({ scope }) => scope),
      domains: countBy(programs, ({ domain }) => domain),
      configuration_types: new Set(configurationTypes.map(({ qualified_name: name }) => name)).size,
      configuration_type_shapes: configurationTypes.length,
      expression_shapes: expressions.length,
      postfix_opcode_sequences: new Set(expressions
        .filter(({ encoding }) => encoding === "PostfixBase64")
        .map(({ opcodes }) => opcodes)).size,
      postfix_opcode_bytes: opcodeBytes.length,
      selector_shapes: selectors.length,
      trigger_shapes: triggers.length,
      state_shapes: states.length,
      lifecycle_shapes: lifecycles.length,
      record_shapes: recordShapes.length,
      missing_capabilities: missingCapabilities.length,
      dispositions: countBy([
        ...configurationTypes, ...expressions, ...selectors, ...triggers,
        ...states, ...lifecycles, ...recordShapes,
      ], ({ mapping }) => mapping.disposition),
    },
    postfix_opcode_bytes: opcodeBytes,
    missing_capabilities: missingCapabilities,
    configuration_type_shapes: configurationTypes,
    expression_shapes: expressions,
    selector_shapes: selectors,
    trigger_shapes: triggers,
    state_shapes: states,
    lifecycle_shapes: lifecycles,
    record_shapes: recordShapes,
    programs,
  };
}

function walk(value, context, accumulators, local) {
  if (Array.isArray(value)) {
    for (const item of value)
      walk(item, context, accumulators, local);
    return;
  }
  if (value === null || typeof value !== "object")
    return;

  const fields = Object.keys(value).filter((key) => key !== "$type").sort();
  const configurationKind = typeof value.$type === "string"
    ? classifyConfigurationType(value.$type) : null;
  const authoritative = configurationKind === "Presentation"
    ? false
    : configurationKind === null || configurationKind === "Selector"
      ? context.authoritative : true;
  if (typeof value.$type === "string") {
    const mapping = mapConfigurationType(value.$type, context.domain);
    const id = addShape(accumulators.configurationTypes, "configuration-type", {
      domain: context.domain,
      qualified_name: value.$type,
      shape_kind: classifyConfigurationType(value.$type),
      parent_key: context.parentKey,
      fields,
      mapping,
    }, context.program.mechanic_id);
    local.configurationTypes.add(id);
    if (configurationKind === "Selector") {
      const selectorId = addShape(accumulators.selectors, "selector", {
        domain: context.domain,
        selector_kind: "TypedSelector",
        token: value.$type,
        parent_key: context.parentKey,
        fields,
        mapping: mapSelector(context.domain, authoritative),
      }, context.program.mechanic_id);
      local.selectors.add(selectorId);
    }
  }

  if (typeof value.OpCodes === "string"
      && Array.isArray(value.FixedValues) && Array.isArray(value.DynamicHashes)) {
    const bytes = [...Buffer.from(value.OpCodes, "base64")];
    const id = addShape(accumulators.expressions, "expression", {
      domain: context.domain,
      encoding: "PostfixBase64",
      opcodes: value.OpCodes,
      opcode_bytes: bytes,
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
      trigger: value.Event,
      callback_fields: fields,
      mapping: mapTrigger(context.domain, value.Event, authoritative),
    }, context.program.mechanic_id);
    local.triggers.add(id);
  }

  if (value.ReadInfo !== undefined && value.ReadInfo !== null
      && typeof value.ReadInfo === "object") {
    const id = addShape(accumulators.states, "state", {
      domain: context.domain,
      state_kind: "DynamicValueDefinition",
      fields: Object.keys(value.ReadInfo).sort(),
      read_type: scalarToken(value.ReadInfo.Type),
      mapping: mapState(context.domain, "DynamicValueDefinition", authoritative),
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
        mapping: mapSelector(context.domain, authoritative),
      }, context.program.mechanic_id);
      local.selectors.add(id);
    }
    if (statePropertyKeys.has(key) && typeof item === "string") {
      const id = addShape(accumulators.states, "state", {
        domain: context.domain,
        state_kind: key,
        token: item,
        mapping: mapState(context.domain, key, authoritative),
      }, context.program.mechanic_id);
      local.states.add(id);
    }
    if (lifecycleKeyPattern.test(key) && (Array.isArray(item) || isObject(item))) {
      const id = addShape(accumulators.lifecycles, "lifecycle", {
        domain: context.domain,
        hook: key,
        value_shape: valueShape(item),
        mapping: mapLifecycle(context.domain, key, authoritative),
      }, context.program.mechanic_id);
      local.lifecycles.add(id);
    }
    if (key === "Modifiers" && isObject(item)) {
      for (const definition of Object.values(item)) {
        if (!isObject(definition))
          continue;
        const id = addShape(accumulators.states, "state", {
          domain: context.domain,
          state_kind: "ModifierDefinition",
          fields: Object.keys(definition).sort(),
          mapping: mapState(context.domain, "ModifierDefinition", authoritative),
        }, context.program.mechanic_id);
        local.states.add(id);
      }
    }
    walk(item, { ...context, authoritative, parentKey: key }, accumulators, local);
  }
}

function addRecordShape(accumulator, program, value, domain) {
  return addShape(accumulator, "record", {
    domain,
    mechanic_family: program.capability,
    target_execution: program.target_execution,
    source_root: program.source_path.split("/").slice(0, 2).join("/"),
    value_shape: valueShape(value),
    fields: isObject(value) ? Object.keys(value).sort() : [],
    mapping: mapRecordShape(program),
  }, program.mechanic_id);
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

function readSource(sourceCache, program, sourceFiles) {
  const sourcePath = path.join(sourceCache, program.source_path);
  const bytes = fs.readFileSync(sourcePath);
  const fileDigest = hashBytes(bytes);
  const previous = sourceFiles.get(program.source_path);
  if (previous !== undefined)
    assert(previous === fileDigest, `source file changed while reading: ${program.source_path}`);
  sourceFiles.set(program.source_path, fileDigest);
  const parsed = JSON.parse(bytes.toString("utf8"));
  if (program.source_path.startsWith("ExcelOutput/")) {
    const row = parsed[Number(program.source_locator)];
    assert(row !== undefined, `missing source locator ${program.source_locator}: ${program.source_path}`);
    assert(hashBytes(Buffer.from(JSON.stringify(row))) === program.source_sha256,
      `source row digest drift: ${program.mechanic_id}`);
    return { value: row };
  }
  assert(fileDigest === program.source_sha256, `source file digest drift: ${program.mechanic_id}`);
  return { value: parsed };
}

function expressionOpcodeBytes(expressions) {
  const counts = new Map();
  for (const expression of expressions) {
    if (expression.encoding !== "PostfixBase64")
      continue;
    for (const byte of expression.opcode_bytes)
      counts.set(byte, (counts.get(byte) ?? 0) + expression.occurrence_count);
  }
  return [...counts].sort(([left], [right]) => left - right)
    .map(([byte, occurrenceCount]) => ({
      byte,
      hexadecimal: `0x${byte.toString(16).padStart(2, "0")}`,
      occurrence_count: occurrenceCount,
      semantic_status: "UnresolvedExactByte",
      missing_capability: "shared.version-4.4-postfix-opcode-semantics",
    }));
}

function collectMissingCapabilities(groups, opcodeBytes) {
  const capabilities = new Map();
  for (const shape of groups.flat()) {
    const capability = shape.mapping.missing_capability;
    if (capability === null || shape.mapping.disposition === "NonAuthoritative")
      continue;
    let entry = capabilities.get(capability);
    if (entry === undefined) {
      entry = { capability, shape_ids: [], programCount: new Set() };
      capabilities.set(capability, entry);
    }
    entry.shape_ids.push(shape.shape_id);
    for (const id of shape.sample_mechanic_ids)
      entry.programCount.add(id);
  }
  for (const opcode of opcodeBytes) {
    const capability = opcode.missing_capability;
    if (!capabilities.has(capability))
      capabilities.set(capability, { capability, shape_ids: [], programCount: new Set() });
  }
  return [...capabilities.values()].map(({ capability, shape_ids: shapeIds }) => ({
    capability,
    shape_count: shapeIds.length,
    shape_ids: shapeIds.sort(),
  })).sort((left, right) => left.capability.localeCompare(right.capability));
}

function makeAccumulators() {
  return {
    configurationTypes: new Map(),
    expressions: new Map(),
    selectors: new Map(),
    triggers: new Map(),
    states: new Map(),
    lifecycles: new Map(),
    recordShapes: new Map(),
  };
}

function makeLocalInventory() {
  return {
    configurationTypes: new Set(), expressions: new Set(), selectors: new Set(),
    triggers: new Set(), states: new Set(), lifecycles: new Set(),
  };
}

function parseOptions(args) {
  let check = false;
  let sourceCache = defaultSourceCache;
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--check")
      check = true;
    else if (argument === "--source-cache") {
      sourceCache = args[index + 1];
      index += 1;
    } else
      throw new Error(`unknown argument: ${argument}`);
  }
  return { check, sourceCache };
}

function verifyRevision(sourceCache) {
  const head = fs.readFileSync(path.join(sourceCache, ".git/HEAD"), "utf8").trim();
  const revision = head.startsWith("ref: ")
    ? fs.readFileSync(path.join(sourceCache, ".git", head.slice(5)), "utf8").trim()
    : head;
  assert(revision === sourceRevision,
    `source cache must be detached at ${sourceRevision}; found ${revision}`);
}

function valueShape(value) {
  if (Array.isArray(value))
    return "Array";
  if (value === null)
    return "Null";
  if (typeof value === "object")
    return "Object";
  return typeof value;
}

function scalarToken(value) {
  return ["string", "number", "boolean"].includes(typeof value) ? String(value) : null;
}

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function stableId(prefix, value) {
  return `currency-wars.capability-${prefix}.${hashBytes(Buffer.from(value)).slice(0, 24)}`;
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

function json(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

function sha256File(file) {
  return hashBytes(fs.readFileSync(file));
}

function hashBytes(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function summaryLine(value) {
  return `${value.summary.mechanic_programs} programs; ${value.summary.configuration_types} types; `
    + `${value.summary.missing_capabilities} named gaps`;
}

function assert(condition, message) {
  if (!condition)
    throw new Error(message);
}
