# Starclock agent cross-platform evidence

Goal 02 added two suites to the pinned CI matrix. `agent-schema` executes the
exact `agent-api-v1` schema/golden bundle and canonical value properties.
`agent-trace` executes the shared nine-hash, 987-byte replay artifact through
the in-process session API, stdio MCP and authorized real-TCP HTTP MCP.

The native matrix is Windows x64 (`windows-2025`), Linux x64
(`ubuntu-24.04`) and macOS ARM64 (`macos-15`). Each native job runs
`node tools/repository-check/run.mjs --full` exactly once. That gate owns all
current tests, including the Goal 02 schema and transport coverage; CI must not
replay completed Goal runners after it. A successful per-run artifact may mark
both suites `executed` for the exact checked commit and hosted image.

Windows ARM64, Linux ARM64 and macOS x64 are deliberately compile-only. Their
jobs compile all workspace targets and test sources but run no target binary.
Their evidence says `compiled-not-executed`; it is not schema-byte, replay,
hash, numeric or runtime compatibility evidence.

The committed
[`ci-golden-matrix.json`](../evidence/core-combat-v1/hardening/ci-golden-matrix.json)
binds the workflow contract, normalized suite-source hashes, schema digest and
transport artifact digest. Hosted run IDs remain in the 30-day CI artifacts
rather than being fabricated in the repository. Verify the static evidence
boundary with:

```text
node tools/ci/verify-workflow.mjs
node tools/ci/verify-golden-matrix.mjs
node tools/agent-control/verify-agent-ci-matrix.mjs
```
