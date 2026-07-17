# Modifier and snapshot runtime boundary

`G01-P4-B2` implements the stat-query subset of the normative modifier pipeline
in `starclock-combat`. Immutable definitions and runtime instances stay separate;
the resolver accepts Starclock domain values, never generated Sora rows or
fixed-point implementation types.

Stat queries carry subject, stat, purpose and explicit filter context. The
resolver evaluates BaseAdd, PercentOfBase, Flat, FinalAdd and FinalMultiply in
that order, partitions applicable instances by stacking group, applies the
group's declared policy, then applies named-stage bounds. Every comparison ends
with source, insertion sequence and typed instance identities. Context caches
are non-authoritative and disabling them produces identical results.

Rule IR owns arithmetic. Its typed `QueryStat` leaf calls a read-only bridge,
so nested stat expressions use the same checked expression evaluator as other
rules. Re-entering `(subject, stat, purpose)` produces a stable ordered
`StatQueryCycle` path. No character identity is inspected by the resolver.

Snapshots store values. Full boundary policies consume a captured expression
value; source/target partial policies and ExplicitFields consume only their
ordered captured stat fields and leave other inputs dynamic.
`RecomputeOnStackChange` replaces its captured value only at the explicit stack
mutation boundary. Missing required captures fault instead of silently becoming
dynamic.

The dedicated Asta V1a probe is isolated under
`config/probes/v1a/asta-modifier`. Its reviewed rows are materialized into 80
production-schema `.xlsx` workbooks, exported twice by pinned Sora 0.3.0,
decoded by the production generated reader, and lowered into Rule IR plus the
modifier registry. The exact Astrometry level-10 relation is an explicit
integer-to-scalar conversion followed by `Charging * 0.14`. The executable
fixture proves one aura instance at 0/1/5/3 stacks and separately proves
distinct-target/Fire-weak hit-time credit. All four identities are disabled
`ProjectFixture` rows, so the production loader rejects the bundle and coverage
credit remains zero.
