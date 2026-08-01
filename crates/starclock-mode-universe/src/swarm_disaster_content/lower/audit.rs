use std::collections::BTreeSet;

use crate::swarm_disaster_generated::{
    SoraConfig, swarm_disaster_coverage::SwarmDisasterCoverage,
    swarm_disaster_coverage_state::SwarmDisasterCoverageState,
    swarm_disaster_evidence_grade::SwarmDisasterEvidenceGrade,
    swarm_disaster_mechanic_rule::SwarmDisasterMechanicRule,
    swarm_disaster_ownership::SwarmDisasterOwnership,
    swarm_disaster_pack_index::SwarmDisasterPackIndex,
    swarm_disaster_reconcile_receipt::SwarmDisasterReconcileReceipt,
    swarm_disaster_research_gap::SwarmDisasterResearchGap,
    swarm_disaster_research_gap_affected::SwarmDisasterResearchGapAffected,
    swarm_disaster_review_fixture::SwarmDisasterReviewFixture,
    swarm_disaster_source_record::SwarmDisasterSourceRecord,
};

use super::{fail, json, metadata, nonempty, positive, stable, text_list};
use crate::swarm_disaster_content::{
    SwarmDisasterContentError, SwarmDisasterContentErrorKind, types::*,
};

type LoweredAudit = (
    Box<[MechanicRuleDefinition]>,
    Box<[ReviewFixtureDefinition]>,
    AuditCatalogSummary,
);

pub(super) fn lower(source: &SoraConfig) -> Result<LoweredAudit, SwarmDisasterContentError> {
    let mechanic_rules = source
        .swarm_disaster_mechanic_rule()
        .ordered_rows()
        .map(mechanic_rule)
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    let source_keys = validate_sources(source)?;
    validate_coverage(source, &source_keys)?;
    validate_gaps(source)?;
    validate_fixtures(source)?;
    let review_fixtures = source
        .swarm_disaster_review_fixture()
        .ordered_rows()
        .map(review_fixture)
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    validate_receipts(source)?;
    let manifest = source
        .swarm_disaster_manifest()
        .ordered_rows()
        .next()
        .ok_or_else(|| super::error(SwarmDisasterContentErrorKind::Denominator, "manifest"))?;
    if source.swarm_disaster_manifest().len() != 1
        || manifest.schema_revision != "starclock.swarm-disaster-pack-manifest.v1"
        || manifest.stable_key != "swarm-disaster.pack-manifest.v1"
        || manifest.goal_id != "swarm-disaster-reference-v1"
        || manifest.profile_id != "swarm-disaster.profile.v1"
        || manifest.frozen_source_obligations != 6_963
        || manifest.data_ready_source_obligations != 6_963
        || manifest.mechanic_rule_count != 23
        || manifest.semantic_fixture_family_count != 23
        || manifest.research_gap_count != 31
        || manifest.blocking_research_gap_count != 0
        || manifest.reconciliation_receipt_count != 609
        || manifest.runtime_loading != "ForbiddenReferenceOnly"
        || !manifest.candidate_quality
    {
        return fail(SwarmDisasterContentErrorKind::Metadata, "manifest");
    }
    json(&manifest.snapshot_json, &manifest.stable_key)?;
    validate_pack_index(source)?;
    Ok((
        mechanic_rules,
        review_fixtures,
        AuditCatalogSummary {
            source_records: source.swarm_disaster_source_record().len(),
            coverage_rows: source.swarm_disaster_coverage().len(),
            research_gaps: source.swarm_disaster_research_gap().len(),
            affected_rows: source.swarm_disaster_research_gap_affected().len(),
            fixtures: source.swarm_disaster_review_fixture().len(),
            receipts: source.swarm_disaster_reconcile_receipt().len(),
            manifest_rows: source.swarm_disaster_manifest().len(),
            pack_rows: source.swarm_disaster_pack_index().len(),
            frozen_obligations: u32::try_from(manifest.frozen_source_obligations)
                .map_err(|_| super::error(SwarmDisasterContentErrorKind::Identifier, "manifest"))?,
            mechanic_rules: u16::try_from(manifest.mechanic_rule_count)
                .map_err(|_| super::error(SwarmDisasterContentErrorKind::Identifier, "manifest"))?,
            fixture_families: u16::try_from(manifest.semantic_fixture_family_count)
                .map_err(|_| super::error(SwarmDisasterContentErrorKind::Identifier, "manifest"))?,
        },
    ))
}

fn review_fixture(
    row: &SwarmDisasterReviewFixture,
) -> Result<ReviewFixtureDefinition, SwarmDisasterContentError> {
    let quality = match row.fixture_evidence_quality {
        SwarmDisasterEvidenceGrade::ExactStructured => ReviewFixtureQuality::ExactStructured,
        SwarmDisasterEvidenceGrade::ProjectPolicy => ReviewFixtureQuality::ProjectPolicy,
        _ => return fail(SwarmDisasterContentErrorKind::Metadata, &row.stable_key),
    };
    if row.ownership != SwarmDisasterOwnership::SwarmDisaster
        || row.coverage_state != SwarmDisasterCoverageState::DataReady
        || row.evidence_quality != row.fixture_evidence_quality
    {
        return fail(SwarmDisasterContentErrorKind::Metadata, &row.stable_key);
    }
    Ok(ReviewFixtureDefinition {
        key: stable(&row.stable_key, &row.stable_key)?,
        family: stable(&row.family_id, &row.stable_key)?,
        source_record_keys: text_list(&row.source_record_ids, &row.stable_key)?,
        preconditions: json(&row.preconditions_json, &row.stable_key)?,
        input: json(&row.input_json, &row.stable_key)?,
        ordered_operations: json(&row.ordered_operations_json, &row.stable_key)?,
        expected_facts: json(&row.expected_facts_json, &row.stable_key)?,
        quality,
    })
}

fn mechanic_rule(
    row: &SwarmDisasterMechanicRule,
) -> Result<MechanicRuleDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.execution_disposition != "ReferenceOnly" || row.runtime_handler_id.is_some() {
        return fail(SwarmDisasterContentErrorKind::Metadata, &row.stable_key);
    }
    Ok(MechanicRuleDefinition {
        id: MechanicRuleId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        family_key: stable(&row.family_id, &row.stable_key)?,
        domain: nonempty(&row.domain, &row.stable_key)?,
        triggers: text_list(&row.triggers, &row.stable_key)?,
        slots: json(&row.state_slots_json, &row.stable_key)?,
        program: json(&row.program_json, &row.stable_key)?,
        fixture_keys: text_list(&row.fixture_ids, &row.stable_key)?,
        disposition: row.execution_disposition.as_str().into(),
    })
}

fn validate_sources(source: &SoraConfig) -> Result<BTreeSet<&str>, SwarmDisasterContentError> {
    if source.swarm_disaster_source_record().len() != 8_139 {
        return fail(SwarmDisasterContentErrorKind::Denominator, "source-records");
    }
    let mut keys = BTreeSet::new();
    let mut source_ids = BTreeSet::new();
    for row in source.swarm_disaster_source_record().ordered_rows() {
        source_metadata(row)?;
        if !keys.insert(row.stable_key.as_str()) || !source_ids.insert(row.source_id.as_str()) {
            return fail(SwarmDisasterContentErrorKind::Duplicate, &row.stable_key);
        }
    }
    Ok(source_ids)
}

fn source_metadata(row: &SwarmDisasterSourceRecord) -> Result<(), SwarmDisasterContentError> {
    stable(&row.stable_key, &row.stable_key)?;
    if row.schema_revision != "starclock.swarm-disaster-source.v1" || row.kind != "SourceRecord" {
        return fail(SwarmDisasterContentErrorKind::Metadata, &row.stable_key);
    }
    for value in [
        &row.source_id,
        &row.source_kind,
        &row.repository,
        &row.revision,
        &row.game_version,
        &row.path,
        &row.locator,
        &row.access_date,
    ] {
        nonempty(value, &row.stable_key)?;
    }
    sha256(&row.sha256, &row.stable_key)
}

fn validate_coverage(
    source: &SoraConfig,
    source_ids: &BTreeSet<&str>,
) -> Result<(), SwarmDisasterContentError> {
    if source.swarm_disaster_coverage().len() != 6_963 {
        return fail(SwarmDisasterContentErrorKind::Denominator, "coverage");
    }
    let mut records = BTreeSet::new();
    for row in source.swarm_disaster_coverage().ordered_rows() {
        coverage_metadata(row)?;
        if !records.insert((
            row.manifest_category.as_str(),
            row.manifest_record_id.as_str(),
        )) || row.source_refs.as_deref().is_none_or(|refs| {
            refs.is_empty() || refs.iter().any(|key| !source_ids.contains(key.as_str()))
        }) {
            return fail(SwarmDisasterContentErrorKind::Reference, &row.stable_key);
        }
    }
    Ok(())
}

fn coverage_metadata(row: &SwarmDisasterCoverage) -> Result<(), SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    nonempty(&row.manifest_category, &row.stable_key)?;
    nonempty(&row.manifest_record_id, &row.stable_key)?;
    nonempty(&row.source_locator, &row.stable_key)?;
    sha256(&row.source_evidence_sha256, &row.stable_key)?;
    json(&row.normalized_refs_json, &row.stable_key)?;
    if row
        .blocking_gap_ids
        .as_ref()
        .is_some_and(|keys| !keys.is_empty())
    {
        return fail(SwarmDisasterContentErrorKind::Reference, &row.stable_key);
    }
    Ok(())
}

fn validate_gaps(source: &SoraConfig) -> Result<(), SwarmDisasterContentError> {
    if source.swarm_disaster_research_gap().len() != 31
        || source.swarm_disaster_research_gap_affected().len() != 5_560
    {
        return fail(SwarmDisasterContentErrorKind::Denominator, "research-gaps");
    }
    let mut gap_ids = BTreeSet::new();
    for row in source.swarm_disaster_research_gap().ordered_rows() {
        research_gap_metadata(row)?;
        gap_ids.insert(positive(row.id, &row.stable_key)?);
    }
    let mut order = BTreeSet::new();
    for row in source.swarm_disaster_research_gap_affected().ordered_rows() {
        affected_metadata(row, &gap_ids)?;
        if !order.insert((row.research_gap_id, row.ordinal)) {
            return fail(
                SwarmDisasterContentErrorKind::Ordering,
                &row.record_stable_key,
            );
        }
    }
    Ok(())
}

fn research_gap_metadata(row: &SwarmDisasterResearchGap) -> Result<(), SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.blocking {
        return fail(SwarmDisasterContentErrorKind::Metadata, &row.stable_key);
    }
    for value in [
        &row.state,
        &row.gap_state,
        &row.field,
        &row.policy_source_id,
        &row.known_facts,
        &row.selected_policy,
        &row.rationale,
        &row.confidence,
        &row.replacement_condition,
    ] {
        nonempty(value, &row.stable_key)?;
    }
    Ok(())
}

fn affected_metadata(
    row: &SwarmDisasterResearchGapAffected,
    gap_ids: &BTreeSet<u32>,
) -> Result<(), SwarmDisasterContentError> {
    positive(row.id, &row.record_stable_key)?;
    let gap = positive(row.research_gap_id, &row.record_stable_key)?;
    if !gap_ids.contains(&gap) || row.ordinal < 0 {
        return fail(
            SwarmDisasterContentErrorKind::Reference,
            &row.record_stable_key,
        );
    }
    nonempty(&row.file, &row.record_stable_key)?;
    stable(&row.record_stable_key, &row.record_stable_key)?;
    Ok(())
}

fn validate_fixtures(source: &SoraConfig) -> Result<(), SwarmDisasterContentError> {
    if source.swarm_disaster_review_fixture().len() != 23 {
        return fail(SwarmDisasterContentErrorKind::Denominator, "fixtures");
    }
    let mut families = BTreeSet::new();
    for row in source.swarm_disaster_review_fixture().ordered_rows() {
        fixture_metadata(row)?;
        if !families.insert(row.family_id.as_str()) {
            return fail(SwarmDisasterContentErrorKind::Duplicate, &row.stable_key);
        }
    }
    Ok(())
}

fn fixture_metadata(row: &SwarmDisasterReviewFixture) -> Result<(), SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    stable(&row.family_id, &row.stable_key)?;
    if row.source_record_ids.is_empty() || row.evidence_refs.is_empty() {
        return fail(SwarmDisasterContentErrorKind::Reference, &row.stable_key);
    }
    for value in [
        &row.preconditions_json,
        &row.input_json,
        &row.ordered_operations_json,
        &row.expected_facts_json,
    ] {
        json(value, &row.stable_key)?;
    }
    Ok(())
}

fn validate_receipts(source: &SoraConfig) -> Result<(), SwarmDisasterContentError> {
    if source.swarm_disaster_reconcile_receipt().len() != 609 {
        return fail(SwarmDisasterContentErrorKind::Denominator, "receipts");
    }
    for row in source.swarm_disaster_reconcile_receipt().ordered_rows() {
        receipt_metadata(row)?;
    }
    Ok(())
}

fn receipt_metadata(row: &SwarmDisasterReconcileReceipt) -> Result<(), SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    sha256(&row.evidence_sha256, &row.stable_key)?;
    for value in [
        &row.source_path,
        &row.row_locator,
        &row.swarm_category,
        &row.swarm_record_id,
        &row.goal08_category,
        &row.goal08_record_id,
        &row.goal08_commit,
        &row.outcome,
    ] {
        nonempty(value, &row.stable_key)?;
    }
    Ok(())
}

fn validate_pack_index(source: &SoraConfig) -> Result<(), SwarmDisasterContentError> {
    if source.swarm_disaster_pack_index().len() != 63 {
        return fail(SwarmDisasterContentErrorKind::Denominator, "pack-index");
    }
    let mut files = BTreeSet::new();
    let mut pack_digest = None;
    for row in source.swarm_disaster_pack_index().ordered_rows() {
        pack_metadata(row)?;
        if !files.insert(row.file.as_str())
            || pack_digest
                .replace(row.pack_sha256.as_str())
                .is_some_and(|old| old != row.pack_sha256)
        {
            return fail(SwarmDisasterContentErrorKind::Duplicate, &row.stable_key);
        }
    }
    Ok(())
}

fn pack_metadata(row: &SwarmDisasterPackIndex) -> Result<(), SwarmDisasterContentError> {
    if row.schema_revision != "starclock.swarm-disaster-pack-index.v1"
        || row.bytes <= 0
        || row.rows < 0
    {
        return fail(SwarmDisasterContentErrorKind::Metadata, &row.stable_key);
    }
    stable(&row.stable_key, &row.stable_key)?;
    nonempty(&row.file, &row.stable_key)?;
    sha256(&row.sha256, &row.stable_key)?;
    sha256(&row.pack_sha256, &row.stable_key)
}

fn sha256(value: &str, key: &str) -> Result<(), SwarmDisasterContentError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return fail(SwarmDisasterContentErrorKind::Metadata, key);
    }
    Ok(())
}
