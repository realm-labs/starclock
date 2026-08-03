//! Service interaction bindings used by the spatial-free topology compiler.

use crate::id::ServiceId;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityRngLabel, ActivitySlotId, ActivityValue,
    ParticipantId, ParticipantLock,
};

use crate::{
    ability_runtime::AbilityTarget,
    catalog::UniverseCatalog,
    id::RoomId,
    progression::ServiceKind,
    service_effect_runtime::TrailblazeBonusTier,
    service_interaction::{
        SERVICE_INTERACTION_HANDLER_ID, ServiceInteractionRuntimeCatalog,
        ServiceInteractionSelection,
    },
    topology::UniverseTopologyCompileError,
};

pub(super) struct RoomServiceBinding {
    pub(super) source_content_id: Box<str>,
    pub(super) handler: u32,
    pub(super) payload: Box<[u8]>,
    pub(super) random_candidate_count: Option<u32>,
    pub(super) random_label: Option<ActivityRngLabel>,
    pub(super) required_fragments: Option<u32>,
    pub(super) required_ability: Option<AbilityTarget>,
    pub(super) required_defeated_participant: Option<ParticipantId>,
}

struct ServiceSelectionSpec {
    service: ServiceId,
    selection: ServiceInteractionSelection,
    source_content_id: Box<str>,
    required_ability: Option<AbilityTarget>,
    required_defeated_participant: Option<ParticipantId>,
}

pub(super) fn compile_room_services(
    catalog: &UniverseCatalog,
    runtime: &ServiceInteractionRuntimeCatalog,
    participants: &ParticipantLock,
    room: RoomId,
) -> Result<Option<Vec<RoomServiceBinding>>, UniverseTopologyCompileError> {
    let Some(domain_key) = catalog
        .room(room)
        .and_then(|definition| catalog.domain(definition.domain()))
        .map(|definition| definition.stable_key())
    else {
        return Ok(None);
    };
    let selections: Vec<ServiceSelectionSpec> = match domain_key {
        "universe.domain.respite" => {
            let mut values = vec![
                ServiceSelectionSpec {
                    service: service_id(catalog, "universe.service.respite-offers")?,
                    selection: ServiceInteractionSelection::RespiteBlessing,
                    source_content_id: "universe.service.respite-offers.one-star-blessing".into(),
                    required_ability: None,
                    required_defeated_participant: None,
                },
                ServiceSelectionSpec {
                    service: service_id(catalog, "universe.service.respite-offers")?,
                    selection: ServiceInteractionSelection::RespiteCurio,
                    source_content_id: "universe.service.respite-offers.curio".into(),
                    required_ability: None,
                    required_defeated_participant: None,
                },
                ServiceSelectionSpec {
                    service: service_id(catalog, "universe.service.downloader")?,
                    selection: ServiceInteractionSelection::Activate,
                    source_content_id: "universe.service.downloader".into(),
                    required_ability: None,
                    required_defeated_participant: None,
                },
            ];
            let reviver = service_id(catalog, "universe.service.reviver")?;
            values.extend(participants.entries().iter().map(|entry| {
                ServiceSelectionSpec {
                    service: reviver,
                    selection: ServiceInteractionSelection::ReviveCharacter(entry.participant()),
                    source_content_id: format!(
                        "universe.service.reviver.participant.{}",
                        entry.participant().get()
                    )
                    .into(),
                    required_ability: Some(AbilityTarget::ServiceReviver),
                    required_defeated_participant: Some(entry.participant()),
                }
            }));
            values
        }
        "universe.domain.transaction" => catalog
            .services()
            .iter()
            .filter(|service| {
                matches!(
                    service.kind(),
                    ServiceKind::BlessingShop | ServiceKind::CurioShop
                )
            })
            .map(|service| ServiceSelectionSpec {
                service: service.id(),
                selection: ServiceInteractionSelection::Activate,
                source_content_id: service.stable_key().into(),
                required_ability: None,
                required_defeated_participant: None,
            })
            .collect(),
        _ => return Ok(None),
    };
    selections
        .into_iter()
        .map(|spec| {
            let compiled = runtime
                .compile_selection(spec.service, &spec.selection)
                .map_err(|_| UniverseTopologyCompileError::InvalidServiceInteraction)?;
            Ok(RoomServiceBinding {
                source_content_id: spec.source_content_id,
                handler: SERVICE_INTERACTION_HANDLER_ID,
                payload: compiled.payload().into(),
                random_candidate_count: compiled.random_candidate_count(),
                random_label: compiled
                    .random_candidate_count()
                    .map(|_| ActivityRngLabel::Shop),
                required_fragments: compiled.required_fragments(),
                required_ability: spec.required_ability,
                required_defeated_participant: spec.required_defeated_participant,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

pub(super) fn option_condition(
    room: ActivityCondition,
    fragments: ActivitySlotId,
    required: Option<u32>,
    ability_projection: ActivitySlotId,
    required_ability: Option<AbilityTarget>,
    required_defeated_participant: Option<ParticipantId>,
) -> ActivityCondition {
    let mut conditions = vec![room];
    if let Some(amount) = required {
        conditions.push(ActivityCondition::Not(Box::new(
            ActivityCondition::LessThan(
                ActivityExpression::Slot(fragments),
                ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::from(amount))),
            ),
        )));
    }
    if let Some(target) = required_ability {
        conditions.push(ActivityCondition::Equal(
            ActivityExpression::CounterValue {
                slot: ability_projection,
                key: target.activity_key(),
            },
            ActivityExpression::Literal(ActivityValue::BoundedInteger(1_000_000)),
        ));
    }
    if let Some(participant) = required_defeated_participant {
        conditions.push(ActivityCondition::ParticipantDefeated(participant));
    }
    ActivityCondition::All(conditions.into_boxed_slice())
}

pub(super) fn trailblaze_bonus_condition(
    fragments: ActivitySlotId,
    required_fragments: Option<u32>,
    ability_projection: ActivitySlotId,
    tier: TrailblazeBonusTier,
) -> ActivityCondition {
    let enhanced = ActivityCondition::Equal(
        ActivityExpression::CounterValue {
            slot: ability_projection,
            key: AbilityTarget::EnhancedTrailblazeBonus.activity_key(),
        },
        ActivityExpression::Literal(ActivityValue::BoundedInteger(1_000_000)),
    );
    let mut conditions = vec![match tier {
        TrailblazeBonusTier::Ordinary => ActivityCondition::Not(Box::new(enhanced)),
        TrailblazeBonusTier::Enhanced => enhanced,
    }];
    if let Some(required) = required_fragments {
        conditions.push(ActivityCondition::Not(Box::new(
            ActivityCondition::LessThan(
                ActivityExpression::Slot(fragments),
                ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::from(required))),
            ),
        )));
    }
    ActivityCondition::All(conditions.into_boxed_slice())
}

fn service_id(
    catalog: &UniverseCatalog,
    stable_key: &str,
) -> Result<ServiceId, UniverseTopologyCompileError> {
    catalog
        .services()
        .iter()
        .find(|service| service.stable_key() == stable_key)
        .map(|service| service.id())
        .ok_or(UniverseTopologyCompileError::InvalidServiceInteraction)
}
