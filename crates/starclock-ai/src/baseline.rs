use starclock_combat::{
    AbilityId, BattleView, Command, DecisionOwner, DecisionPoint, TeamSide, UnitId,
};

const MAX_COMPONENT: i32 = 1_000_000;
const SURVIVAL_TIER: i64 = 200_000_000;

/// Authored command family used only by the reproducible smoke-test scorer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BaselineAbilityClass {
    Basic = 0,
    Skill = 1,
    Interrupt = 2,
    Mandatory = 3,
}

impl BaselineAbilityClass {
    const fn tier(self) -> i64 {
        match self {
            Self::Basic => 20_000_000,
            Self::Skill => 30_000_000,
            Self::Interrupt => 40_000_000,
            Self::Mandatory => 50_000_000,
        }
    }
}

/// Bounded integer score components authored for one ability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BaselineScoreComponents {
    priority: i32,
    survival: i32,
    break_opportunity: i32,
    resource_reserve: i32,
    synergy: i32,
    prevents_immediate_loss: bool,
}

impl BaselineScoreComponents {
    #[must_use]
    pub const fn new(
        priority: i32,
        survival: i32,
        break_opportunity: i32,
        resource_reserve: i32,
        synergy: i32,
        prevents_immediate_loss: bool,
    ) -> Option<Self> {
        if in_range(priority)
            && in_range(survival)
            && in_range(break_opportunity)
            && in_range(resource_reserve)
            && in_range(synergy)
        {
            Some(Self {
                priority,
                survival,
                break_opportunity,
                resource_reserve,
                synergy,
                prevents_immediate_loss,
            })
        } else {
            None
        }
    }
}

const fn in_range(value: i32) -> bool {
    value >= -MAX_COMPONENT && value <= MAX_COMPONENT
}

/// Stable authored hint for one player ability definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BaselineAbilityHint {
    ability: AbilityId,
    class: BaselineAbilityClass,
    components: BaselineScoreComponents,
}

impl BaselineAbilityHint {
    #[must_use]
    pub const fn new(
        ability: AbilityId,
        class: BaselineAbilityClass,
        components: BaselineScoreComponents,
    ) -> Self {
        Self {
            ability,
            class,
            components,
        }
    }

    #[must_use]
    pub const fn ability(self) -> AbilityId {
        self.ability
    }
}

/// Stable resolved target hint for one currently visible battle unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BaselineTargetHint {
    target: UnitId,
    value: i32,
}

impl BaselineTargetHint {
    #[must_use]
    pub const fn new(target: UnitId, value: i32) -> Option<Self> {
        if in_range(value) {
            Some(Self { target, value })
        } else {
            None
        }
    }
}

/// Canonical immutable scorer hints. Input row order cannot affect selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaselineHints {
    abilities: Box<[BaselineAbilityHint]>,
    targets: Box<[BaselineTargetHint]>,
}

impl BaselineHints {
    pub fn new(
        mut abilities: Vec<BaselineAbilityHint>,
        mut targets: Vec<BaselineTargetHint>,
    ) -> Result<Self, BaselineHintError> {
        abilities.sort_by_key(|hint| hint.ability);
        if abilities
            .windows(2)
            .any(|pair| pair[0].ability == pair[1].ability)
        {
            return Err(BaselineHintError::DuplicateAbility);
        }
        targets.sort_by_key(|hint| hint.target);
        if targets
            .windows(2)
            .any(|pair| pair[0].target == pair[1].target)
        {
            return Err(BaselineHintError::DuplicateTarget);
        }
        Ok(Self {
            abilities: abilities.into_boxed_slice(),
            targets: targets.into_boxed_slice(),
        })
    }

    fn ability(&self, id: AbilityId) -> Option<BaselineAbilityHint> {
        self.abilities
            .binary_search_by_key(&id, |hint| hint.ability)
            .ok()
            .map(|index| self.abilities[index])
    }

    fn target(&self, id: UnitId) -> Option<BaselineTargetHint> {
        self.targets
            .binary_search_by_key(&id, |hint| hint.target)
            .ok()
            .map(|index| self.targets[index])
    }
}

/// Auditable integer breakdown for one exact offered command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaselineCommandScore {
    command: Command,
    class_tier: i64,
    survival_tier: i64,
    priority: i32,
    survival: i32,
    break_opportunity: i32,
    target_value: i32,
    resource_reserve: i32,
    synergy: i32,
    total: i64,
}

impl BaselineCommandScore {
    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }
    #[must_use]
    pub const fn total(&self) -> i64 {
        self.total
    }
    #[must_use]
    pub const fn prevents_immediate_loss(&self) -> bool {
        self.survival_tier != 0
    }
    #[must_use]
    pub const fn target_value(&self) -> i32 {
        self.target_value
    }
}

/// Selected exact command plus deterministic diagnostics for every offer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaselineDecision {
    command: Command,
    scores: Box<[BaselineCommandScore]>,
}

impl BaselineDecision {
    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }
    #[must_use]
    pub fn scores(&self) -> &[BaselineCommandScore] {
        &self.scores
    }
}

/// Stateless reproducible player smoke-test controller.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BaselineController;

impl BaselineController {
    pub const REVISION: &'static str = "baseline-battle-controller-v1";

    pub fn decide(
        self,
        view: BattleView<'_>,
        decision: &DecisionPoint,
        hints: &BaselineHints,
    ) -> Result<BaselineDecision, BaselineDecisionError> {
        match decision.owner() {
            DecisionOwner::System | DecisionOwner::Team(TeamSide::Player) => {}
            DecisionOwner::Team(TeamSide::Enemy) => return Err(BaselineDecisionError::WrongOwner),
        }
        let units = view
            .units_by_id()
            .map(|unit| (unit.id(), unit.side()))
            .collect::<Vec<_>>();
        score_offered(decision.legal_commands(), &units, hints)
    }
}

fn score_offered(
    commands: &[Command],
    units: &[(UnitId, TeamSide)],
    hints: &BaselineHints,
) -> Result<BaselineDecision, BaselineDecisionError> {
    if commands.is_empty() {
        return Err(BaselineDecisionError::EmptyOffer);
    }
    let mut commands = commands.to_vec();
    commands.sort_by(Command::canonical_cmp);
    let survival_available = commands.iter().any(|command| {
        command_ability(command)
            .and_then(|ability| hints.ability(ability))
            .is_some_and(|hint| hint.components.prevents_immediate_loss)
    });
    let mut scores = Vec::with_capacity(commands.len());
    for command in commands {
        scores.push(score(command, units, hints, survival_available)?);
    }
    let selected = scores
        .iter()
        .enumerate()
        .max_by(|(left_index, left), (right_index, right)| {
            left.total
                .cmp(&right.total)
                .then_with(|| right_index.cmp(left_index))
        })
        .ok_or(BaselineDecisionError::EmptyOffer)?
        .0;
    Ok(BaselineDecision {
        command: scores[selected].command.clone(),
        scores: scores.into_boxed_slice(),
    })
}

fn score(
    command: Command,
    units: &[(UnitId, TeamSide)],
    hints: &BaselineHints,
    survival_available: bool,
) -> Result<BaselineCommandScore, BaselineDecisionError> {
    let (class_tier, components, target) = match &command {
        Command::StartBattle { .. } => (60_000_000, empty_components(), None),
        Command::PassInterruptWindow { .. } => (10_000_000, empty_components(), None),
        Command::Concede { .. } => (-100_000_000, empty_components(), None),
        Command::UseAbility {
            actor,
            ability,
            primary_target,
            ..
        } => {
            let hint = checked_hint(*actor, *ability, units, hints)?;
            if hint.class == BaselineAbilityClass::Interrupt {
                return Err(BaselineDecisionError::AbilityClassMismatch(*ability));
            }
            (hint.class.tier(), hint.components, *primary_target)
        }
        Command::UseInterrupt {
            actor,
            ability,
            primary_target,
            ..
        } => {
            let hint = checked_hint(*actor, *ability, units, hints)?;
            if !matches!(
                hint.class,
                BaselineAbilityClass::Interrupt | BaselineAbilityClass::Mandatory
            ) {
                return Err(BaselineDecisionError::AbilityClassMismatch(*ability));
            }
            (hint.class.tier(), hint.components, *primary_target)
        }
    };
    let target_value = match target {
        None => 0,
        Some(target) => {
            if !units.iter().any(|(id, _)| *id == target) {
                return Err(BaselineDecisionError::MissingTarget(target));
            }
            hints
                .target(target)
                .ok_or(BaselineDecisionError::MissingTargetHint(target))?
                .value
        }
    };
    let survival_tier = if survival_available && components.prevents_immediate_loss {
        SURVIVAL_TIER
    } else {
        0
    };
    let total = class_tier
        + survival_tier
        + i64::from(components.priority)
        + i64::from(components.survival)
        + i64::from(components.break_opportunity)
        + i64::from(target_value)
        + i64::from(components.resource_reserve)
        + i64::from(components.synergy);
    Ok(BaselineCommandScore {
        command,
        class_tier,
        survival_tier,
        priority: components.priority,
        survival: components.survival,
        break_opportunity: components.break_opportunity,
        target_value,
        resource_reserve: components.resource_reserve,
        synergy: components.synergy,
        total,
    })
}

fn checked_hint(
    actor: UnitId,
    ability: AbilityId,
    units: &[(UnitId, TeamSide)],
    hints: &BaselineHints,
) -> Result<BaselineAbilityHint, BaselineDecisionError> {
    let side = units
        .iter()
        .find(|(id, _)| *id == actor)
        .map(|(_, side)| *side)
        .ok_or(BaselineDecisionError::MissingActor(actor))?;
    if side != TeamSide::Player {
        return Err(BaselineDecisionError::WrongActorSide(actor));
    }
    hints
        .ability(ability)
        .ok_or(BaselineDecisionError::MissingAbilityHint(ability))
}

const fn empty_components() -> BaselineScoreComponents {
    BaselineScoreComponents {
        priority: 0,
        survival: 0,
        break_opportunity: 0,
        resource_reserve: 0,
        synergy: 0,
        prevents_immediate_loss: false,
    }
}

fn command_ability(command: &Command) -> Option<AbilityId> {
    match command {
        Command::UseAbility { ability, .. } | Command::UseInterrupt { ability, .. } => {
            Some(*ability)
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselineHintError {
    DuplicateAbility,
    DuplicateTarget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselineDecisionError {
    WrongOwner,
    EmptyOffer,
    MissingActor(UnitId),
    WrongActorSide(UnitId),
    MissingTarget(UnitId),
    MissingAbilityHint(AbilityId),
    MissingTargetHint(UnitId),
    AbilityClassMismatch(AbilityId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use starclock_combat::{DecisionId, UnitId};

    fn runtime(raw: u64) -> UnitId {
        UnitId::try_from(raw).unwrap()
    }
    fn decision(raw: u64) -> DecisionId {
        DecisionId::try_from(raw).unwrap()
    }
    fn ability(raw: u32) -> AbilityId {
        AbilityId::new(raw).unwrap()
    }

    fn command(ability_id: u32, target: u64) -> Command {
        Command::UseAbility {
            decision: decision(1),
            actor: runtime(1),
            ability: ability(ability_id),
            primary_target: Some(runtime(target)),
        }
    }

    fn components(prevents_loss: bool, synergy: i32) -> BaselineScoreComponents {
        BaselineScoreComponents::new(0, 0, 0, 0, synergy, prevents_loss).unwrap()
    }

    #[test]
    fn survival_hint_wins_and_input_order_cannot_change_diagnostics() {
        let basic = BaselineAbilityHint::new(
            ability(1),
            BaselineAbilityClass::Basic,
            components(false, 900_000),
        );
        let sustain =
            BaselineAbilityHint::new(ability(2), BaselineAbilityClass::Skill, components(true, 0));
        let target = BaselineTargetHint::new(runtime(2), 5).unwrap();
        let left_hints = BaselineHints::new(vec![basic, sustain], vec![target]).unwrap();
        let right_hints = BaselineHints::new(vec![sustain, basic], vec![target]).unwrap();
        let units = [
            (runtime(1), TeamSide::Player),
            (runtime(2), TeamSide::Enemy),
        ];
        let left = score_offered(&[command(1, 2), command(2, 2)], &units, &left_hints).unwrap();
        let right = score_offered(&[command(2, 2), command(1, 2)], &units, &right_hints).unwrap();
        assert_eq!(left, right);
        assert_eq!(left.command(), &command(2, 2));
        assert!(
            left.scores()
                .iter()
                .any(BaselineCommandScore::prevents_immediate_loss)
        );
    }

    #[test]
    fn equal_scores_break_by_canonical_command_and_target_identity() {
        let hint = BaselineAbilityHint::new(
            ability(1),
            BaselineAbilityClass::Basic,
            components(false, 0),
        );
        let hints = BaselineHints::new(
            vec![hint],
            vec![
                BaselineTargetHint::new(runtime(3), 0).unwrap(),
                BaselineTargetHint::new(runtime(2), 0).unwrap(),
            ],
        )
        .unwrap();
        let units = [
            (runtime(1), TeamSide::Player),
            (runtime(2), TeamSide::Enemy),
            (runtime(3), TeamSide::Enemy),
        ];
        let selected = score_offered(&[command(1, 3), command(1, 2)], &units, &hints).unwrap();
        assert_eq!(selected.command(), &command(1, 2));
    }

    #[test]
    fn missing_authored_hint_rejects_instead_of_inventing_a_score() {
        let hints = BaselineHints::new(
            vec![],
            vec![BaselineTargetHint::new(runtime(2), 0).unwrap()],
        )
        .unwrap();
        let units = [
            (runtime(1), TeamSide::Player),
            (runtime(2), TeamSide::Enemy),
        ];
        assert_eq!(
            score_offered(&[command(1, 2)], &units, &hints).unwrap_err(),
            BaselineDecisionError::MissingAbilityHint(ability(1))
        );
    }
}
