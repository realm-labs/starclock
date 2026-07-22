//! Shared Blessing acquisition, level and contribution compilation.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityInventoryId, ActivityInventoryView,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityValue,
};

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{BlessingId, BlessingLevelId, PathId},
    path::ExactParameter,
};

pub const BLESSING_RUNTIME_REVISION: &str = "standard-universe-blessing-runtime-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlessingOfferEligibility {
    rarities: Box<[u8]>,
    unlocked_prerequisites: Option<Box<[Box<str>]>>,
}

impl BlessingOfferEligibility {
    pub fn fully_unlocked(mut rarities: Vec<u8>) -> Result<Self, BlessingRuntimeError> {
        rarities.sort_unstable();
        rarities.dedup();
        validate_rarities(&rarities)?;
        Ok(Self {
            rarities: rarities.into_boxed_slice(),
            unlocked_prerequisites: None,
        })
    }

    pub fn explicit(
        mut rarities: Vec<u8>,
        mut unlocked_prerequisites: Vec<Box<str>>,
    ) -> Result<Self, BlessingRuntimeError> {
        rarities.sort_unstable();
        rarities.dedup();
        validate_rarities(&rarities)?;
        unlocked_prerequisites.sort_unstable();
        if unlocked_prerequisites
            .iter()
            .any(|key| key.trim().is_empty())
            || unlocked_prerequisites
                .windows(2)
                .any(|pair| pair[0] == pair[1])
        {
            return Err(BlessingRuntimeError::InvalidOfferEligibility);
        }
        Ok(Self {
            rarities: rarities.into_boxed_slice(),
            unlocked_prerequisites: Some(unlocked_prerequisites.into_boxed_slice()),
        })
    }

    #[must_use]
    pub fn allows(&self, definition: &BlessingRuntimeDefinition) -> bool {
        self.rarities.binary_search(&definition.rarity).is_ok()
            && self.unlocked_prerequisites.as_ref().is_none_or(|unlocked| {
                definition
                    .prerequisite_keys
                    .iter()
                    .all(|required| unlocked.binary_search(required).is_ok())
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlessingLevelContribution {
    id: BlessingLevelId,
    level: u8,
    source_binding_key: Box<str>,
    rule_key: Box<str>,
    parameters: Box<[ExactParameter]>,
}

impl BlessingLevelContribution {
    #[must_use]
    pub const fn id(&self) -> BlessingLevelId {
        self.id
    }
    #[must_use]
    pub const fn level(&self) -> u8 {
        self.level
    }
    #[must_use]
    pub fn source_binding_key(&self) -> &str {
        &self.source_binding_key
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub fn parameters(&self) -> &[ExactParameter] {
        &self.parameters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlessingRuntimeDefinition {
    blessing: BlessingId,
    path: PathId,
    rarity: u8,
    prerequisite_keys: Box<[Box<str>]>,
    mechanic_tags: Box<[Box<str>]>,
    levels: [BlessingLevelContribution; 2],
}

impl BlessingRuntimeDefinition {
    #[must_use]
    pub const fn blessing(&self) -> BlessingId {
        self.blessing
    }
    #[must_use]
    pub const fn path(&self) -> PathId {
        self.path
    }
    #[must_use]
    pub const fn rarity(&self) -> u8 {
        self.rarity
    }
    #[must_use]
    pub fn prerequisite_keys(&self) -> &[Box<str>] {
        &self.prerequisite_keys
    }
    #[must_use]
    pub fn mechanic_tags(&self) -> &[Box<str>] {
        &self.mechanic_tags
    }
    #[must_use]
    pub fn level(&self, level: u8) -> Option<&BlessingLevelContribution> {
        self.levels.get(usize::from(level.checked_sub(1)?))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlessingRuntimeCatalog {
    definitions: Box<[BlessingRuntimeDefinition]>,
    digest: [u8; 32],
}

impl BlessingRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, BlessingRuntimeError> {
        let mut definitions = Vec::with_capacity(catalog.blessings().len());
        for blessing in catalog.blessings() {
            if !(1..=3).contains(&blessing.rarity()) || blessing.levels().len() != 2 {
                return Err(BlessingRuntimeError::InvalidDefinition(blessing.id()));
            }
            let mut levels = blessing
                .levels()
                .iter()
                .map(|id| {
                    let level = catalog
                        .blessing_level(*id)
                        .ok_or(BlessingRuntimeError::MissingLevel(*id))?;
                    if level.blessing() != blessing.id() || !(1..=2).contains(&level.level()) {
                        return Err(BlessingRuntimeError::InvalidLevel(*id));
                    }
                    Ok(BlessingLevelContribution {
                        id: level.id(),
                        level: level.level(),
                        source_binding_key: level.source_binding_key().into(),
                        rule_key: level.rule_key().into(),
                        parameters: level.parameters().to_vec().into_boxed_slice(),
                    })
                })
                .collect::<Result<Vec<_>, BlessingRuntimeError>>()?;
            levels.sort_by_key(BlessingLevelContribution::level);
            let levels: [BlessingLevelContribution; 2] = levels
                .try_into()
                .map_err(|_| BlessingRuntimeError::InvalidDefinition(blessing.id()))?;
            if levels[0].level != 1 || levels[1].level != 2 {
                return Err(BlessingRuntimeError::InvalidDefinition(blessing.id()));
            }
            definitions.push(BlessingRuntimeDefinition {
                blessing: blessing.id(),
                path: blessing.path(),
                rarity: blessing.rarity(),
                prerequisite_keys: blessing.prerequisite_keys().to_vec().into_boxed_slice(),
                mechanic_tags: blessing.mechanic_tags().to_vec().into_boxed_slice(),
                levels,
            });
        }
        definitions.sort_by_key(BlessingRuntimeDefinition::blessing);
        if definitions.len() != 162
            || definitions
                .windows(2)
                .any(|pair| pair[0].blessing == pair[1].blessing)
        {
            return Err(BlessingRuntimeError::InvalidDenominator);
        }
        let digest = contribution_catalog_digest(&definitions);
        Ok(Self {
            definitions: definitions.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub fn definitions(&self) -> &[BlessingRuntimeDefinition] {
        &self.definitions
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub fn definition(&self, id: BlessingId) -> Option<&BlessingRuntimeDefinition> {
        self.definitions
            .binary_search_by_key(&id, BlessingRuntimeDefinition::blessing)
            .ok()
            .map(|index| &self.definitions[index])
    }

    pub fn eligible<'a>(
        &'a self,
        policy: &'a BlessingOfferEligibility,
    ) -> impl Iterator<Item = &'a BlessingRuntimeDefinition> + 'a {
        self.definitions
            .iter()
            .filter(move |definition| policy.allows(definition))
    }

    #[must_use]
    pub fn acquisition_option(
        &self,
        id: BlessingId,
        option: ActivityOptionId,
        priority: i32,
        inventory: ActivityInventoryId,
        mut settlement: Vec<ActivityOperation>,
    ) -> Option<ActivityOptionDefinition> {
        self.definition(id)?;
        let count = inventory_count(inventory, id);
        settlement.insert(
            0,
            ActivityOperation::AddInventory {
                inventory,
                content: u64::from(id.get()),
                count: integer(1),
            },
        );
        Some(ActivityOptionDefinition::new(
            option,
            priority,
            equals(count, 0),
            settlement,
        ))
    }

    #[must_use]
    pub fn enhancement_operations(
        &self,
        inventory: ActivityInventoryId,
        id: BlessingId,
    ) -> Option<Box<[ActivityOperation]>> {
        self.definition(id)?;
        Some(
            vec![
                ActivityOperation::Require(equals(inventory_count(inventory, id), 1)),
                ActivityOperation::AddInventory {
                    inventory,
                    content: u64::from(id.get()),
                    count: integer(1),
                },
            ]
            .into_boxed_slice(),
        )
    }

    #[must_use]
    pub fn replacement_operations(
        &self,
        inventory: ActivityInventoryId,
        removed: BlessingId,
        acquired: BlessingId,
    ) -> Option<Box<[ActivityOperation]>> {
        if removed == acquired
            || self.definition(removed).is_none()
            || self.definition(acquired).is_none()
        {
            return None;
        }
        let removed_count = inventory_count(inventory, removed);
        Some(
            vec![
                ActivityOperation::Require(ActivityCondition::LessThan(
                    integer(0),
                    removed_count.clone(),
                )),
                ActivityOperation::Require(equals(inventory_count(inventory, acquired), 0)),
                ActivityOperation::RemoveInventory {
                    inventory,
                    content: u64::from(removed.get()),
                    count: removed_count,
                },
                ActivityOperation::AddInventory {
                    inventory,
                    content: u64::from(acquired.get()),
                    count: integer(1),
                },
            ]
            .into_boxed_slice(),
        )
    }

    pub fn contributions(
        &self,
        inventory: &ActivityInventoryView,
    ) -> Result<BlessingContributionSet, BlessingRuntimeError> {
        self.contributions_from_raw(inventory.entries())
    }

    pub fn contributions_from_owned(
        &self,
        entries: &[(BlessingId, u32)],
    ) -> Result<BlessingContributionSet, BlessingRuntimeError> {
        let raw = entries
            .iter()
            .map(|(id, stacks)| (u64::from(id.get()), *stacks))
            .collect::<Vec<_>>();
        self.contributions_from_raw(&raw)
    }

    fn contributions_from_raw(
        &self,
        inventory: &[(u64, u32)],
    ) -> Result<BlessingContributionSet, BlessingRuntimeError> {
        let mut entries = Vec::with_capacity(inventory.len());
        for (raw, stacks) in inventory {
            let raw = u32::try_from(*raw)
                .map_err(|_| BlessingRuntimeError::UnknownInventoryEntry(*raw))?;
            let id = BlessingId::new(raw)
                .ok_or(BlessingRuntimeError::UnknownInventoryEntry(u64::from(raw)))?;
            let definition = self
                .definition(id)
                .ok_or(BlessingRuntimeError::UnknownInventoryEntry(u64::from(raw)))?;
            let level = u8::try_from(*stacks)
                .ok()
                .and_then(|level| definition.level(level))
                .ok_or(BlessingRuntimeError::InvalidInventoryLevel(id))?;
            entries.push(BlessingContribution {
                blessing: id,
                path: definition.path,
                rarity: definition.rarity,
                level: level.clone(),
                mechanic_tags: definition.mechanic_tags.clone(),
            });
        }
        entries.sort_by_key(BlessingContribution::blessing);
        Ok(BlessingContributionSet::new(entries))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlessingContribution {
    blessing: BlessingId,
    path: PathId,
    rarity: u8,
    level: BlessingLevelContribution,
    mechanic_tags: Box<[Box<str>]>,
}

impl BlessingContribution {
    #[must_use]
    pub const fn blessing(&self) -> BlessingId {
        self.blessing
    }
    #[must_use]
    pub const fn path(&self) -> PathId {
        self.path
    }
    #[must_use]
    pub const fn rarity(&self) -> u8 {
        self.rarity
    }
    #[must_use]
    pub const fn level(&self) -> &BlessingLevelContribution {
        &self.level
    }
    #[must_use]
    pub fn mechanic_tags(&self) -> &[Box<str>] {
        &self.mechanic_tags
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlessingContributionSet {
    entries: Box<[BlessingContribution]>,
    digest: [u8; 32],
}

impl BlessingContributionSet {
    fn new(entries: Vec<BlessingContribution>) -> Self {
        let digest = contribution_set_digest(&entries);
        Self {
            entries: entries.into_boxed_slice(),
            digest,
        }
    }
    #[must_use]
    pub fn entries(&self) -> &[BlessingContribution] {
        &self.entries
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

fn inventory_count(inventory: ActivityInventoryId, id: BlessingId) -> ActivityExpression {
    ActivityExpression::InventoryCount {
        inventory,
        content: u64::from(id.get()),
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn equals(expression: ActivityExpression, value: i64) -> ActivityCondition {
    ActivityCondition::Equal(expression, integer(value))
}

fn validate_rarities(rarities: &[u8]) -> Result<(), BlessingRuntimeError> {
    if rarities.is_empty() || rarities.iter().any(|rarity| !(1..=3).contains(rarity)) {
        Err(BlessingRuntimeError::InvalidOfferEligibility)
    } else {
        Ok(())
    }
}

fn contribution_catalog_digest(definitions: &[BlessingRuntimeDefinition]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-blessing-runtime-catalog-v1");
    encoder.text(BLESSING_RUNTIME_REVISION);
    encoder.u32(definitions.len() as u32);
    for definition in definitions {
        encoder.u32(definition.blessing.get());
        encoder.u32(definition.path.get());
        encoder.u8(definition.rarity);
        encoder.u32(definition.prerequisite_keys.len() as u32);
        for key in &definition.prerequisite_keys {
            encoder.text(key);
        }
        encode_levels(&mut encoder, &definition.levels);
    }
    encoder.finish()
}

fn contribution_set_digest(entries: &[BlessingContribution]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-blessing-contribution-set-v1");
    encoder.u32(entries.len() as u32);
    for entry in entries {
        encoder.u32(entry.blessing.get());
        encoder.u32(entry.path.get());
        encoder.u8(entry.rarity);
        encode_levels(&mut encoder, core::slice::from_ref(&entry.level));
        encoder.u32(entry.mechanic_tags.len() as u32);
        for tag in &entry.mechanic_tags {
            encoder.text(tag);
        }
    }
    encoder.finish()
}

fn encode_levels(encoder: &mut Encoder, levels: &[BlessingLevelContribution]) {
    encoder.u32(levels.len() as u32);
    for level in levels {
        encoder.u32(level.id.get());
        encoder.u8(level.level);
        encoder.text(&level.source_binding_key);
        encoder.text(&level.rule_key);
        encoder.u32(level.parameters.len() as u32);
        for parameter in &level.parameters {
            encoder.i64(parameter.coefficient());
            encoder.u8(parameter.scale());
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlessingRuntimeError {
    InvalidDenominator,
    InvalidDefinition(BlessingId),
    MissingLevel(BlessingLevelId),
    InvalidLevel(BlessingLevelId),
    UnknownInventoryEntry(u64),
    InvalidInventoryLevel(BlessingId),
    InvalidOfferEligibility,
}
