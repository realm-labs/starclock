//! Stable Universe catalog loading failures.

/// Stable failure family at the generated-row-free loading boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniverseCatalogLoadErrorKind {
    UniverseBundleDigest,
    BundleFormat,
    UniverseRevision,
    CoreCompatibility,
    Coverage,
}

/// Catalog loading error without generated Sora row types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseCatalogLoadError {
    kind: UniverseCatalogLoadErrorKind,
    message: Box<str>,
}

impl UniverseCatalogLoadError {
    pub(crate) fn new(kind: UniverseCatalogLoadErrorKind, message: impl Into<Box<str>>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> UniverseCatalogLoadErrorKind {
        self.kind
    }
}

impl std::fmt::Display for UniverseCatalogLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for UniverseCatalogLoadError {}
