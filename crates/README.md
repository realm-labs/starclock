# Workspace crates

The repository root is a virtual Cargo workspace. Each crate owns one direction
in the data-to-battle pipeline:

```text
combat <- build
combat <- activity
combat/activity <- rules, replay
combat <- ai
combat/activity <- mode-standard
combat/build/activity/rules/mode-standard <- data
all required crates <- cli
```

`starclock-combat` has no dependencies. Build selections and equipment never
enter battle state; activities hand resolved combat input to battles; Sora rows
remain inside the data boundary; engines and future mode crates are absent.

`tools/workspace/verify-dependencies.mjs` checks the exact graph. Its verifier is
also invoked by the `starclock-cli` integration test so `cargo test --workspace`
enforces the boundary.
