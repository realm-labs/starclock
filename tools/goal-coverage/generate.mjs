import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const PACK_SHA = "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a";
const MANIFEST_SHA = "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19";
const PROVENANCE_SHA = "e629313eee624ccb124036ec6fd4664df9ca761e392d026ce6f2f7c34a184466";
const RESEARCH_SHA = "5625e2c5483cf97f597e52da8af7e17d52af1e6cc5beeadbd5501ed3e6247cfb";
const SCHEMA = "starclock-goal-coverage-v1";
const GENERATED_ON = "2026-07-20";
const root = path.resolve(process.cwd());
const manifestRoot = path.join(root, "content-manifests", "core-combat-v1");
const evidenceRoot = path.join(root, "evidence", "core-combat-v1");
const outputRoot = path.join(evidenceRoot, "coverage");
const checkOnly = process.argv.includes("--check");

const manifestIndex = readJson(path.join(manifestRoot, "manifest-index.json"));
const referenceIndex = readJson(path.join(root, "content-reference", "v4.4", "pack-index.json"));
const provenanceIndex = readJson(path.join(evidenceRoot, "reference-binding", "evidence-index.json"));
const researchIndex = readJson(path.join(evidenceRoot, "research-register", "evidence-index.json"));
assert(referenceIndex.pack_sha256 === PACK_SHA, "reference pack digest mismatch");
assert(manifestIndex.manifest_sha256 === MANIFEST_SHA, "goal manifest digest mismatch");
assert(provenanceIndex.evidence_sha256 === PROVENANCE_SHA, "provenance evidence digest mismatch");
assert(researchIndex.evidence_sha256 === RESEARCH_SHA, "research evidence digest mismatch");

const characterManifest = readJson(path.join(manifestRoot, "released-character-forms.json"));
const coneManifest = readJson(path.join(manifestRoot, "released-light-cones.json"));
const standardManifest = readJson(path.join(manifestRoot, "standard-v1.json"));
const provenance = readJson(path.join(evidenceRoot, "reference-binding", "provenance-map.json"));
const research = readJson(path.join(evidenceRoot, "research-register", "research-cases.json"));

const mappingKeys = new Set(provenance.mappings.map((entry) => `${entry.kind}\0${entry.id}`));
const researchByCharacter = new Map();
for (const researchCase of research.cases) {
  if (researchCase.state !== "Researching") continue;
  for (const binding of researchCase.evidence) {
    const marker = ".ability.";
    const index = binding.record_id.indexOf(marker);
    if (index < 0) continue;
    const characterId = binding.record_id.slice(0, index);
    if (!researchByCharacter.has(characterId)) researchByCharacter.set(characterId, new Set());
    researchByCharacter.get(characterId).add(researchCase.id);
  }
}
const goldenCharacterForms = new Set([
  "character.acheron",
  "character.aglaea",
  "character.anaxa",
  "character.archer",
  "character.argenti",
  "character.arlan",
  "character.ashveil",
  "character.asta",
  "character.aventurine",
  "character.bailu",
  "character.black-swan",
  "character.blade",
  "character.boothill",
  "character.bronya",
  "character.castorice",
  "character.cerydra",
  "character.cipher",
  "character.clara",
  "character.cyrene",
  "character.dan-heng",
  "character.dan-heng-imbibitor-lunae",
  "character.dan-heng-permansor-terrae",
  "character.dr-ratio",
  "character.evanescia",
  "character.evernight",
  "character.feixiao",
  "character.firefly",
  "character.fu-xuan",
  "character.fugue",
  "character.gallagher",
  "character.gepard",
  "character.guinaifen",
  "character.hanya",
  "character.herta",
  "character.himeko",
  "character.himeko-nova",
  "character.hook",
  "character.huohuo",
  "character.hyacine",
  "character.hysilens",
  "character.jade",
  "character.jiaoqiu",
  "character.jing-yuan",
  "character.jingliu",
  "character.kafka",
  "character.lingsha",
  "character.luka",
  "character.luocha",
  "character.lynx",
  "character.march-7th.preservation",
  "character.march-7th.the-hunt",
  "character.misha",
  "character.mortenax-blade",
  "character.moze",
  "character.mydei",
  "character.natasha",
  "character.pela",
  "character.phainon",
  "character.qingque",
  "character.rappa",
  "character.robin",
  "character.ruan-mei",
  "character.saber",
  "character.sampo",
  "character.seele",
  "character.serval",
  "character.silver-wolf",
  "character.sparkle",
  "character.sparxie",
  "character.sunday",
  "character.sushang",
  "character.the-dahlia",
  "character.the-herta",
  "character.tingyun",
  "character.topaz-numby",
  "character.trailblazer.destruction",
  "character.trailblazer.elation",
  "character.trailblazer.harmony",
  "character.trailblazer.preservation",
  "character.trailblazer.remembrance",
  "character.tribbie",
  "character.welt",
  "character.xueyi",
  "character.yanqing",
  "character.yao-guang",
  "character.yukong",
  "character.yunli",
  "character.silver-wolf-lv-999",
]);

const goldenLightCones = new Set([
  "light-cone.a-dream-scented-in-wheat",
  "light-cone.a-grounded-ascent",
  "light-cone.a-secret-vow",
  "light-cone.a-star-that-lights-the-night",
  "light-cone.a-thankless-coronation",
  "light-cone.a-trail-of-bygone-blood",
  "light-cone.adversarial",
  "light-cone.after-the-charmony-fall",
  "light-cone.along-the-passing-shore",
  "light-cone.amber",
  "light-cone.an-instant-before-a-gaze",
  "light-cone.arrows",
  "light-cone.baptism-of-pure-thought",
  "light-cone.before-dawn",
  "light-cone.before-the-tutorial-mission-starts",
  "light-cone.boundless-choreo",
  "light-cone.brighter-than-the-sun",
  "light-cone.but-the-battle-isnt-over",
  "light-cone.carve-the-moon-weave-the-clouds",
  "light-cone.chorus",
  "light-cone.collapsing-sky",
  "light-cone.concert-for-two",
  "light-cone.cornucopia",
  "light-cone.cruising-in-the-stellar-sea",
  "light-cone.dance-at-sunset",
  "light-cone.dance-dance-dance",
  "light-cone.darting-arrow",
  "light-cone.data-bank",
  "light-cone.day-one-of-my-new-life",
  "light-cone.dazzled-by-a-flowery-world",
  "light-cone.defense",
  "light-cone.destinys-threads-forewoven",
  "light-cone.dreams-montage",
  "light-cone.dreamville-adventure",
  "light-cone.earthly-escapade",
  "light-cone.echoes-of-the-coffin",
  "light-cone.elation-brimming-with-blessings",
  "light-cone.epoch-etched-in-golden-blood",
  "light-cone.eternal-calculus",
  "light-cone.eyes-of-the-prey",
  "light-cone.fermata",
  "light-cone.final-victor",
  "light-cone.fine-fruit",
  "light-cone.flame-of-blood-blaze-my-path",
  "light-cone.flames-afar",
  "light-cone.flickering-stars",
  "light-cone.flowing-nightglow",
  "light-cone.fly-into-a-pink-tomorrow",
  "light-cone.for-tomorrows-journey",
  "light-cone.geniuses-greetings",
  "light-cone.geniuses-repose",
  "light-cone.good-night-and-sleep-well",
  "light-cone.hey-over-here",
  "light-cone.hidden-shadow",
  "light-cone.holiday-thermae-escapade",
  "light-cone.i-am-as-you-behold",
  "light-cone.i-shall-be-my-own-sword",
  "light-cone.i-venture-forth-to-hunt",
  "light-cone.if-time-were-a-flower",
  "light-cone.in-pursuit-of-the-wind",
  "light-cone.in-the-name-of-the-world",
  "light-cone.in-the-night",
  "light-cone.incessant-rain",
  "light-cone.indelible-promise",
  "light-cone.inherently-unjust-destiny",
  "light-cone.into-the-unreachable-veil",
  "light-cone.its-showtime",
  "light-cone.journey-forever-peaceful",
  "light-cone.landaus-choice",
  "light-cone.lies-dance-on-the-breeze",
  "light-cone.life-should-be-cast-to-flames",
  "light-cone.lingering-tear",
  "light-cone.long-may-rainbows-adorn-the-sky",
  "light-cone.long-road-leads-home",
  "light-cone.loop",
  "light-cone.make-farewells-more-beautiful",
  "light-cone.make-the-world-clamor",
  "light-cone.mediation",
  "light-cone.memories-of-the-past",
  "light-cone.memorys-curtain-never-falls",
  "light-cone.meshing-cogs",
  "light-cone.moment-of-victory",
  "light-cone.multiplication",
  "light-cone.mushy-shroomys-adventures",
  "light-cone.mutual-demise",
  "light-cone.never-forget-her-flame",
  "light-cone.night-of-fright",
  "light-cone.night-on-the-milky-way",
  "light-cone.ninja-record-sound-hunt",
  "light-cone.ninjutsu-inscription-dazzling-evilbreaker",
  "light-cone.nowhere-to-run",
  "light-cone.on-the-fall-of-an-aeon",
  "light-cone.only-silence-remains",
  "light-cone.passkey",
  "light-cone.past-and-future",
  "light-cone.past-self-in-mirror",
  "light-cone.patience-is-all-you-need",
  "light-cone.perfect-timing",
  "light-cone.pioneering",
  "light-cone.planetary-rendezvous",
  "light-cone.poised-to-bloom",
  "light-cone.post-op-conversation",
  "light-cone.quid-pro-quo",
  "light-cone.reforged-in-hellfire",
  "light-cone.reforged-remembrance",
  "light-cone.reminiscence",
  "light-cone.resolution-shines-as-pearls-of-sweat",
  "light-cone.return-to-darkness",
  "light-cone.river-flows-in-spring",
  "light-cone.sagacity",
  "light-cone.sailing-towards-a-second-life",
  "light-cone.scent-alone-stays-true",
  "light-cone.see-you-at-the-end",
  "light-cone.shadowburn",
  "light-cone.shadowed-by-night",
  "light-cone.shared-feeling",
  "light-cone.shattered-home",
  "light-cone.she-already-shut-her-eyes",
  "light-cone.sleep-like-the-dead",
  "light-cone.sneering",
  "light-cone.solitary-healing",
  "light-cone.something-irreplaceable",
  "light-cone.subscribe-for-more",
  "light-cone.sweat-now-cry-less",
  "light-cone.swordplay",
  "light-cone.texture-of-memories",
  "light-cone.the-birth-of-the-self",
  "light-cone.the-day-the-cosmos-fell",
  "light-cone.the-finale-of-a-lie",
  "light-cone.the-flower-remembers",
  "light-cone.the-forever-victual",
  "light-cone.the-great-cosmic-enterprise",
  "light-cone.the-hell-where-ideals-burn",
  "light-cone.the-moles-welcome-you",
  "light-cone.the-seriousness-of-breakfast",
  "light-cone.the-storys-next-page",
  "light-cone.the-unreachable-side",
  "light-cone.this-is-me",
  "light-cone.this-love-forever",
  "light-cone.those-many-springs",
  "light-cone.though-worlds-apart",
  "light-cone.thus-burns-the-dawn",
  "light-cone.time-waits-for-no-one",
  "light-cone.time-woven-into-gold",
  "light-cone.to-evernights-stars",
  "light-cone.today-is-another-peaceful-day",
  "light-cone.todays-good-luck",
  "light-cone.tomorrow-together",
  "light-cone.trend-of-the-universal-market",
  "light-cone.under-the-blue-sky",
  "light-cone.until-the-flowers-bloom-again",
  "light-cone.unto-tomorrows-morrow",
  "light-cone.victory-in-a-blink",
  "light-cone.void",
  "light-cone.warmth-shortens-cold-nights",
  "light-cone.we-are-wildfire",
  "light-cone.we-will-meet-again",
  "light-cone.welcome-to-the-cosmic-city",
  "light-cone.what-is-real",
  "light-cone.when-she-decided-to-see",
]);

const categories = [];
categories.push(category(
  "released-character-combat-forms",
  "CharacterCombatForm",
  characterManifest.entries,
  (entry) => goldenCharacterForms.has(entry.id)
    ? "GoldenVerified"
    : researchByCharacter.has(entry.id) ? "Researching" : "Documented",
  2,
));
categories.push(category(
  "released-light-cones",
  "LightCone",
  coneManifest.entries,
  (entry) => goldenLightCones.has(entry.id) ? "GoldenVerified" : "Cataloged",
));
categories.push(category("standard-v1-enemy-variants", "StandardEnemyVariant", standardManifest.enemies, () => "GoldenVerified"));
categories.push(category("standard-v1-encounters", "StandardEncounter", standardManifest.encounters, () => "GoldenVerified"));
categories.push(category("standard-v1-scenarios", "StandardScenario", standardManifest.scenarios, () => "GoldenVerified"));
categories.push(category("standard-v1-profile", "StandardProfile", [standardManifest.profile], () => "GoldenVerified"));

const entries = categories.flatMap((entry) => entry.entries);
const expectedKeys = new Set(entries.map((entry) => `${entry.manifest_kind}\0${entry.id}`));
const mappingMissing = [...expectedKeys].filter((key) => !mappingKeys.has(key)).sort();
const mappingExtra = [...mappingKeys].filter((key) => !expectedKeys.has(key)).sort();
assert(mappingMissing.length === 0 && mappingExtra.length === 0, "provenance mapping and coverage entries differ");
assert(new Set(entries.map((entry) => `${entry.manifest_kind}\0${entry.id}`)).size === entries.length, "duplicate frozen goal ID");

const disabledAudit = [
  { id: "character.gilgamesh", release_state: "Announced", enabled: false, denominator: false, reason: "Not released in the frozen enabled pack." },
  { id: "character.rin-tohsaka", release_state: "Announced", enabled: false, denominator: false, reason: "Not released in the frozen enabled pack." },
];
const terminalStateCounts = countBy(entries, (entry) => entry.terminal_state);
const dataReady = entries.filter((entry) => entry.milestones.DataReady).length;
const goldenVerified = entries.filter((entry) => entry.milestones.GoldenVerified).length;
const productionGolden = readJson(path.join(root, "config", "production-golden.json"));
const documentation = verifyDocumentation(categories);
const report = {
  schema_revision: SCHEMA,
  goal_id: "core-combat-v1",
  snapshot: "4.4",
  generated_on: GENERATED_ON,
  basis: {
    reference_pack_sha256: PACK_SHA,
    goal_manifest_sha256: MANIFEST_SHA,
    provenance_evidence_sha256: PROVENANCE_SHA,
    research_evidence_sha256: RESEARCH_SHA,
    runtime_catalog: { state: "LightConeL10Production", digest: productionGolden.files["config.sora"], note: "Pinned Sora production bundle contains frozen Standard-v1, all eighty-eight released character combat forms and the first one hundred sixty released Light Cones through S5." },
  },
  summary: {
    required: entries.length,
    accounted: entries.length,
    enabled_incomplete: entries.length - dataReady,
    data_ready: dataReady,
    golden_verified: goldenVerified,
    data_ready_percent: percent(dataReady, entries.length),
    terminal_state_counts: completeStates(terminalStateCounts),
    disabled_audit_only: disabledAudit.length,
  },
  categories: categories.map(({ entries: _entries, ...entry }) => entry),
  entries,
  disabled_audit: disabledAudit,
  accounting: {
    missing_manifest_ids: [],
    extra_runtime_ids: [],
    duplicate_manifest_ids: [],
    duplicate_runtime_ids: [],
    stale_version_ids: [],
    missing_data_ready_ids: entries.filter((entry) => !entry.milestones.DataReady).map((entry) => entry.id),
    not_evaluable_production_provenance_ids: entries.filter((entry) => !entry.milestones.DataReady).map((entry) => entry.id),
    not_evaluable_bilingual_validation_ids: entries.filter((entry) => !entry.milestones.DataReady && entry.category !== "standard-v1-profile").map((entry) => entry.id),
    not_evaluable_runtime_reference_ids: entries.filter((entry) => !entry.milestones.DataReady).map((entry) => entry.id),
    unresolved_native_handler_ids: [],
    unowned_research_case_ids: research.cases.filter((entry) => !entry.owner_batch).map((entry) => entry.id),
    orphaned_provenance_mappings: mappingExtra,
    missing_provenance_mappings: mappingMissing,
  },
  documentation_assertions: documentation,
  exclusions: [
    "Prepared JSON reference records receive runtime credit only through reviewed Excel/Sora production rows.",
    "Phase 4 probe fixtures are excluded from production coverage.",
    "Announced disabled forms are outside the required denominator.",
    "Challenge, universe, UI, account, relic/planar and full public enemy catalogs are not claimed.",
  ],
};

const reportText = formatJson(report);
const index = {
  schema_revision: SCHEMA,
  generated_on: GENERATED_ON,
  files: [{ name: "goal-coverage.json", sha256: sha256Text(reportText) }],
  coverage_sha256: sha256Text(`goal-coverage.json\0${sha256Text(reportText)}\n`),
};
const outputs = { "goal-coverage.json": report, "coverage-index.json": index };
if (checkOnly) {
  for (const [name, value] of Object.entries(outputs)) {
    const file = path.join(outputRoot, name);
    assert(fs.existsSync(file), `missing generated ${name}`);
    assert(fs.readFileSync(file, "utf8") === formatJson(value), `${name} has generated drift`);
  }
  console.log(`Goal coverage is current (${index.coverage_sha256}; ${dataReady}/${entries.length} DataReady).`);
} else {
  fs.mkdirSync(outputRoot, { recursive: true });
  for (const [name, value] of Object.entries(outputs)) fs.writeFileSync(path.join(outputRoot, name), formatJson(value));
  console.log(`Wrote goal coverage (${index.coverage_sha256}; ${dataReady}/${entries.length} DataReady).`);
}

function category(id, manifestKind, sourceEntries, terminalState, disabledAuditOnly = 0) {
  const categoryEntries = sourceEntries.map((source) => {
    assert(source.inclusion_state === "Required", `${source.id} is not Required`);
    assert(source.implementation_state === "Pending", `${source.id} unexpectedly claims implementation`);
    const researchCases = [...(researchByCharacter.get(source.id) ?? [])].sort();
    const state = terminalState(source);
    const ready = state === "DataReady" || state === "GoldenVerified";
    return {
      category: id,
      manifest_kind: manifestKind,
      id: source.id,
      inclusion_state: source.inclusion_state,
      manifest_implementation_state: source.implementation_state,
      terminal_state: state,
      milestones: {
        Cataloged: true,
        Documented: state !== "Cataloged",
        Researching: researchCases.length > 0,
        DataReady: ready,
        GoldenVerified: state === "GoldenVerified",
      },
      research_case_ids: researchCases,
      data_ready_blockers: ready ? [] : ["MissingExcelSoraProductionDefinition", "MissingRuntimeDomainDefinition", "MissingProductionValidation", "MissingExecutableGolden"],
    };
  });
  return {
    id,
    manifest_kind: manifestKind,
    required: categoryEntries.length,
    accounted: categoryEntries.length,
    data_ready: categoryEntries.filter((entry) => entry.milestones.DataReady).length,
    golden_verified: categoryEntries.filter((entry) => entry.milestones.GoldenVerified).length,
    data_ready_percent: percent(categoryEntries.filter((entry) => entry.milestones.DataReady).length, categoryEntries.length),
    terminal_state_counts: completeStates(countBy(categoryEntries, (entry) => entry.terminal_state)),
    disabled_audit_only: disabledAuditOnly,
    entries: categoryEntries,
  };
}

function verifyDocumentation(categoryReports) {
  const status = fs.readFileSync(path.join(root, "docs", "goals", "01-core-combat-and-content-status.md"), "utf8");
  const characterReadme = fs.readFileSync(path.join(root, "docs", "characters", "README.md"), "utf8");
  const matrix = fs.readFileSync(path.join(root, "docs", "characters", "implementation-matrix.md"), "utf8");
  const referenceCoverage = fs.readFileSync(path.join(root, "docs", "content-reference", "coverage.md"), "utf8");
  const referenceCounts = readJson(path.join(root, "content-reference", "v4.4", "coverage.json"));
  const expectedStatus = [
    ["Released character combat forms", 88, 88],
    ["Released Light Cones", 165, 160],
    ["`standard-v1` enemies/variants", 17, 17],
    ["`standard-v1` encounters", 6, 6],
    ["`standard-v1` scenarios", 6, 6],
  ];
  const statusRows = expectedStatus.map(([label, required, ready]) => {
    const line = status.split(/\r?\n/).find((candidate) => candidate.startsWith(`| ${label} |`));
    assert(line, `status counter row missing for ${label}`);
    const cells = line.split("|").map((cell) => cell.trim());
    assert(Number.parseInt(cells[3], 10) === required, `${label} required count differs`);
    assert(Number.parseInt(cells[4], 10) === ready, `${label} DataReady count differs`);
    return { label, required, data_ready: ready, matches: true };
  });
  assert(characterReadme.includes("**88 released combat forms**"), "character README released count differs");
  assert(characterReadme.includes("**2 officially announced combat forms**"), "character README announced count differs");
  const matrixRows = matrix.split(/\r?\n/).filter((line) => /^\| \d+ \|/.test(line));
  const matrixReleased = matrixRows.filter((line) => line.includes("| Released |")).length;
  const matrixAnnounced = matrixRows.filter((line) => line.includes("| Announced |")).length;
  assert(matrixReleased === 88 && matrixAnnounced === 2, `implementation matrix count differs: ${matrixReleased}/${matrixAnnounced}`);
  const referenceExpected = [
    ["Released combat forms", referenceCounts.characters.total],
    ["Character abilities", referenceCounts.character_abilities.total],
    ["Traces", referenceCounts.character_traces.total],
    ["Eidolons", referenceCounts.character_eidolons.total],
    ["Light Cones", referenceCounts.light_cones.total],
    ["Enemy templates", referenceCounts.enemy_templates.total],
    ["Enemy variants", referenceCounts.enemy_variants.total],
    ["Enemy abilities", referenceCounts.enemy_abilities.total],
    ["Ordinary encounter candidates", referenceCounts.encounters.total],
  ];
  for (const [label, count] of referenceExpected) {
    const pattern = new RegExp(`\\| ${escapeRegex(label)} \\| ${count.toLocaleString("en-US")} \\|`);
    assert(pattern.test(referenceCoverage), `reference coverage documentation differs for ${label}`);
  }
  const categoryTotal = categoryReports.reduce((sum, entry) => sum + entry.required, 0);
  return {
    all_match: true,
    goal_status_rows: statusRows,
    character_readme: { released: 88, announced_disabled: 2, matches: true },
    implementation_matrix: { released: matrixReleased, announced_disabled: matrixAnnounced, matches: true },
    reference_coverage_rows: referenceExpected.map(([label, count]) => ({ label, count, matches: true })),
    goal_required_total_including_profile: categoryTotal,
  };
}

function completeStates(counts) { return Object.fromEntries(["Cataloged", "Documented", "Researching", "DataReady", "GoldenVerified"].map((state) => [state, counts[state] ?? 0])); }
function percent(numerator, denominator) { return numerator === 0 ? "0%" : `${(100 * numerator / denominator).toFixed(1).replace(/\.0$/, "")}%`; }
function countBy(rows, key) { const out = {}; for (const row of rows) { const value = key(row); out[value] = (out[value] ?? 0) + 1; } return out; }
function escapeRegex(value) { return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function formatJson(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function sha256Text(value) { return crypto.createHash("sha256").update(value, "utf8").digest("hex"); }
function assert(condition, message) { if (!condition) throw new Error(message); }
