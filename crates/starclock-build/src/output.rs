//! Successful generic combat compilation output.

use starclock_combat::ResolvedCombatantSpec;

use crate::report::BuildCompilationReport;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledBuild {
    combatant: ResolvedCombatantSpec,
    report: BuildCompilationReport,
}

impl CompiledBuild {
    pub(crate) const fn new(
        combatant: ResolvedCombatantSpec,
        report: BuildCompilationReport,
    ) -> Self {
        Self { combatant, report }
    }
    #[must_use]
    pub const fn combatant(&self) -> &ResolvedCombatantSpec {
        &self.combatant
    }
    #[must_use]
    pub const fn report(&self) -> &BuildCompilationReport {
        &self.report
    }
}
