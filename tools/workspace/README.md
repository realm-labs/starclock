# Workspace boundary verification

`verify-dependencies.mjs` reads `cargo metadata` and rejects missing/extra
workspace members, misplaced manifests, external dependencies introduced before
their policy review, or any local edge outside the reviewed declarative graph.

```sh
node tools/workspace/verify-dependencies.mjs
```

The `starclock-cli` integration test invokes the same verifier, so the boundary
is enforced by ordinary workspace tests as well as direct repository checks.
