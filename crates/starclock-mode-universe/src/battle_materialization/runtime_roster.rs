use super::*;

impl UniverseBattleRoster {
    /// Binds exact compiler inputs while retaining a deterministic
    /// runtime-only base-stat envelope for smoke and integration fixtures.
    pub fn new_with_build_specs_and_runtime_stats(
        lock: &ParticipantLock,
        combatants: Vec<(
            ParticipantId,
            starclock_build::spec::CombatantBuildSpec,
            ResolvedCombatantSpec,
            ResolvedCombatantSpec,
        )>,
    ) -> Result<Self, UniverseBattleMaterializationError> {
        if combatants.iter().any(|(_, build, compiled, runtime)| {
            build.form() != compiled.form() || compiled.form() != runtime.form()
        }) {
            return Err(UniverseBattleMaterializationError::RosterMismatch);
        }
        let combatants = combatants
            .into_iter()
            .map(|(participant, build, compiled, runtime)| {
                (
                    participant,
                    runtime,
                    Some(build),
                    Some(compiled.digest()),
                    true,
                )
            })
            .collect();
        Self::new_inner(lock, combatants)
    }
}
