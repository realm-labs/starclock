# Omniscient debug policy

`PlayerVisible` remains the only default battle projection. The trusted debug
path has a distinct `AgentDebugProjection` type and cannot be selected through
a visibility enum alone: callers must explicitly acknowledge an
`OmniscientDebugCapability`. A missing capability returns
`UnauthorizedDebug` before projection.

Every successful debug value serializes both
`"visibility_policy":"omniscient_debug"` and
`"debug_authorized":true`. The player battle type contains neither marker, and
negative serialization tests protect that structural separation.

The capability is an explicit in-process trust acknowledgement, not a bearer
credential and not proof of remote authorization. The MCP adapter may construct
it only after the independent `starclock:debug:omniscient` scope check defined
by the threat model. It must never be inferred from a requested policy string.

The frozen version-one schema does not define an extra hidden-state payload, so
debug mode currently returns the same bounded battle facts as `PlayerVisible`
under its separate marked envelope. This conservative implementation avoids
silently extending the frozen schema or serializing internal AI, rule, command,
resolver, seed or RNG state. Any richer omniscient payload requires a new schema
revision with its own visibility review and bounds.
