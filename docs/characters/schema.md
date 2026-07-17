# Character Profile Schema

## Normative boundary

Each compact profile describes an E0 character with its ordinary Trace behavior. The executable character definition additionally contains level/promotion data, every battle-relevant Trace and Technique, and six ordered Eidolon patches. A profile explains behavior, not final balance values.

Character balance data is authored in Excel and compiled by Sora according to [Excel and Sora configuration pipeline](../07-configuration-pipeline.md). The conceptual types in this document are combat-domain types; generated Sora rows must be converted into them at the `combat-data` boundary.

Use these four layers:

```text
CharacterDefinition   immutable authored metadata and leveled coefficients
CharacterState        mutable per-battle resources, marks, modes, and counters
AbilityProgram        validated commands/effects for one ability or passive
RuleExtension         reusable scheduler/damage/lifecycle behavior
EidolonPatch          ordered changes to abilities, rules, state caps, and modifiers
```

If an ability needs a new behavior, add a reusable command, selector, trigger, or modifier. Never add `if character_id == ...` to global combat code.

## Profile fields

| Field | Meaning |
|---|---|
| Identity | Combat-form name, element, path, and release status. |
| Core loop | The smallest description of how actions and resources feed one another. |
| Ability contract | Semantics of Basic, Skill, Ultimate, Talent, enhanced actions, summons, and memosprites. |
| State | Character-owned resource, target mark, team field/zone, stance, counter, or linked actor. |
| Required primitives | Engine capabilities beyond ordinary damage/heal/buff/debuff commands. |
| Open data | Values that must remain in a versioned data pack or are not yet public. |

## Shared vocabulary

- **Action**: a normal turn, Ultimate interrupt, follow-up attack, counter, extra action, summon action, or memosprite action.
- **Extra turn**: a scheduler entry that does not reset the actor's normal Action Gauge unless the specific rule says so.
- **Follow-up attack**: a triggered attack with its own action envelope and authored retarget policy.
- **Additional damage**: damage emitted inside another action; it is not a new action and does not independently trigger “used an attack” hooks.
- **Zone/field**: a team- or battlefield-owned effect with explicit owner, duration clock, replacement policy, and teardown event.
- **Mark**: state attached to a target, optionally unique per source.
- **Linked actor**: a summon, memosprite, companion, or countdown entry with an owner and independent timeline rules.
- **Transformation**: an atomic ability-set/state replacement with entry and exit hooks.
- **Tally**: accumulated damage, healing, HP loss, Skill Points, Toughness damage, or actions, recorded with a clear reset boundary.
- **Joint attack**: one trigger that queues contributions from two actors; ownership of damage and trigger credit must remain explicit.
- **True damage**: authored damage that bypasses the ordinary formula stages specified by the effect. It must declare whether it can CRIT and which multipliers still apply.
- **Elation damage**: a distinct damage category used by the Elation subsystem; do not alias it to ordinary additional damage.

## Required data representation

At minimum, an ability definition should be able to express:

```rust
pub struct AbilityDefinition {
    pub id: AbilityId,
    pub kind: AbilityKind,
    pub cost: Vec<ResourceDelta>,
    pub target: TargetProgram,
    pub effects: Vec<EffectProgram>,
    pub hit_plan: HitPlan,
    pub tags: AbilityTags,
    pub retarget: RetargetPolicy,
}
```

`AbilityTags` must distinguish Basic, Skill, Ultimate, follow-up, counter, DoT, Break, Super Break, summon, memosprite, additional, joint, and Elation damage. Several character kits inspect these categories.

## Timing contract

Every trigger records both a condition and a timing point. Useful points include:

```text
BattleStarted
BeforeAction / ActionStarted / AfterAction
BeforeAbility / AbilityResolved
BeforeHit / AfterHit / TargetDefeated
HpChanged / ShieldChanged / EnergyChanged / SkillPointsChanged
ToughnessChanged / WeaknessBroken / BreakRecovered
EffectApplied / EffectRemoved / UnitEntered / UnitExited
TurnStarted / TurnEnded / WaveStarted / WaveEnded
```

Counters must say whether they react once per ability, target, hit, action, or event batch. A multi-hit bounce attack must not accidentally trigger a once-per-action passive once per hit.

## Versioning rule

The singleton Sora config manifest carries `game_version`, `rules_revision`, `data_revision`, and `sora_cli_version`; the exported bundle receives a SHA-256 digest. An announced profile may use optional Excel fields, but simulation startup must reject an incomplete or disabled definition unless a test explicitly enables provisional data. This prevents guessed values for Rin Tohsaka or Gilgamesh from silently becoming production rules.
