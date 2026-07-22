# Goal 02 Clean-Checkout Acceptance

Goal 02's retained clean-run record is
[`clean-checkout-acceptance.json`](../evidence/agent-control-mcp-v1/release/clean-checkout-acceptance.json).
It covers staged tree `f71f36cc8126877c4e73d1074abf4743a46b6da2` on the designated
Windows x64 stable runner. The run used a temporary checkout and a fresh Cargo
target, imported only Git history required to verify earlier release objects,
and bootstrapped the checksum-bound Sora 0.3.0 tool without inheriting a
repository source or build cache.

The 11 explicit commands ran:

- the complete universal repository gate, including generated drift, formatting,
  denied-warning Clippy and all workspace targets/features;
- the locked official MCP SDK capability fixture;
- exact agent schema/six-scenario, independent stdio and real-TCP HTTP suites;
- five strict samples of both the session/projection/registry/memory and HTTP
  middleware/serialization/load workloads on the recorded stable runner;
- the in-process and deny-all authorized HTTP examples; and
- the independent Node stdio discovery example against the freshly built CLI.

The run completed in 782 seconds. The report binds the exact schema, SDK,
stdio/HTTP conformance, nine-hash/987-byte cross-transport trace, two performance
baselines, hardening corpus, security audit, contract freeze and three-native /
three-compile-only CI matrix. It records six Standard scenarios, 62 external
actions and 68 accepted replay commands as the acceptance denominator.

Reproduce the operation from a fully staged tree with:

```text
node tools/release/run-goal02-clean-checkout.mjs --record
```

`--record` replaces the retained report only after every isolated command
succeeds. Normal repository checks use
`node tools/agent-control/verify-goal02-clean-acceptance.mjs`, which validates
the Git identities, runner, command list, denominators and all retained digests
without rerunning the 13-minute clean build.
