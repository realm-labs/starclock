//! Explicit pre-battle technique input lowered into ordinary combat Rule IR.

use starclock_activity::{
    ActivityOptionId, ParticipantId, TechniqueEngagement, TechniqueOptionDefinition,
};
use starclock_combat::{
    AbilityId, ProgramId, RuleBundleId, RuleId, SelectorId, SourceDefinitionId, TriggerId,
    catalog::{
        CombatCatalog,
        action::{ReactionBoundary, TargetPattern, TargetRelation},
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

use crate::digest::Encoder;

const RULE_ID: u32 = 0x7640_0001;
const BUNDLE_ID: u32 = 0x7640_0002;
const PROGRAM_ID: u32 = 0x7640_0003;
const ACTOR_SELECTOR_ID: u32 = 0x7640_0004;
const TARGET_SELECTOR_ID: u32 = 0x7640_0005;
const TRIGGER_ID: u32 = 0x7640_0006;
const SOURCE_ID: u32 = 0x7640_0007;

/// One explicitly authored technique offered at a Standard Universe encounter.
///
/// The ability identity is supplied by the build/content layer. Combat never
/// infers a technique from a generic action kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UniverseBattleTechniqueDefinition {
    option: ActivityOptionId,
    participant: ParticipantId,
    ability: AbilityId,
    point_cost: u16,
    engagement: TechniqueEngagement,
}

impl UniverseBattleTechniqueDefinition {
    #[must_use]
    pub const fn new(
        option: ActivityOptionId,
        participant: ParticipantId,
        ability: AbilityId,
        point_cost: u16,
        engagement: TechniqueEngagement,
    ) -> Option<Self> {
        if point_cost == 0 {
            None
        } else {
            Some(Self {
                option,
                participant,
                ability,
                point_cost,
                engagement,
            })
        }
    }

    #[must_use]
    pub const fn option(self) -> ActivityOptionId {
        self.option
    }

    #[must_use]
    pub const fn participant(self) -> ParticipantId {
        self.participant
    }

    #[must_use]
    pub const fn ability(self) -> AbilityId {
        self.ability
    }

    #[must_use]
    pub const fn point_cost(self) -> u16 {
        self.point_cost
    }

    #[must_use]
    pub const fn engagement(self) -> TechniqueEngagement {
        self.engagement
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CompiledUniverseBattleTechnique {
    definition: UniverseBattleTechniqueDefinition,
    actor_selector: SelectorDefinition,
    target_selector: SelectorDefinition,
    program: ProgramDefinition,
    rule: RuleDefinition,
    bundle: RuleBundle,
    digest: [u8; 32],
}

impl CompiledUniverseBattleTechnique {
    pub(crate) fn compile(
        catalog: &CombatCatalog,
        definition: UniverseBattleTechniqueDefinition,
    ) -> Result<Self, UniverseBattleTechniqueError> {
        let ability = catalog
            .ability(definition.ability)
            .and_then(|ability| ability.action().map(|_| ability))
            .ok_or(UniverseBattleTechniqueError::MissingAbility)?;
        let authored_selector = catalog
            .selector(ability.selector())
            .and_then(|selector| selector.unit_targets())
            .ok_or(UniverseBattleTechniqueError::MissingTargetSelector)?;
        let actor_selector = SelectorDefinition::new(
            SelectorId::new(ACTOR_SELECTOR_ID).expect("reserved selector ID is non-zero"),
        )
        .with_rule_units(selector(
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
            TargetPattern::Single => (RuleSelectorChoice::First, 1),
            TargetPattern::All => (RuleSelectorChoice::All, 16),
            TargetPattern::Blast => return Err(UniverseBattleTechniqueError::UnsupportedBlast),
        };
        let target_selector = SelectorDefinition::new(
            SelectorId::new(TARGET_SELECTOR_ID).expect("reserved selector ID is non-zero"),
        )
        .with_rule_units(selector(origin, side, choice, maximum)?);
        let program_id =
            ProgramId::new(PROGRAM_ID).expect("reserved technique program ID is non-zero");
        let actor_id = actor_selector.id();
        let target_id = target_selector.id();
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
                ability: definition.ability,
                priority: ReactionPriority::new(-200),
                forced_use: true,
                boundary: ReactionBoundary::BeforeTimeline,
                owner: RuleActionOwner::Actor,
                payment: Some(RuleActionPaymentPolicy::Suppressed),
            },
        )]);
        let digest = technique_digest(definition);
        let source = RuleSource::new(
            SourceDefinitionId::new(SOURCE_ID).expect("reserved source ID is non-zero"),
            SourceClass::Mode,
            Vec::new(),
            digest,
        );
        let rule_id = RuleId::new(RULE_ID).expect("reserved rule ID is non-zero");
        let rule = RuleDefinition::new(rule_id, vec![program_id], vec![actor_id, target_id])
            .with_runtime(BattleRuleDefinition::new(
                source,
                Vec::new(),
                vec![TriggerDef {
                    id: TriggerId::new(TRIGGER_ID).expect("reserved trigger ID is non-zero"),
                    event: RuleEventKind::Battle,
                    event_point: RuleEventPoint::BattleStarted,
                    phase: TriggerPhase::AfterEvent,
                    filter: EventFilter::default(),
                    condition: ConditionExpr::Literal(true),
                    once_scope: OnceScope::Battle,
                    priority: ReactionPriority::new(-200),
                    program: program_id,
                }],
                None,
            ));
        let bundle = RuleBundle::new(
            RuleBundleId::new(BUNDLE_ID).expect("reserved bundle ID is non-zero"),
            vec![rule_id],
        );
        Ok(Self {
            definition,
            actor_selector,
            target_selector,
            program,
            rule,
            bundle,
            digest,
        })
    }

    pub(crate) const fn definition(&self) -> UniverseBattleTechniqueDefinition {
        self.definition
    }

    pub(crate) const fn actor_selector(&self) -> &SelectorDefinition {
        &self.actor_selector
    }

    pub(crate) const fn target_selector(&self) -> &SelectorDefinition {
        &self.target_selector
    }

    pub(crate) const fn program(&self) -> &ProgramDefinition {
        &self.program
    }

    pub(crate) const fn rule(&self) -> &RuleDefinition {
        &self.rule
    }

    pub(crate) const fn bundle(&self) -> &RuleBundle {
        &self.bundle
    }

    pub(crate) const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub(crate) fn activity_definition(&self) -> TechniqueOptionDefinition {
        TechniqueOptionDefinition::new(
            self.definition.option,
            self.definition.participant,
            self.definition.point_cost,
            self.definition.engagement,
        )
        .expect("public technique definition rejects zero point cost")
    }
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    choice: RuleSelectorChoice,
    maximum: u16,
) -> Result<RuleUnitSelector, UniverseBattleTechniqueError> {
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
    .ok_or(UniverseBattleTechniqueError::InvalidSelector)
}

fn technique_digest(definition: UniverseBattleTechniqueDefinition) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.standard-universe.battle-technique.v1");
    encoder.u64(definition.option.get());
    encoder.u32(definition.participant.get());
    encoder.u32(definition.ability.get());
    encoder.u32(u32::from(definition.point_cost));
    encoder.u8(definition.engagement as u8);
    encoder.finish()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniverseBattleTechniqueError {
    MissingAbility,
    MissingTargetSelector,
    UnsupportedBlast,
    InvalidSelector,
}

impl core::fmt::Display for UniverseBattleTechniqueError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Standard Universe technique error: {self:?}")
    }
}

impl std::error::Error for UniverseBattleTechniqueError {}
