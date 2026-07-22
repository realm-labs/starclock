use starclock_agent_api::{action, error, observation, schema, session};

#[test]
fn facade_exposes_five_responsibility_modules_without_protocol_types() {
    assert_eq!(
        schema::RESPONSIBILITY,
        "schema revisions and exact agent values"
    );
    assert_eq!(
        observation::RESPONSIBILITY,
        "owned visibility-controlled projections"
    );
    assert_eq!(
        action::RESPONSIBILITY,
        "opaque offered actions and exact command bindings"
    );
    assert_eq!(
        session::RESPONSIBILITY,
        "ephemeral authoritative sessions and registry"
    );
    assert_eq!(error::RESPONSIBILITY, "stable protocol-neutral failures");
}
