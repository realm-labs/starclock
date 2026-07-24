# Goal 05 Integration Probes

The machine-readable baseline is
`policy/goal05-integration-probes.json`; validate it with:

```text
node tools/goal05/verify-integration-probes.mjs
```

The baseline intentionally records observable Goal 04 integration debt rather
than treating it as a permanent acceptance rule:

- CLI and agent workflows use the explicitly labeled reference Won projection;
- nine Path effect entry points plus Occurrence/service plan evaluators are not
  automatically connected to the complete-run mutation path;
- 579 source domains across 37 acyclic templates expand to seven physical
  Activity nodes each and 4,058 nodes/5,993 edges overall;
- the focused Goal 05 loop has a hard three-minute wall budget.

Each owning Goal 05 batch replaces its corresponding baseline assertion with a
positive end-to-end test. The final release must satisfy
`terminal_expectations`; it must not keep this debt verifier as evidence that
the integration is complete.
