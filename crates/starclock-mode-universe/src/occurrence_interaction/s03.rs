use super::*;

const ALL: &str = "universe.blessing-pool.all";
const ONE_STAR: &str = "universe.blessing-pool.rarity.1";
const THREE_STAR: &str = "universe.blessing-pool.rarity.3";
const PRESERVATION_TWO_STAR: &str = "universe.blessing-pool.path.preservation.rarity.2";

pub(super) fn referenced_blessings(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
) -> Result<Vec<u64>, OccurrenceInteractionError> {
    let mut selected = Vec::new();
    for reference in outcome.parameter_refs() {
        let reference = reference.as_ref();
        if reference.starts_with("universe.blessing.") {
            let blessing = catalog
                .blessings()
                .iter()
                .find(|value| value.stable_key() == reference)
                .ok_or(OccurrenceInteractionError::InvalidChoice)?;
            selected.push(u64::from(blessing.id().get()));
            continue;
        }
        if reference.starts_with("universe.blessing-pool.")
            && !matches!(
                reference,
                ALL | ONE_STAR | THREE_STAR | PRESERVATION_TWO_STAR
            )
        {
            return Err(OccurrenceInteractionError::InvalidChoice);
        }
        if !reference.starts_with("universe.blessing-pool.") {
            continue;
        }
        let preservation = catalog
            .paths()
            .iter()
            .find(|path| path.stable_key() == "universe.path.preservation")
            .expect("validated Preservation path")
            .id();
        selected.extend(
            catalog
                .blessings()
                .iter()
                .filter(|value| match reference {
                    ALL => true,
                    ONE_STAR => value.rarity() == 1,
                    THREE_STAR => value.rarity() == 3,
                    PRESERVATION_TWO_STAR => value.rarity() == 2 && value.path() == preservation,
                    _ => false,
                })
                .map(|value| u64::from(value.id().get())),
        );
    }
    if selected.is_empty() {
        selected.extend(
            catalog
                .blessings()
                .iter()
                .map(|value| u64::from(value.id().get())),
        );
    }
    selected.sort_unstable();
    selected.dedup();
    Ok(selected)
}
