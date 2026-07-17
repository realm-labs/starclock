# Dependency policy verification

The verifier compares the complete resolved registry graph with the reviewed
machine-readable policy, checks active compiler/tool versions, and ensures that
`fixnum` and `rand` remain confined to their reviewed private combat numeric
and RNG backends; `sha2` remains confined to private RNG derivation and
canonical battle-state codec owners.

```sh
node tools/dependency-policy/verify.mjs
```

It intentionally fails when a transitive package appears without review or when
the local tool versions differ from the committed pins.
