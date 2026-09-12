//! Frozen `G22-P6-A12` external Adventure outcome mechanic partition.

use starclock_activity::{ActivityStateHash, ActivityTransactionEvent, GraphActivity};
use starclock_data::{
    divergent_universe_mechanic_catalog::{
        DivergentUniverseMechanicRuleId, DivergentUniverseMechanicSourceId,
    },
    divergent_universe_service_catalog::DivergentUniverseAdventureOutcomeId,
};

use crate::digest::Encoder;

use super::{
    DivergentUniverseAdventureExternalResult, DivergentUniverseRuntimeFactory,
    DivergentUniverseServiceAdventureError, DivergentUniverseServiceAdventureRuntime,
};

const PARTITION: &str = "G22-P6-A12";
const PROGRAM: &str = "divergent-universe.mechanic-rule.config-configadventuremodifier-adventuremodifier-rogue-tourn1-json";
const FIXTURE: &str = "divergent-universe.review-fixture.adventure-abstract-outcome";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseExternalOutcomeMechanicOperation {
    operation_type: Box<str>,
    ordinal: u16,
    source_occurrences: u32,
}

impl DivergentUniverseExternalOutcomeMechanicOperation {
    #[must_use]
    pub fn operation_type(&self) -> &str {
        &self.operation_type
    }

    #[must_use]
    pub const fn ordinal(&self) -> u16 {
        self.ordinal
    }

    #[must_use]
    pub const fn source_occurrences(&self) -> u32 {
        self.source_occurrences
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseExternalOutcomeMechanicDefinition {
    id: DivergentUniverseMechanicRuleId,
    source: DivergentUniverseMechanicSourceId,
    source_path: Box<str>,
    source_sha256: Box<str>,
    operations: Box<[DivergentUniverseExternalOutcomeMechanicOperation]>,
    digest: [u8; 32],
}

impl DivergentUniverseExternalOutcomeMechanicDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseMechanicRuleId {
        &self.id
    }

    #[must_use]
    pub const fn source(&self) -> &DivergentUniverseMechanicSourceId {
        &self.source
    }

    #[must_use]
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    #[must_use]
    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }

    #[must_use]
    pub fn operations(&self) -> &[DivergentUniverseExternalOutcomeMechanicOperation] {
        &self.operations
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseExternalOutcomeMechanicRuntime {
    definition: DivergentUniverseExternalOutcomeMechanicDefinition,
    service: DivergentUniverseServiceAdventureRuntime,
    digest: [u8; 32],
}

impl DivergentUniverseRuntimeFactory {
    pub fn activity_external_outcome_mechanic_runtime(
        &self,
    ) -> Result<
        DivergentUniverseExternalOutcomeMechanicRuntime,
        DivergentUniverseExternalOutcomeMechanicError,
    > {
        DivergentUniverseExternalOutcomeMechanicRuntime::compile(self)
    }
}

impl DivergentUniverseExternalOutcomeMechanicRuntime {
    fn compile(
        factory: &DivergentUniverseRuntimeFactory,
    ) -> Result<Self, DivergentUniverseExternalOutcomeMechanicError> {
        let catalog = factory.bundle.mechanic_catalog();
        let mut rules = catalog
            .rules()
            .iter()
            .filter(|rule| rule.state_lifecycle.as_ref() == "ExternalOutcomeSettlement");
        let rule = rules
            .next()
            .ok_or(DivergentUniverseExternalOutcomeMechanicError::InvalidCatalog)?;
        if rules.next().is_some()
            || rule.id.as_str() != PROGRAM
            || rule.scope.as_ref() != "CrossBattle"
            || rule.trigger.as_ref() != "AcceptedExternalAdventureResult"
            || rule.fixture_ids.len() != 1
            || rule.fixture_ids[0].as_ref() != FIXTURE
        {
            return Err(DivergentUniverseExternalOutcomeMechanicError::InvalidCatalog);
        }
        let source = catalog
            .sources()
            .binary_search_by(|source| source.id.cmp(&rule.source))
            .ok()
            .and_then(|index| catalog.sources().get(index))
            .ok_or(DivergentUniverseExternalOutcomeMechanicError::InvalidCatalog)?;
        if source.source_path.as_ref()
            != "Config/ConfigAdventureModifier/AdventureModifier_Rogue_Tourn1.json"
            || source.source_sha256.len() != 64
            || source.scope.as_ref() != "CrossBattle"
            || source.operation_types.len() != rule.ordered_operations.len()
        {
            return Err(DivergentUniverseExternalOutcomeMechanicError::InvalidCatalog);
        }
        let operations = rule
            .ordered_operations
            .iter()
            .map(
                |operation| DivergentUniverseExternalOutcomeMechanicOperation {
                    operation_type: operation.operation_type.clone(),
                    ordinal: operation.ordinal,
                    source_occurrences: operation.source_occurrences,
                },
            )
            .collect::<Vec<_>>()
            .into_boxed_slice();
        if operations.len() != 2
            || operations[0].operation_type() != "RPG.GameCore.SetDynamicValueByCustomName"
            || operations[0].ordinal() != 1
            || operations[0].source_occurrences() != 9
            || operations[1].operation_type() != "None"
            || operations[1].ordinal() != 2
            || operations[1].source_occurrences() != 9
        {
            return Err(DivergentUniverseExternalOutcomeMechanicError::InvalidCatalog);
        }
        let definition_digest = definition_digest(
            rule.id.as_str(),
            source.id.as_str(),
            &source.source_sha256,
            &operations,
        );
        let definition = DivergentUniverseExternalOutcomeMechanicDefinition {
            id: rule.id.clone(),
            source: source.id.clone(),
            source_path: source.source_path.clone(),
            source_sha256: source.source_sha256.clone(),
            operations,
            digest: definition_digest,
        };
        let mut encoder =
            Encoder::new(b"starclock.divergent-universe.external-outcome-mechanics.v1");
        encoder.text(PARTITION);
        encoder.digest(definition_digest);
        Ok(Self {
            definition,
            service: factory
                .service_adventure_runtime()
                .map_err(DivergentUniverseExternalOutcomeMechanicError::Service)?,
            digest: encoder.finish(),
        })
    }

    #[must_use]
    pub const fn definition(&self) -> &DivergentUniverseExternalOutcomeMechanicDefinition {
        &self.definition
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub fn settle(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        mechanic: &DivergentUniverseMechanicRuleId,
        adventure: &DivergentUniverseAdventureOutcomeId,
        result: DivergentUniverseAdventureExternalResult,
    ) -> Result<
        DivergentUniverseExternalOutcomeMechanicResolution,
        DivergentUniverseExternalOutcomeMechanicError,
    > {
        if mechanic != self.definition.id() {
            return Err(DivergentUniverseExternalOutcomeMechanicError::UnknownMechanic);
        }
        let settlement = self
            .service
            .settle_external_adventure_result(activity, expected, adventure, result)
            .map_err(DivergentUniverseExternalOutcomeMechanicError::Service)?;
        Ok(DivergentUniverseExternalOutcomeMechanicResolution {
            mechanic_digest: self.definition.digest(),
            tier: settlement.tier(),
            events: settlement.events().into(),
            state_hash: settlement.state_hash(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseExternalOutcomeMechanicResolution {
    mechanic_digest: [u8; 32],
    tier: u8,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseExternalOutcomeMechanicResolution {
    #[must_use]
    pub const fn mechanic_digest(&self) -> [u8; 32] {
        self.mechanic_digest
    }

    #[must_use]
    pub const fn tier(&self) -> u8 {
        self.tier
    }

    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }

    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

fn definition_digest(
    id: &str,
    source: &str,
    source_sha256: &str,
    operations: &[DivergentUniverseExternalOutcomeMechanicOperation],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.divergent-universe.external-outcome-mechanic.v1");
    encoder.text(id);
    encoder.text(source);
    encoder.text(source_sha256);
    encoder.u32(u32::try_from(operations.len()).expect("bounded operation shapes fit u32"));
    for operation in operations {
        encoder.text(operation.operation_type());
        encoder.u32(u32::from(operation.ordinal()));
        encoder.u32(operation.source_occurrences());
    }
    encoder.finish()
}

#[derive(Debug)]
pub enum DivergentUniverseExternalOutcomeMechanicError {
    InvalidCatalog,
    UnknownMechanic,
    Service(DivergentUniverseServiceAdventureError),
}

impl core::fmt::Display for DivergentUniverseExternalOutcomeMechanicError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe external outcome mechanic error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseExternalOutcomeMechanicError {}
