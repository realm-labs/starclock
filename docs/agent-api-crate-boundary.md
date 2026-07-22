# Protocol-neutral agent API crate boundary

`G02-P1-B1` introduces `starclock-agent-api` as an independent workspace crate.
Its façade is intentionally small and responsibility-split before behavior is
implemented:

- `schema` owns revisioned exact agent values;
- `observation` owns bounded visibility-controlled projections;
- `action` owns public offered-action summaries and private exact-command
  bindings;
- `session` owns authoritative ephemeral session/registry composition;
- `error` owns stable protocol-neutral failures.

The crate currently has zero dependencies. Later batches may add only direct
Goal 01 libraries needed by implemented responsibilities plus the reviewed
serialization dependency used for deterministic schema conversion. MCP,
JSON-RPC, HTTP, async runtimes, authentication, model-provider and storage
dependencies are forbidden here. `starclock-mcp` is the future outward adapter
and no deterministic/domain crate may depend on either new layer.

Public agent values are owned DTOs rather than aliases for combat stores or MCP
models. Mutable battle access remains session-private and every commit continues
through Goal 01 apply methods.
