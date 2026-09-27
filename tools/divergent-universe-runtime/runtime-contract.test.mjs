import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { buildRuntimeContract } from "./generate-runtime-contract.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const dispositionFile = "content-manifests/divergent-universe-runtime-v1/runtime-dispositions.json";
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
function changedDispositions(mutate) {
  const data = JSON.parse(read(dispositionFile));
  mutate(data);
  return { read: (file) => file === dispositionFile ? JSON.stringify(data) : read(file) };
}

test("current target contract retains all obligations without execution credit", () => {
  const contract = buildRuntimeContract();
  assert.deepEqual(contract.coverage_boundary, {
    obligations: 6762, pending: 6578, terminal: 184, runtime_execution_credit: false,
  });
  assert.equal(contract.status, "FrozenTargetBoundary");
  assert.match(contract.input_digests.foundation_sha256, /^[0-9a-f]{64}$/u);
});

test("the old subset cannot replace the current obligation denominator", () => {
  const injected = changedDispositions((data) => {
    data.obligations = data.obligations.slice(0, 6215);
    data.summary.obligations = 6215;
    for (const status of ["Pending", "Terminal"])
      data.summary.runtime_status[status] = data.obligations.filter(
        (row) => row.runtime_status === status).length;
  });
  assert.throws(() => buildRuntimeContract(injected), /exact current disposition boundary/u);
});

test("unchanged totals cannot hide duplicate obligation identities", () => {
  assert.throws(() => buildRuntimeContract(changedDispositions((data) => {
    data.obligations[1].obligation_id = data.obligations[0].obligation_id;
  })), /exact current disposition boundary/u);
});

test("status counts must match the rows rather than a claimed summary", () => {
  assert.throws(() => buildRuntimeContract(changedDispositions((data) => {
    data.summary.runtime_status.Pending -= 1;
    data.summary.runtime_status.Terminal += 1;
  })), /exact current disposition boundary/u);
});

test("unknown row or summary statuses cannot receive target coverage", () => {
  for (const mutate of [
    (data) => { data.obligations[0].runtime_status = "CatalogOnly"; },
    (data) => { data.summary.runtime_status.CatalogOnly = 0; },
  ]) {
    assert.throws(() => buildRuntimeContract(changedDispositions(mutate)),
      /exact current disposition boundary/u);
  }
});
