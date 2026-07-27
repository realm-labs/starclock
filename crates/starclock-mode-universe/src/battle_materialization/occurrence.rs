//! Occurrence-specific encounter overlay materialization.

use std::{collections::BTreeMap, sync::Arc};

use starclock_activity::{
    ActivityBattleResultContract, ActivityMetricProjectionBinding, ActivityOptionId,
    ActivityParticipantCarryDefinition, ActivitySlotId, BattleBinding, BattleResultProjection,
    EncounterInitiativePolicy, EnergyCarryPolicy, HpCarryPolicy, LifeCarryPolicy,
    MetricSettlementPolicy, MetricValueKind, PreparedBattleVariant, PresenceCarryPolicy,
    ProjectionField, ProjectionId, TechniqueContributionDigest,
};
use starclock_combat::{EnemyDefinitionId, ParticipantSpec, catalog::CombatCatalog};

use crate::{
    battle_contribution::UniverseBattleContributionSet,
    battle_overlay::UniverseEncounterBattleBinding,
    battle_technique::CompiledUniverseBattleTechnique,
    occurrence_battle::{OccurrenceBattleDefinition, OccurrenceBattleReward},
};

use super::{
    NORMAL_ENGAGEMENT_OPTION, UNIVERSE_BATTLE_MATERIALIZATION_REVISION,
    UniverseBattleMaterializationError, UniverseBattleRoster, battle_spec::member_spec,
    materialization_digest::technique_variant_digest, validate_executable,
};

const OCCURRENCE_PROJECTION_ID: u32 = 0x7540_0003;
pub(crate) const DEFEATED_ENEMY_COUNT_METRIC: &str = "enemy.defeated.count";

#[allow(clippy::too_many_arguments)]
pub(super) fn extend_overlay(
    battles: &[OccurrenceBattleDefinition],
    overlay: &mut Vec<UniverseEncounterBattleBinding>,
    roster: &UniverseBattleRoster,
    players: &[ParticipantSpec],
    technique_players: Option<&[ParticipantSpec]>,
    enemy_map: &BTreeMap<&str, EnemyDefinitionId>,
    combat_catalog: &Arc<CombatCatalog>,
    revision: &str,
    digest: [u8; 32],
    contributions: &UniverseBattleContributionSet,
    technique: Option<&CompiledUniverseBattleTechnique>,
) -> Result<(), UniverseBattleMaterializationError> {
    for battle in battles {
        let member = battle.member();
        let spec = member_spec(
            member,
            players,
            enemy_map,
            combat_catalog,
            revision,
            digest,
            contributions,
        )?;
        validate_executable(combat_catalog, &spec)?;
        let mut variants = vec![PreparedBattleVariant::new(
            Vec::new(),
            TechniqueContributionDigest::new(contributions.digest())
                .expect("contribution digest is non-zero"),
            BattleBinding::new(
                spec,
                "standard-universe-occurrence-battle",
                UNIVERSE_BATTLE_MATERIALIZATION_REVISION,
                roster.participant_lock(),
            )
            .map_err(|_| UniverseBattleMaterializationError::InvalidBattleBinding)?,
        )];
        if let (Some(technique), Some(technique_players)) = (technique, technique_players) {
            let technique_spec = member_spec(
                member,
                technique_players,
                enemy_map,
                combat_catalog,
                revision,
                digest,
                contributions,
            )?;
            validate_executable(combat_catalog, &technique_spec)?;
            variants.push(PreparedBattleVariant::new(
                vec![technique.definition().option()],
                TechniqueContributionDigest::new(technique_variant_digest(
                    contributions.digest(),
                    technique.digest(),
                ))
                .expect("combined technique digest is non-zero"),
                BattleBinding::new(
                    technique_spec,
                    "standard-universe-occurrence-battle-technique",
                    UNIVERSE_BATTLE_MATERIALIZATION_REVISION,
                    roster.participant_lock(),
                )
                .map_err(|_| UniverseBattleMaterializationError::InvalidBattleBinding)?,
            ));
        }
        let preparation = starclock_activity::EncounterPreparationDefinition::new(
            ActivityOptionId::new(u64::from(NORMAL_ENGAGEMENT_OPTION))
                .expect("reserved engagement option is non-zero"),
            EncounterInitiativePolicy::PlayerControlled,
            roster.participant_lock(),
            0,
            technique
                .map(|technique| vec![technique.activity_definition()])
                .unwrap_or_default(),
            variants,
        )
        .map_err(|_| UniverseBattleMaterializationError::InvalidBattleBinding)?;
        let reward_slot = match battle.reward() {
            OccurrenceBattleReward::BlessingPerDefeatedEnemy => {
                ActivitySlotId::new(crate::entry::state_layout::OCCURRENCE_BATTLE_REWARD_COUNT_SLOT)
                    .expect("reserved occurrence reward slot is non-zero")
            }
        };
        overlay.push(UniverseEncounterBattleBinding::new(
            member.id(),
            Arc::new(preparation),
            settlement_contract(roster, reward_slot)?,
        ));
    }
    Ok(())
}

fn settlement_contract(
    roster: &UniverseBattleRoster,
    reward_slot: ActivitySlotId,
) -> Result<Arc<ActivityBattleResultContract>, UniverseBattleMaterializationError> {
    let mut fields = vec![
        ProjectionField::Outcome,
        ProjectionField::FinalStateHash,
        ProjectionField::EventDigest,
        ProjectionField::TerminalFault,
    ];
    fields.extend(
        roster
            .entries()
            .iter()
            .map(|entry| ProjectionField::ParticipantState(entry.participant())),
    );
    fields.push(ProjectionField::Metric {
        key: DEFEATED_ENEMY_COUNT_METRIC.into(),
        kind: MetricValueKind::BoundedInteger,
    });
    let projection = BattleResultProjection::new(
        ProjectionId::new(OCCURRENCE_PROJECTION_ID)
            .expect("reserved occurrence projection ID is non-zero"),
        fields,
    )
    .map_err(|_| UniverseBattleMaterializationError::InvalidBattleBinding)?;
    let carry = roster
        .entries()
        .iter()
        .map(|entry| {
            ActivityParticipantCarryDefinition::new(
                entry.participant(),
                HpCarryPolicy::CarryExact,
                EnergyCarryPolicy::CarryExact,
                LifeCarryPolicy::CarryExact,
                PresenceCarryPolicy::CarryExact,
            )
        })
        .collect();
    let metric = ActivityMetricProjectionBinding::new(
        DEFEATED_ENEMY_COUNT_METRIC,
        MetricValueKind::BoundedInteger,
        reward_slot,
        MetricSettlementPolicy::Replace,
    )
    .expect("static occurrence metric binding is valid");
    ActivityBattleResultContract::new(Arc::new(projection), carry, vec![metric])
        .map(Arc::new)
        .map_err(|_| UniverseBattleMaterializationError::InvalidBattleBinding)
}
