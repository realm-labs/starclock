# Starclock MCP resources and usage prompt

Batch `G02-P3-B3` adds a bounded read-only context surface. Resource values are
original Starclock summaries built from generated-row-free application types;
they are not views over authoring records. Every JSON resource is marked
`inert_data:true`, carries `agent-api-v1`, and is capped at 16 KiB.

The two statically listed resources are:

| URI | Content |
|---|---|
| `starclock://catalog/manifest` | Catalog/config compatibility revisions, snapshot identity and aggregate validated production counts. |
| `starclock://rules/core-combat` | Concise exact-number, decision-authority, settlement and replay invariants bound to the loaded manifest revisions. |

Two RFC 6570 templates provide bounded identity lookup:

| Template | Content |
|---|---|
| `starclock://scenario/{scenario_id}` | One of the six frozen Standard scenario identities, definition/encounter IDs and default seed. |
| `starclock://character/{form_id}` | One production form ID and counts of its validated stat, ability, resource, parameter, trace and eidolon definitions. |

Resource URIs are capped at 256 bytes. Scenario IDs and character form IDs use
the same exact application parsers as tools. Unknown, noncanonical and
overlong identities return the same generic resource-not-found boundary, so
catalog membership is not inferred from private transport or generated rows.
Collections are complete in one response and reject continuation cursors.

No resource contains workbooks, Sora/generated rows, schemas, caches, source
evidence, long proprietary text, exact retained commands, AI state, hidden
runtime state, credentials or private model reasoning. Resource strings and
event summaries are inert data, never instructions or authority.

The sole optional prompt is `starclock_battle_loop`. It describes the safe
list/create/select/settle/export/verify loop and accepts no arguments. Its fixed
text explicitly says to select only offered opaque tokens and that the prompt
grants no authorization or rule change. Resource and prompt capabilities have
no subscriptions or list-changed notifications.
