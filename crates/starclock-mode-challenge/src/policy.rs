/// Review confidence attached to a replaceable inferred runtime behavior.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PolicyConfidence {
    Low,
    Medium,
    High,
}

/// Explicit non-parity behavior used only when released evidence is incomplete.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectPolicy {
    pub id: Box<str>,
    pub known_facts: Box<str>,
    pub selected_behavior: Box<str>,
    pub rejected_alternatives: Box<[Box<str>]>,
    pub rationale: Box<str>,
    pub affected_tests: Box<[Box<str>]>,
    pub confidence: PolicyConfidence,
    pub replacement_condition: Box<str>,
}
