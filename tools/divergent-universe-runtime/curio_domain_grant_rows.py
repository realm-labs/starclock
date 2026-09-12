"""Exact domain-entry income with independently replaceable lifecycle timing."""


def append_curio_domain_grants(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (55, "state", "ExcelOutput/RogueTournMiracle.json; MiracleID=9071; TournMode=Tourn3; HandbookMiracleID=9068; MiracleEffectID=2071",
         "176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438"),
        (56, "parameters", "ExcelOutput/RogueMiracleEffect.json; MiracleEffectID=2071; ParamList[0]=60; ParamList[1]=3",
         "fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35"),
        (57, "text", "TextMap/TextMapEN.json; hash=3761031791427163166; domain-entry fragments and three-domain discard",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.curio-domain-grant.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-12",
            locator, digest, "ExactStructured",
            "Exact released state, 60-fragment amount and three-entry limit. Activation, expiry ordering and destroyed-state counting are separately authored policies.",
        ])
    data["CurioDomainExpiries"].append([
        3, "du.curio-domain-expiry.9071", "divergent-universe.curio-state.9071", "2071", 2, 3,
        "ActiveFutureSelectedDomainsGrantThenDiscard",
        "Discard after the third future domain-entry grant.", "第三次后续领域入场奖励发放后丢弃。",
        "VersionedProjectPolicy: initialize three remaining entries on acquisition without counting its already entered domain. At an accepted future public domain selection, generate the intrinsic grant from the active positive-allowance pre-entry holding, then consume one allowance and discard all counters at zero before encounter construction and other new-domain rewards. Destroyed holdings neither grant nor count; repair resumes; replacement/reacquisition refill. Trusted zero allowance grants nothing and is removed at the next entry. Internal graph nodes, service choices, rejected and duplicate commands never count. Grant, discard, route and encounter RNG share one rollback boundary.",
        "Low parity confidence for acquisition-domain counting, pre-entry grant/expiry order, destroyed-state time and route placement. Alternatives include counting the acquisition domain, discarding before its final intrinsic grant and counting while destroyed. Replace independently when released execution or reproducible observations establish them. Keep exact three domains; the current public route offers only two future entries, not full exhaustion.",
        "55|56|57",
    ])
    data["CurioDomainGrants"] = [[
        1, "du.curio-domain-grant.9071", 3, 1, 60,
        "ActivePositiveAllowanceFragmentsBeforeDiscard",
        "Gain 60 fragments on each eligible domain entry.", "每次符合条件的领域入场获得 60 碎片。",
        "VersionedProjectPolicy: select active positive-allowance holdings from the authenticated pre-entry view, in stable state-key order. Emit each exact fragment grant through the shared nonrecursive credit pipeline before any entry allowance consumption. Active global fragment modifiers apply using their existing original-base independently-floored policy. The third eligible entry grants before discarding. No grant on acquisition, internal nodes, battles, destroyed/zero holdings or rejected/duplicate choices. Credit overflow or later traversal failure rolls back grants, allowance, route and RNG together.",
        "Low confidence for intrinsic-grant ordering, acquisition timing, gain-modifier interaction and placement, not for exact 60. Alternatives include final-entry suppression, post-expiry modifier snapshots and exempting triggered income from gain bonuses. Replace fields independently when released execution or reproducible observations establish them. Public Tawot Cards text cross-checks operands but asks for Arcadian Chronicles verification; it is not exact execution evidence. Require real public domain choices, paid acquisition/replay, three-entry operation vectors and rollback tests.",
        "55|56|57",
    ]]
