# Goal 01 research-register tools

These tools generate and verify the Phase 0 research register. The register
turns every known V1a, shared-Elation and Himeko Nova mechanism ambiguity into a
named case with an owner, source/evidence binding and reproducible observation
or golden-fixture specification.

```sh
node tools/goal-research/generate.mjs
node tools/goal-research/generate.mjs --check
node tools/goal-research/verify.mjs
```

The generated cases intentionally remain `Researching`. A later owning batch may
change a case only after it binds the required observation and makes the named
golden executable through the production Excel/Sora-to-domain boundary. The
register is evidence and test planning, not production content staging.
