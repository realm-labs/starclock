//! Immutable composition boundary for Activity extension handlers.

use crate::{
    ActivityHandlerId, ActivityOperation, ActivityPlayerView, codec::ActivityRegistryWriter,
};

pub const ACTIVITY_HANDLER_REGISTRY_REVISION: &str = "activity-handler-registry-v1";
pub const MAX_ACTIVITY_HANDLER_BUNDLES: usize = 64;
pub const MAX_ACTIVITY_HANDLERS: usize = 4_096;
pub const MAX_ACTIVITY_HANDLER_PAYLOAD_BYTES: usize = 64 * 1024;

pub fn core_activity_handler_bundle() -> ActivityHandlerBundle {
    ActivityHandlerBundle::new(
        "starclock.activity.core",
        ACTIVITY_HANDLER_REGISTRY_REVISION,
        Vec::new(),
        Vec::new(),
    )
    .expect("the static core Activity handler bundle is valid")
}

#[derive(Clone, Copy, Debug)]
pub struct ActivityHandlerInput<'a> {
    view: &'a ActivityPlayerView,
    payload: &'a [u8],
    random_index: Option<u32>,
}

impl<'a> ActivityHandlerInput<'a> {
    pub fn new(
        view: &'a ActivityPlayerView,
        payload: &'a [u8],
    ) -> Result<Self, ActivityHandlerFault> {
        if payload.len() > MAX_ACTIVITY_HANDLER_PAYLOAD_BYTES {
            return Err(ActivityHandlerFault::new(
                ActivityHandlerFaultKind::PayloadTooLarge,
            ));
        }
        Ok(Self {
            view,
            payload,
            random_index: None,
        })
    }

    pub(crate) fn with_random_index(mut self, random_index: Option<u32>) -> Self {
        self.random_index = random_index;
        self
    }

    #[must_use]
    pub const fn view(self) -> &'a ActivityPlayerView {
        self.view
    }

    #[must_use]
    pub const fn payload(self) -> &'a [u8] {
        self.payload
    }

    #[must_use]
    pub const fn random_index(self) -> Option<u32> {
        self.random_index
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityHandlerOutput {
    operations: Box<[ActivityOperation]>,
}

impl ActivityHandlerOutput {
    #[must_use]
    pub fn new(operations: Vec<ActivityOperation>) -> Self {
        Self {
            operations: operations.into_boxed_slice(),
        }
    }

    #[must_use]
    pub fn operations(&self) -> &[ActivityOperation] {
        &self.operations
    }
}

pub type ActivityHandler =
    for<'a> fn(ActivityHandlerInput<'a>) -> Result<ActivityHandlerOutput, ActivityHandlerFault>;

#[derive(Clone, Copy)]
pub struct ActivityHandlerRegistration {
    id: ActivityHandlerId,
    stable_key: &'static str,
    version: &'static str,
    payload_schema_digest: [u8; 32],
    determinism_note: &'static str,
    owner: &'static str,
    handler: ActivityHandler,
}

impl ActivityHandlerRegistration {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        id: ActivityHandlerId,
        stable_key: &'static str,
        version: &'static str,
        payload_schema_digest: [u8; 32],
        determinism_note: &'static str,
        owner: &'static str,
        handler: ActivityHandler,
    ) -> Self {
        Self {
            id,
            stable_key,
            version,
            payload_schema_digest,
            determinism_note,
            owner,
            handler,
        }
    }

    #[must_use]
    pub const fn id(self) -> ActivityHandlerId {
        self.id
    }

    #[must_use]
    pub const fn stable_key(self) -> &'static str {
        self.stable_key
    }

    #[must_use]
    pub const fn version(self) -> &'static str {
        self.version
    }

    #[must_use]
    pub const fn payload_schema_digest(self) -> [u8; 32] {
        self.payload_schema_digest
    }

    #[must_use]
    pub const fn determinism_note(self) -> &'static str {
        self.determinism_note
    }

    #[must_use]
    pub const fn owner(self) -> &'static str {
        self.owner
    }

    pub fn execute(
        self,
        input: ActivityHandlerInput<'_>,
    ) -> Result<ActivityHandlerOutput, ActivityHandlerFault> {
        (self.handler)(input)
    }
}

impl core::fmt::Debug for ActivityHandlerRegistration {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ActivityHandlerRegistration")
            .field("id", &self.id)
            .field("stable_key", &self.stable_key)
            .field("version", &self.version)
            .field("payload_schema_digest", &self.payload_schema_digest)
            .field("determinism_note", &self.determinism_note)
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct ActivityHandlerBundle {
    id: &'static str,
    revision: &'static str,
    dependencies: Box<[&'static str]>,
    registrations: Box<[ActivityHandlerRegistration]>,
}

impl ActivityHandlerBundle {
    pub fn new(
        id: &'static str,
        revision: &'static str,
        mut dependencies: Vec<&'static str>,
        mut registrations: Vec<ActivityHandlerRegistration>,
    ) -> Result<Self, ActivityHandlerRegistryError> {
        validate_text(id)?;
        validate_text(revision)?;
        dependencies.sort_unstable();
        if dependencies.windows(2).any(|pair| pair[0] == pair[1]) || dependencies.contains(&id) {
            return Err(ActivityHandlerRegistryError::InvalidDependency);
        }
        for dependency in &dependencies {
            validate_text(dependency)?;
        }
        registrations.sort_unstable_by_key(|registration| registration.id);
        if registrations
            .windows(2)
            .any(|pair| pair[0].id == pair[1].id)
        {
            return Err(ActivityHandlerRegistryError::DuplicateHandler);
        }
        for registration in &registrations {
            validate_registration(*registration)?;
        }
        Ok(Self {
            id,
            revision,
            dependencies: dependencies.into_boxed_slice(),
            registrations: registrations.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> &'static str {
        self.id
    }

    #[must_use]
    pub const fn revision(&self) -> &'static str {
        self.revision
    }

    #[must_use]
    pub fn dependencies(&self) -> &[&'static str] {
        &self.dependencies
    }

    #[must_use]
    pub fn registrations(&self) -> &[ActivityHandlerRegistration] {
        &self.registrations
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActivityHandlerRegistryDigest([u8; 32]);

impl ActivityHandlerRegistryDigest {
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct ActivityHandlerRegistry {
    bundles: Box<[ActivityHandlerBundle]>,
    registrations: Box<[ActivityHandlerRegistration]>,
    digest: ActivityHandlerRegistryDigest,
}

impl ActivityHandlerRegistry {
    pub fn compose(
        mut bundles: Vec<ActivityHandlerBundle>,
    ) -> Result<Self, ActivityHandlerRegistryError> {
        if bundles.len() > MAX_ACTIVITY_HANDLER_BUNDLES {
            return Err(ActivityHandlerRegistryError::TooManyBundles);
        }
        bundles.sort_unstable_by_key(ActivityHandlerBundle::id);
        if bundles.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(ActivityHandlerRegistryError::DuplicateBundle);
        }
        for (index, bundle) in bundles.iter().enumerate() {
            for dependency in bundle.dependencies() {
                let dependency_index = bundles
                    .binary_search_by_key(dependency, ActivityHandlerBundle::id)
                    .map_err(|_| ActivityHandlerRegistryError::MissingDependency)?;
                if dependency_index >= index {
                    return Err(ActivityHandlerRegistryError::InvalidDependency);
                }
            }
        }
        let mut registrations = bundles
            .iter()
            .flat_map(|bundle| bundle.registrations().iter().copied())
            .collect::<Vec<_>>();
        if registrations.len() > MAX_ACTIVITY_HANDLERS {
            return Err(ActivityHandlerRegistryError::TooManyHandlers);
        }
        registrations.sort_unstable_by_key(|registration| registration.id);
        if registrations
            .windows(2)
            .any(|pair| pair[0].id == pair[1].id)
        {
            return Err(ActivityHandlerRegistryError::DuplicateHandler);
        }
        let digest = registry_digest(&bundles);
        Ok(Self {
            bundles: bundles.into_boxed_slice(),
            registrations: registrations.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub fn bundles(&self) -> &[ActivityHandlerBundle] {
        &self.bundles
    }

    #[must_use]
    pub const fn digest(&self) -> ActivityHandlerRegistryDigest {
        self.digest
    }

    #[must_use]
    pub fn handler(&self, id: ActivityHandlerId) -> Option<ActivityHandlerRegistration> {
        self.registrations
            .binary_search_by_key(&id, |registration| registration.id)
            .ok()
            .map(|index| self.registrations[index])
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActivityHandlerRegistryError {
    InvalidText,
    InvalidRegistration,
    DuplicateBundle,
    DuplicateHandler,
    MissingDependency,
    InvalidDependency,
    TooManyBundles,
    TooManyHandlers,
}

impl core::fmt::Display for ActivityHandlerRegistryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "activity handler registry error: {self:?}")
    }
}

impl std::error::Error for ActivityHandlerRegistryError {}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActivityHandlerFaultKind {
    PayloadTooLarge,
    InvalidPayload,
    InvalidState,
    Arithmetic,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityHandlerFault {
    kind: ActivityHandlerFaultKind,
}

impl ActivityHandlerFault {
    #[must_use]
    pub const fn new(kind: ActivityHandlerFaultKind) -> Self {
        Self { kind }
    }

    #[must_use]
    pub const fn kind(self) -> ActivityHandlerFaultKind {
        self.kind
    }
}

impl core::fmt::Display for ActivityHandlerFault {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "activity handler fault: {:?}", self.kind)
    }
}

impl std::error::Error for ActivityHandlerFault {}

fn validate_registration(
    registration: ActivityHandlerRegistration,
) -> Result<(), ActivityHandlerRegistryError> {
    validate_text(registration.stable_key)?;
    validate_text(registration.version)?;
    validate_text(registration.determinism_note)?;
    validate_text(registration.owner)?;
    if registration
        .payload_schema_digest
        .iter()
        .all(|byte| *byte == 0)
    {
        return Err(ActivityHandlerRegistryError::InvalidRegistration);
    }
    Ok(())
}

fn validate_text(value: &str) -> Result<(), ActivityHandlerRegistryError> {
    if value.is_empty() || value.len() > 128 || !value.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(ActivityHandlerRegistryError::InvalidText);
    }
    Ok(())
}

fn registry_digest(bundles: &[ActivityHandlerBundle]) -> ActivityHandlerRegistryDigest {
    let mut writer = ActivityRegistryWriter::new(b"starclock-activity-handler-registry-v1");
    writer.text(ACTIVITY_HANDLER_REGISTRY_REVISION);
    writer.u32(u32::try_from(bundles.len()).expect("bundle limit fits u32"));
    for bundle in bundles {
        writer.text(bundle.id);
        writer.text(bundle.revision);
        writer.u32(u32::try_from(bundle.dependencies.len()).expect("bundle limit fits u32"));
        for dependency in &bundle.dependencies {
            writer.text(dependency);
        }
        writer.u32(u32::try_from(bundle.registrations.len()).expect("handler limit fits u32"));
        for registration in &bundle.registrations {
            writer.u32(registration.id.get());
            writer.text(registration.stable_key);
            writer.text(registration.version);
            writer.digest(registration.payload_schema_digest);
            writer.text(registration.determinism_note);
            writer.text(registration.owner);
        }
    }
    ActivityHandlerRegistryDigest(writer.finish())
}
