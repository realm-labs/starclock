//! Service interaction bindings used by the spatial-free topology compiler.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityRngLabel, ActivitySlotId, ActivityValue,
};

use crate::{
    catalog::UniverseCatalog,
    id::RoomId,
    progression::ServiceKind,
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
}

pub(super) fn compile_room_services(
    catalog: &UniverseCatalog,
    runtime: &ServiceInteractionRuntimeCatalog,
    room: RoomId,
) -> Result<Option<Vec<RoomServiceBinding>>, UniverseTopologyCompileError> {
    let Some(domain_key) = catalog
        .room(room)
        .and_then(|definition| catalog.domain(definition.domain()))
        .map(|definition| definition.stable_key())
    else {
        return Ok(None);
    };
    let selections = match domain_key {
        "universe.domain.respite" => vec![
            (
                service_id(catalog, "universe.service.respite-offers")?,
                ServiceInteractionSelection::RespiteBlessing,
                "universe.service.respite-offers.one-star-blessing",
            ),
            (
                service_id(catalog, "universe.service.respite-offers")?,
                ServiceInteractionSelection::RespiteCurio,
                "universe.service.respite-offers.curio",
            ),
            (
                service_id(catalog, "universe.service.downloader")?,
                ServiceInteractionSelection::Activate,
                "universe.service.downloader",
            ),
        ],
        "universe.domain.transaction" => catalog
            .services()
            .iter()
            .filter(|service| {
                matches!(
                    service.kind(),
                    ServiceKind::BlessingShop | ServiceKind::CurioShop
                )
            })
            .map(|service| {
                (
                    service.id(),
                    ServiceInteractionSelection::Activate,
                    service.stable_key(),
                )
            })
            .collect(),
        _ => return Ok(None),
    };
    selections
        .into_iter()
        .map(|(service, selection, source_content_id)| {
            let compiled = runtime
                .compile_selection(service, &selection)
                .map_err(|_| UniverseTopologyCompileError::InvalidServiceInteraction)?;
            Ok(RoomServiceBinding {
                source_content_id: source_content_id.into(),
                handler: SERVICE_INTERACTION_HANDLER_ID,
                payload: compiled.payload().into(),
                random_candidate_count: compiled.random_candidate_count(),
                random_label: compiled
                    .random_candidate_count()
                    .map(|_| ActivityRngLabel::Shop),
                required_fragments: compiled.required_fragments(),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

pub(super) fn option_condition(
    room: ActivityCondition,
    fragments: ActivitySlotId,
    required: Option<u32>,
) -> ActivityCondition {
    let Some(amount) = required else {
        return room;
    };
    ActivityCondition::All(
        vec![
            room,
            ActivityCondition::Not(Box::new(ActivityCondition::LessThan(
                ActivityExpression::Slot(fragments),
                ActivityExpression::Literal(ActivityValue::BoundedInteger(i64::from(amount))),
            ))),
        ]
        .into_boxed_slice(),
    )
}

fn service_id(
    catalog: &UniverseCatalog,
    stable_key: &str,
) -> Result<crate::id::ServiceId, UniverseTopologyCompileError> {
    catalog
        .services()
        .iter()
        .find(|service| service.stable_key() == stable_key)
        .map(|service| service.id())
        .ok_or(UniverseTopologyCompileError::InvalidServiceInteraction)
}
