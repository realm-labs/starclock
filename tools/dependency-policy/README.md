# Dependency policy verification

The verifier compares the complete resolved registry graph with the reviewed
machine-readable policy, checks active compiler/tool versions, and ensures that
`fixnum` is named only inside the private combat numeric scalar backend.

```sh
node tools/dependency-policy/verify.mjs
```

It intentionally fails when a transitive package appears without review or when
the local tool versions differ from the committed pins.
