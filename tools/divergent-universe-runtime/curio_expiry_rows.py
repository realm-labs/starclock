"""Exact entry limits with separately authored timing/lifecycle policy."""


def append_curio_expiries(data: dict[str, list[list[object]]]) -> None:
    data["Sources"].append([
        42, "du.source.curio-domain-expiry.parameters",
        "https://gitlab.com/Dimbreath/turnbasedgamedata",
        "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-12",
        "ExcelOutput/RogueMiracleEffect.json; MiracleEffectID=2070,2079; ParamList[1]=3,5; MiracleDesc.Hash=7205940712072507551,3429135723207096768",
        "fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35",
        "ExactStructured",
        "Released parameters and EN descriptions (source 14) establish discard after 3/5 domain entries. Acquisition-domain counting, destroyed-state time and within-entry ordering are not specified.",
    ])
    data["CurioDomainExpiries"] = [
        [ordinal, f"du.curio-domain-expiry.{state}",
         f"divergent-universe.curio-state.{state}", str(effect), 2, limit,
         "ActiveFutureSelectedDomainsDiscard",
         f"Discard after entering {limit} domains.", f"累计进入 {limit} 个领域后丢弃。",
         "VersionedProjectPolicy: acquisition initializes remaining entries from the exact parameter. The already entered acquisition domain does not count. Each accepted later public domain selection consumes one entry before encounter generation or new-domain rewards. Pre-selection layer services do not count. On the limiting entry, remove the holding and all lifecycle counters, rather than marking it destroyed. Only active holdings count; destruction pauses, repair resumes without refilling, and replacement or reacquisition starts a fresh allowance. Rejected/duplicate decisions consume nothing and all entry effects, route state and encounter RNG roll back together. Current one-domain-per-layer route only; no inference of original multi-room topology.",
         "Timing and destroyed-state confidence is low. Alternatives include counting the acquisition domain, continuing during destruction, and expiring after entry rewards. Replace these fields independently when released execution programs or reproducible observations establish them. Keep exact 3/5 limits and discard semantics. Bind any future domain route to this entry boundary; do not count physical graph nodes.",
         "12|14|42"]
        for ordinal, state, effect, limit in [(1, 9070, 2070, 3), (2, 9079, 2079, 5)]
    ]
