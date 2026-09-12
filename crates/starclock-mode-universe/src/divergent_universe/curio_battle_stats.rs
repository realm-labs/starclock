//! Immutable source-attributed passive modifiers, separate from locked builds.

use super::battle_passive_bindings::{PassiveBindings, bind_passives};
use super::{
    DivergentUniverseBattleAssemblyError, DivergentUniverseCurioRuntime,
    DivergentUniverseCurioSnapshot, DivergentUniverseEntryFlowError,
    DivergentUniverseRuntimeFactory,
};
use crate::{digest::CanonicalDigestBuilder, path_lowering::parse_decimal};
use starclock_activity::{
    ActivityOperation, ActivityPlayerView, GraphActivityCommandError, GraphActivityRuntimeError,
};
use starclock_combat::{
    ModifierDefinitionId, ModifierStackingGroupId, ParticipantSpec, ResolvedModifierBinding,
    Scalar, SourceDefinitionId, TeamSide,
    catalog::builder::CombatCatalogBuilder,
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{RuleSource, RuleValue, SourceClass, ValueExpr},
};
use starclock_data::divergent_universe_decisions::{
    CurioBattleStat, CurioBattleStatDefinition, CurioBattleStatPolicy,
};

#[derive(Clone, Debug)]
pub(super) struct CurioBattleStats {
    definitions: Box<[CurioBattleStatDefinition]>,
    curios: DivergentUniverseCurioRuntime,
}

struct Attachment {
    modifier: ModifierDefinition,
    group: ModifierStackingGroup,
    source: RuleSource,
}

impl CurioBattleStats {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let definitions = factory
            .decision_catalog()
            .curio_battle_stats()
            .to_vec()
            .into_boxed_slice();
        for definition in &definitions {
            match definition.policy {
                CurioBattleStatPolicy::VersionedProjectPolicyBasePercentVerifiedBattleLifetime => {}
                CurioBattleStatPolicy::VersionedProjectPolicyOutgoingFinalMultiplierVerifiedBattleLifetime => {}
            }
        }
        Ok(Self {
            definitions,
            curios: factory
                .curio_runtime()
                .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?,
        })
    }

    pub(super) fn settle_lifetimes(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        self.curios.battle_lifetime_operations(view).map_err(|_| {
            GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
        })
    }

    /// Adds only reviewed active passives to the already authenticated snapshot.
    /// Catalog construction rejects collisions; the locked build remains intact.
    pub(super) fn assemble(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &DivergentUniverseCurioSnapshot,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        let mut attachments = Vec::new();
        for (index, definition) in self.definitions.iter().enumerate() {
            if !snapshot
                .contributions()
                .iter()
                .any(|held| held.state() == &definition.state && held.charges() > 0)
            {
                continue;
            }
            // Local compiled addresses are stable for sorted current definitions;
            // source identity/digest carries the authored stable key, not row IDs.
            let purposes: &[FormulaPurpose] = match definition.stat {
                CurioBattleStat::Speed => &[FormulaPurpose::Stat],
                CurioBattleStat::FinalDamage => &[
                    FormulaPurpose::OrdinaryDamage,
                    FormulaPurpose::Dot,
                    FormulaPurpose::AdditionalDamage,
                    FormulaPurpose::ElationDamage,
                    FormulaPurpose::Break,
                    FormulaPurpose::SuperBreak,
                ],
            };
            let (stat, stage, aggregation) = match definition.stat {
                CurioBattleStat::Speed => (
                    StatKind::Spd,
                    FormulaStage::PercentOfBase,
                    ModifierAggregation::UniquePerSource,
                ),
                // Formula stages do not participate in underlying ATK queries.
                CurioBattleStat::FinalDamage => (
                    StatKind::Atk,
                    FormulaStage::DamageFinalMultiply,
                    ModifierAggregation::Product,
                ),
            };
            for (purpose_index, purpose) in purposes.iter().copied().enumerate() {
                let ordinal = u32::try_from(index)
                    .ok()
                    .and_then(|index| index.checked_mul(8))
                    .and_then(|index| index.checked_add(u32::try_from(purpose_index).ok()?))
                    .and_then(|index| index.checked_add(1))
                    .filter(|index| *index <= 65535)
                    .ok_or_else(invalid)?;
                let modifier = ModifierDefinitionId::new(
                    0x7e28_0000_u32.checked_add(ordinal).ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?;
                let group = ModifierStackingGroupId::new(
                    0x7e29_0000_u32.checked_add(ordinal).ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?;
                let source = SourceDefinitionId::new(
                    0x7e27_0000_u32.checked_add(ordinal).ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?;
                let parameter = parse_decimal(&definition.bonus_fraction).map_err(|_| invalid())?;
                let exponent = Scalar::FRACTIONAL_DIGITS
                    .checked_sub(u32::from(parameter.scale()))
                    .ok_or_else(invalid)?;
                let value = parameter
                    .coefficient()
                    .checked_mul(10_i64.checked_pow(exponent).ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
                let value = match definition.stat {
                    CurioBattleStat::Speed => Scalar::from_scaled(value),
                    CurioBattleStat::FinalDamage => Scalar::ONE
                        .checked_add(Scalar::from_scaled(value))
                        .map_err(|_| invalid())?,
                };
                let mut digest = CanonicalDigestBuilder::new();
                digest.update(b"starclock.divergent-universe.curio-battle-stat-source");
                digest.update(assembly_digest);
                digest.update(definition.key.as_bytes());
                attachments.push(Attachment {
                    group: ModifierStackingGroup {
                        id: group,
                        aggregation,
                        comparator: None,
                    },
                    modifier: ModifierDefinition {
                        id: modifier,
                        stat,
                        stage,
                        purpose,
                        value: ValueExpr::Literal(RuleValue::Scalar(value)),
                        stacking_group: group,
                        priority: 0,
                        floor: None,
                        cap: None,
                        cap_stage: stage,
                        snapshot: SnapshotPolicy::Dynamic,
                        source_stack_slot: None,
                        filters: Box::new([]),
                    },
                    source: RuleSource::new(
                        source,
                        SourceClass::Mode,
                        Vec::new(),
                        digest.finalize(),
                    ),
                });
            }
        }
        if attachments.is_empty() {
            return Ok(());
        }
        for player in players {
            if player.side() != TeamSide::Player {
                return Err(invalid());
            }
            let added = PassiveBindings {
                modifiers: attachments
                    .iter()
                    .map(|attachment| {
                        ResolvedModifierBinding::new(
                            attachment.modifier.id,
                            attachment.source.definition(),
                        )
                        .with_linked_subjects()
                    })
                    .collect(),
                sources: attachments
                    .iter()
                    .map(|attachment| attachment.source.clone())
                    .collect(),
                rule_bundles: Vec::new(),
            };
            *player = bind_passives(player, &added, assembly_digest)?;
        }
        for attachment in attachments {
            builder.add_modifier_group(attachment.group);
            builder.add_modifier(attachment.modifier);
        }
        Ok(())
    }
}

fn invalid() -> DivergentUniverseBattleAssemblyError {
    DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants
}
