//! Currency Wars automatic Techniques lowered into ordinary combat Rule IR.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use starclock_combat::{
    AbilityId, Energy, ProgramId, ResolvedCombatantSpec, RuleBundleId, RuleId, SelectorId,
    SourceDefinitionId, TriggerId,
    catalog::{
        CombatCatalog,
        action::{ReactionBoundary, TargetPattern, TargetRelation},
        builder::CombatCatalogBuilder,
        definition::{ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleActionOwner, RuleActionPaymentPolicy, RuleEventKind, RuleEventPoint,
        RuleOperationTemplate, RuleSource, SourceClass, TriggerDef, TriggerPhase,
    },
};

use crate::{
    CurrencyWarsContributionSnapshot, CurrencyWarsPositionKind, CurrencyWarsRoleId,
    battle_assembly::{
        CurrencyWarsBattleAssemblyError, combatant_overlay::attach_rule_bundle, debug_error, error,
    },
};

const DEFINITION_BASE: u32 = 0x7d50_0000;
const DEFINITIONS_PER_TECHNIQUE: u32 = 8;

pub(super) fn install(
    builder: &mut CombatCatalogBuilder,
    catalog: &CombatCatalog,
    snapshot: &CurrencyWarsContributionSnapshot,
) -> Result<BTreeMap<CurrencyWarsRoleId, ResolvedCombatantSpec>, CurrencyWarsBattleAssemblyError> {
    let mut combatants = snapshot
        .roles
        .iter()
        .filter(|role| role.position.kind() == CurrencyWarsPositionKind::Front)
        .map(|role| (role.role.id, role.combatant.clone()))
        .collect::<BTreeMap<_, _>>();
    for resource in &snapshot.battle_overrides.special_resources {
        let role = resource.role_state.role();
        let combatant = combatants
            .get(&role)
            .ok_or_else(|| error("Currency Wars special-resource role is not in the front row"))?;
        let maximum = resource
            .maximum
            .scalar()
            .map_err(debug_error)
            .and_then(|value| Energy::from_scaled(value.scaled()).map_err(debug_error))?;
        let current = combatant.current_energy().min(maximum);
        combatants.insert(
            role,
            combatant
                .clone()
                .with_energy(current, maximum)
                .map_err(debug_error)?,
        );
    }
    for (index, technique) in snapshot
        .battle_overrides
        .automatic_techniques
        .iter()
        .enumerate()
    {
        let role = technique.role_state.role();
        let combatant = combatants.get(&role).ok_or_else(|| {
            error("Currency Wars automatic Technique role is not in the front row")
        })?;
        if combatant
            .abilities()
            .binary_search(&technique.ability)
            .is_err()
        {
            return Err(error(
                "Currency Wars automatic Technique is not in the resolved combatant",
            ));
        }
        let compiled = compile(catalog, technique.ability, index)?;
        builder.add_selector(compiled.actor_selector);
        builder.add_selector(compiled.target_selector);
        builder.add_program(compiled.program);
        builder.add_rule(compiled.rule);
        builder.add_rule_bundle(compiled.bundle);
        let replacement = attach_rule_bundle(
            combatant,
            compiled.bundle_id,
            b"starclock.currency-wars.automatic-technique-combatant.v1",
            compiled.digest,
        )?;
        combatants.insert(role, replacement);
    }
    Ok(combatants)
}

struct CompiledTechnique {
    actor_selector: SelectorDefinition,
    target_selector: SelectorDefinition,
    program: ProgramDefinition,
    rule: RuleDefinition,
    bundle: RuleBundle,
    bundle_id: RuleBundleId,
    digest: [u8; 32],
}

fn compile(
    catalog: &CombatCatalog,
    ability_id: AbilityId,
    index: usize,
) -> Result<CompiledTechnique, CurrencyWarsBattleAssemblyError> {
    let ability = catalog
        .ability(ability_id)
        .and_then(|ability| ability.action().map(|_| ability))
        .ok_or_else(|| error("Currency Wars automatic Technique ability is missing"))?;
    let authored_selector = catalog
        .selector(ability.selector())
        .and_then(|selector| selector.unit_targets())
        .ok_or_else(|| error("Currency Wars automatic Technique selector is missing"))?;
    let base = DEFINITION_BASE
        .checked_add(
            u32::try_from(index)
                .map_err(debug_error)?
                .checked_mul(DEFINITIONS_PER_TECHNIQUE)
                .ok_or_else(|| error("Currency Wars automatic Technique ID overflow"))?,
        )
        .ok_or_else(|| error("Currency Wars automatic Technique ID overflow"))?;
    let actor_id = selector_id(base, 1)?;
    let target_id = selector_id(base, 2)?;
    let program_id = ProgramId::new(id(base, 3)?)
        .ok_or_else(|| error("Currency Wars automatic Technique program ID is invalid"))?;
    let rule_id = RuleId::new(id(base, 4)?)
        .ok_or_else(|| error("Currency Wars automatic Technique rule ID is invalid"))?;
    let bundle_id = RuleBundleId::new(id(base, 5)?)
        .ok_or_else(|| error("Currency Wars automatic Technique bundle ID is invalid"))?;
    let trigger_id = TriggerId::new(id(base, 6)?)
        .ok_or_else(|| error("Currency Wars automatic Technique trigger ID is invalid"))?;
    let source_id = SourceDefinitionId::new(id(base, 7)?)
        .ok_or_else(|| error("Currency Wars automatic Technique source ID is invalid"))?;
    let actor_selector = SelectorDefinition::new(actor_id).with_rule_units(selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorChoice::First,
        1,
    )?);
    let (origin, side) = match authored_selector.relation() {
        TargetRelation::SelfUnit => (RuleSelectorOrigin::Owner, RuleSelectorSide::Same),
        TargetRelation::Allied => (RuleSelectorOrigin::Team, RuleSelectorSide::Same),
        TargetRelation::Opposing => (RuleSelectorOrigin::Encounter, RuleSelectorSide::Opposing),
    };
    let (choice, maximum) = match authored_selector.pattern() {
        TargetPattern::Single | TargetPattern::Blast => (RuleSelectorChoice::First, 1),
        TargetPattern::All => (RuleSelectorChoice::All, 16),
    };
    let target_selector = SelectorDefinition::new(target_id)
        .with_rule_units(selector(origin, side, choice, maximum)?);
    let priority = ReactionPriority::new(
        -200_i16
            .checked_add(i16::try_from(index).map_err(debug_error)?)
            .ok_or_else(|| error("Currency Wars automatic Technique priority overflow"))?,
    );
    let program = ProgramDefinition::new(
        program_id,
        Vec::new(),
        vec![actor_id, target_id],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::QueueAction {
            actor_selector: actor_id,
            target_selector: target_id,
            ability: ability_id,
            priority,
            forced_use: true,
            boundary: ReactionBoundary::BeforeTimeline,
            owner: RuleActionOwner::Actor,
            payment: Some(RuleActionPaymentPolicy::Suppressed),
        },
    )]);
    let digest = technique_digest(ability_id, index);
    let source = RuleSource::new(source_id, SourceClass::Mode, Vec::new(), digest);
    let rule = RuleDefinition::new(rule_id, vec![program_id], vec![actor_id, target_id])
        .with_runtime(BattleRuleDefinition::new(
            source,
            Vec::new(),
            vec![TriggerDef {
                id: trigger_id,
                event: RuleEventKind::Battle,
                event_point: RuleEventPoint::BattleStarted,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter::default(),
                condition: ConditionExpr::Literal(true),
                once_scope: OnceScope::Battle,
                priority,
                program: program_id,
            }],
            None,
        ));
    let bundle = RuleBundle::new(bundle_id, vec![rule_id]);
    Ok(CompiledTechnique {
        actor_selector,
        target_selector,
        program,
        rule,
        bundle,
        bundle_id,
        digest,
    })
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    choice: RuleSelectorChoice,
    maximum: u16,
) -> Result<RuleUnitSelector, CurrencyWarsBattleAssemblyError> {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        1,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        None,
        false,
    )
    .ok_or_else(|| error("Currency Wars automatic Technique selector is invalid"))
}

fn selector_id(base: u32, offset: u32) -> Result<SelectorId, CurrencyWarsBattleAssemblyError> {
    SelectorId::new(id(base, offset)?)
        .ok_or_else(|| error("Currency Wars automatic Technique selector ID is invalid"))
}

fn id(base: u32, offset: u32) -> Result<u32, CurrencyWarsBattleAssemblyError> {
    base.checked_add(offset)
        .ok_or_else(|| error("Currency Wars automatic Technique definition ID overflow"))
}

fn technique_digest(ability: AbilityId, index: usize) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.automatic-technique.v1");
    hash.update(ability.get().to_le_bytes());
    hash.update((index as u64).to_le_bytes());
    hash.finalize().into()
}
