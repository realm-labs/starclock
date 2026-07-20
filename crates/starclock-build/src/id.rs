//! Fixed-width build-domain identities.

macro_rules! build_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u32);

        impl $name {
            #[must_use]
            pub const fn new(raw: u32) -> Option<Self> {
                if raw == 0 { None } else { Some(Self(raw)) }
            }
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }
        }
    };
}

build_id!(TraceNodeId);
build_id!(EidolonDefinitionId);
build_id!(LightConeId);
