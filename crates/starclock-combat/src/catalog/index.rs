//! Immutable trigger-definition indexes compiled once with the catalog.

use std::collections::BTreeMap;

use crate::{
    RuleId, TriggerId,
    rule::model::{RuleEventKind, TriggerPhase},
};

use super::{definition::RuleDefinition, table::DefinitionTable};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TriggerIndexEntry {
    pub(super) rule: RuleId,
    pub(super) trigger: TriggerId,
}

#[derive(Debug, Default)]
pub(super) struct TriggerDefinitionIndex {
    groups: BTreeMap<(RuleEventKind, TriggerPhase), Box<[TriggerIndexEntry]>>,
}

impl TriggerDefinitionIndex {
    pub(super) fn compile(rules: &DefinitionTable<RuleId, RuleDefinition>) -> Self {
        let mut groups = BTreeMap::<_, Vec<_>>::new();
        for rule_id in rules.ids() {
            let rule = rules.get(rule_id).expect("ID came from table");
            let Some(runtime) = rule.runtime() else {
                continue;
            };
            for trigger in runtime.triggers() {
                groups
                    .entry((trigger.event, trigger.phase))
                    .or_default()
                    .push((
                        crate::rule::evaluate::trigger_definition_order(
                            rule_id,
                            runtime.source().definition(),
                            trigger,
                        ),
                        TriggerIndexEntry {
                            rule: rule_id,
                            trigger: trigger.id,
                        },
                    ));
            }
        }
        Self {
            groups: groups
                .into_iter()
                .map(|(key, mut entries)| {
                    entries.sort_unstable_by_key(|(order, _)| *order);
                    (
                        key,
                        entries
                            .into_iter()
                            .map(|(_, entry)| entry)
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                    )
                })
                .collect(),
        }
    }

    pub(super) fn get(&self, event: RuleEventKind, phase: TriggerPhase) -> &[TriggerIndexEntry] {
        self.groups.get(&(event, phase)).map_or(&[], AsRef::as_ref)
    }

    pub(super) fn len(&self) -> usize {
        self.groups.values().map(|entries| entries.len()).sum()
    }
}
