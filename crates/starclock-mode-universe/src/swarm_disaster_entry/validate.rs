use starclock_activity::{LoadoutLockScope, ParticipantPolicy, ParticipantUniquenessScope};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};

pub(super) fn validate_participants(
    actual: ParticipantPolicy,
) -> Result<(), UniverseCatalogLoadError> {
    let expected = ParticipantPolicy::new(
        1,
        1,
        4,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .expect("static Swarm participant policy is valid");
    if actual != expected {
        return Err(error("Swarm participant policy mismatch"));
    }
    Ok(())
}

pub(super) fn canonical_communing(
    catalog: &SwarmDisasterUniqueCatalog,
    input: &[(Box<str>, u16)],
) -> Result<Vec<(u32, u16)>, UniverseCatalogLoadError> {
    let mut values = input
        .iter()
        .map(|(key, value)| {
            let (id, maximum) = catalog
                .communing_dimension(key)
                .ok_or_else(|| reference("unknown Communing dimension"))?;
            if *value > maximum {
                return Err(error("Communing dimension exceeds released maximum"));
            }
            Ok((id, *value))
        })
        .collect::<Result<Vec<_>, _>>()?;
    values.sort_unstable_by_key(|(id, _)| *id);
    if values.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(error("duplicate Communing dimension"));
    }
    Ok(values)
}

pub(super) fn canonical_progression(
    catalog: &SwarmDisasterUniqueCatalog,
    input: &[Box<str>],
) -> Result<Vec<u64>, UniverseCatalogLoadError> {
    let mut values = input
        .iter()
        .map(|key| {
            catalog
                .progression_key(key)
                .ok_or_else(|| reference("unknown progression key"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    values.sort_unstable();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(error("duplicate progression key"));
    }
    Ok(values)
}

pub(super) fn error(message: &str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

pub(super) fn reference(message: &str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}
