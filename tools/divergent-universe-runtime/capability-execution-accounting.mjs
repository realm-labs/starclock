// Current disposition accounting, not proof that an IR candidate executes.

const executableDispositions = new Set([
  "ExactRuleIr", "ExactActivityProgram", "PolicyRuleIr", "PolicyActivityProgram",
  "StaticHandler",
]);
const nonRuntimeDispositions = new Set(["MetadataOnly", "ExcludedWithProof"]);

export function summarizeProgramExecution(programs) {
  const identities = new Set();
  const dispositions = new Map();
  const statuses = new Map();
  let executable = 0;
  for (const program of programs) {
    const id = program.mechanic_id;
    if (typeof id !== "string" || id.length === 0 || identities.has(id))
      throw new Error(`mechanic inventory identity is missing or duplicated: ${id}`);
    identities.add(id);
    const disposition = program.execution_disposition;
    if (!executableDispositions.has(disposition)
      && !nonRuntimeDispositions.has(disposition) && disposition !== "Pending")
      throw new Error(`unsupported current execution disposition: ${id}: ${disposition}`);
    const expectedStatus = disposition === "Pending" ? "Pending" : "Terminal";
    if (program.runtime_status !== expectedStatus)
      throw new Error(`execution disposition/status mismatch: ${id}: ${disposition}`);
    dispositions.set(disposition, (dispositions.get(disposition) ?? 0) + 1);
    statuses.set(expectedStatus, (statuses.get(expectedStatus) ?? 0) + 1);
    if (executableDispositions.has(disposition)) executable += 1;
  }
  return {
    executable_programs: executable,
    metadata_only_programs: dispositions.get("MetadataOnly") ?? 0,
    excluded_programs: dispositions.get("ExcludedWithProof") ?? 0,
    pending_programs: dispositions.get("Pending") ?? 0,
    terminal_programs: statuses.get("Terminal") ?? 0,
    execution_dispositions: orderedCounts(dispositions),
    runtime_status: orderedCounts(statuses),
  };
}

function orderedCounts(counts) {
  return Object.fromEntries([...counts].sort(([left], [right]) =>
    left < right ? -1 : left > right ? 1 : 0));
}
