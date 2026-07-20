use core::fmt;

use crate::{catalog::CombatCatalog, id::EnemyDefinitionId};

use super::spec::{BattleSpec, ParticipantSource, TeamSide};

/// Stable category for battle-construction failure against an immutable catalog.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BattleBuildErrorKind {
    /// The selected encounter definition does not exist.
    MissingEncounter,
    /// A resolved combatant form does not exist.
    MissingUnit,
    /// A resolved ability reference does not exist.
    MissingAbility,
    /// No selected ability can lower into the current action envelope.
    NoExecutableAbility,
    /// A resolved rule-bundle reference does not exist.
    MissingRuleBundle,
    /// A resolved modifier reference does not exist.
    MissingModifier,
    /// An encounter enemy source does not exist.
    MissingEnemy,
    /// Player/enemy source kind does not match the formation side.
    InvalidParticipantSource,
    /// Enemy source is not listed by the selected encounter.
    EnemyNotInEncounter,
    /// Enemy source and resolved unit form disagree.
    EnemyFormMismatch,
    /// Enemy source is not authorized for its declared encounter wave.
    EnemyNotInWave,
    /// Every authored encounter wave requires at least one participant.
    MissingWaveParticipant,
}

/// Deterministic catalog/spec composition failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleBuildError {
    kind: BattleBuildErrorKind,
    participant_index: Option<u32>,
    definition_id: Option<u32>,
}

impl BattleBuildError {
    fn new(
        kind: BattleBuildErrorKind,
        participant_index: Option<usize>,
        definition_id: Option<u32>,
    ) -> Self {
        Self {
            kind,
            participant_index: participant_index.and_then(|value| u32::try_from(value).ok()),
            definition_id,
        }
    }

    /// Returns the stable error category.
    #[must_use]
    pub const fn kind(self) -> BattleBuildErrorKind {
        self.kind
    }
    /// Returns the canonical participant index when the failure is participant-owned.
    #[must_use]
    pub const fn participant_index(self) -> Option<u32> {
        self.participant_index
    }
    /// Returns the unresolved/mismatched definition ID when applicable.
    #[must_use]
    pub const fn definition_id(self) -> Option<u32> {
        self.definition_id
    }
}

impl fmt::Display for BattleBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "battle construction failed: {:?}", self.kind)
    }
}

impl std::error::Error for BattleBuildError {}

pub(crate) fn validate(catalog: &CombatCatalog, spec: &BattleSpec) -> Result<(), BattleBuildError> {
    let encounter = catalog.encounter(spec.encounter()).ok_or_else(|| {
        BattleBuildError::new(
            BattleBuildErrorKind::MissingEncounter,
            None,
            Some(spec.encounter().get()),
        )
    })?;
    for (index, participant) in spec.participants().iter().enumerate() {
        let combatant = participant.combatant();
        if catalog.unit(combatant.form()).is_none() {
            return Err(BattleBuildError::new(
                BattleBuildErrorKind::MissingUnit,
                Some(index),
                Some(combatant.form().get()),
            ));
        }
        for ability in combatant.abilities() {
            if catalog.ability(*ability).is_none() {
                return Err(BattleBuildError::new(
                    BattleBuildErrorKind::MissingAbility,
                    Some(index),
                    Some(ability.get()),
                ));
            }
        }
        if !combatant.abilities().iter().any(|ability| {
            catalog
                .ability(*ability)
                .and_then(|definition| definition.action())
                .is_some_and(|action| {
                    action.kind().is_normal_turn()
                        && action.resources().skill_point_cost() == 0
                        && action.resources().energy_cost() == crate::Energy::ZERO
                })
        }) {
            return Err(BattleBuildError::new(
                BattleBuildErrorKind::NoExecutableAbility,
                Some(index),
                None,
            ));
        }
        for bundle in combatant.rule_bundles() {
            if catalog.rule_bundle(*bundle).is_none() {
                return Err(BattleBuildError::new(
                    BattleBuildErrorKind::MissingRuleBundle,
                    Some(index),
                    Some(bundle.get()),
                ));
            }
        }
        for modifier in combatant.modifiers() {
            if catalog.modifier(*modifier).is_none() {
                return Err(BattleBuildError::new(
                    BattleBuildErrorKind::MissingModifier,
                    Some(index),
                    Some(modifier.get()),
                ));
            }
        }
        validate_source(catalog, encounter.enemies(), index, participant)?;
        if participant.side() == TeamSide::Enemy
            && !encounter
                .wave(participant.wave())
                .is_some_and(|wave| match participant.source() {
                    ParticipantSource::EncounterEnemy(enemy) => wave.slots().iter().any(|slot| {
                        slot.enemy() == enemy
                            && slot
                                .formation()
                                .is_none_or(|formation| formation == participant.formation())
                    }),
                    ParticipantSource::Player | ParticipantSource::Linked(_) => false,
                })
        {
            return Err(BattleBuildError::new(
                BattleBuildErrorKind::EnemyNotInWave,
                Some(index),
                match participant.source() {
                    ParticipantSource::Player => None,
                    ParticipantSource::EncounterEnemy(enemy) => Some(enemy.get()),
                    ParticipantSource::Linked(source) => Some(source.get()),
                },
            ));
        }
    }
    for number in 1..=encounter.waves().len() {
        let number = u16::try_from(number).expect("encounter wave count is bounded by u16");
        if !spec.participants().iter().any(|participant| {
            participant.side() == TeamSide::Enemy && participant.wave() == number
        }) {
            return Err(BattleBuildError::new(
                BattleBuildErrorKind::MissingWaveParticipant,
                None,
                Some(u32::from(number)),
            ));
        }
    }
    Ok(())
}

fn validate_source(
    catalog: &CombatCatalog,
    encounter_enemies: &[EnemyDefinitionId],
    index: usize,
    participant: &super::spec::ParticipantSpec,
) -> Result<(), BattleBuildError> {
    match (participant.side(), participant.source()) {
        (TeamSide::Player, ParticipantSource::Player) => Ok(()),
        (TeamSide::Enemy, ParticipantSource::EncounterEnemy(enemy_id)) => {
            let enemy = catalog.enemy(enemy_id).ok_or_else(|| {
                BattleBuildError::new(
                    BattleBuildErrorKind::MissingEnemy,
                    Some(index),
                    Some(enemy_id.get()),
                )
            })?;
            if !encounter_enemies.contains(&enemy_id) {
                return Err(BattleBuildError::new(
                    BattleBuildErrorKind::EnemyNotInEncounter,
                    Some(index),
                    Some(enemy_id.get()),
                ));
            }
            if enemy.unit() != participant.combatant().form() {
                return Err(BattleBuildError::new(
                    BattleBuildErrorKind::EnemyFormMismatch,
                    Some(index),
                    Some(enemy_id.get()),
                ));
            }
            Ok(())
        }
        (_, source) => Err(BattleBuildError::new(
            BattleBuildErrorKind::InvalidParticipantSource,
            Some(index),
            match source {
                ParticipantSource::Player => None,
                ParticipantSource::EncounterEnemy(enemy) => Some(enemy.get()),
                ParticipantSource::Linked(source) => Some(source.get()),
            },
        )),
    }
}
