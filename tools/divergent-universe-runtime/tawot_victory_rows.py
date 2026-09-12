"""Released victory count; pool and domain-lifetime interaction stay explicit policy."""


def append_tawot_victory(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (58, "state", "ExcelOutput/RogueTournMiracle.json; MiracleID=9072; TournMode=Tourn3; HandbookMiracleID=9068; MiracleEffectID=2072",
         "176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438"),
        (59, "parameters", "ExcelOutput/RogueMiracleEffect.json; MiracleEffectID=2072; ParamList[0]=1; ParamList[1]=3",
         "fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35"),
        (60, "text", "TextMap/TextMapEN.json; hash=3816091537220261218; victory random Blessing and three-domain discard",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.tawot-victory.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-12",
            locator, digest, "ExactStructured",
            "Exact current state/effect join, one random victory Blessing and three-domain limit. Rarity pool, weights, suppression and within-entry timing remain policy.",
        ])
    data["CurioDomainExpiries"].append([
        4, "du.curio-domain-expiry.9072", "divergent-universe.curio-state.9072", "2072", 2, 3,
        "ActiveFutureSelectedDomainsDiscard",
        "Discard after three future domain entries.", "进入三个后续领域后丢弃。",
        "VersionedProjectPolicy: acquisition initializes three remaining future domain entries without counting the current acquisition domain. Only active holdings consume one allowance at an accepted later domain selection; discard all holding counters on the limiting entry before encounter construction or battle rewards. Internal nodes and battle results never consume this domain counter. Destruction pauses; repair resumes; replacement/reacquisition refill. A trusted zero allowance contributes no victory grant and is removed at the next entry. Domain marker, discard and encounter RNG share one atomic transaction.",
        "Low confidence for acquisition-domain inclusion, discard-before-new-domain-rewards and destroyed-state counting. Alternatives include counting the current domain, final-domain rewards before discard and counting while destroyed. Replace independently when released execution or reproducible observations establish them; retain exact three domains. Current baseline has only two future entries after initial service acquisition, not full public exhaustion.",
        "58|59|60",
    ])
    data["CurioVictoryBlessings"].append([
        4, "du.curio-victory-blessings.9072", "divergent-universe.curio-state.9072", "2072",
        1, 1, 1, 3, "Combat|Elite|Aberration|Boss", "PositiveDomainAllowanceUniformUnownedAvailableSubset",
        "After each won battle while active with positive domain allowance, gain one random Blessing.",
        "生效且剩余领域次数为正时，每次战斗胜利获得一个随机祝福。",
        "VersionedProjectPolicy: require a verified victory and the active current holding with positive domain allowance. Admit all current mode-owned battle-domain labels, including Combat/Elite/Aberration/Boss; labels do not establish original room topology. Sample one unowned current base Blessing uniformly from the policy-selected 1-3-star pool, not an exact source rarity range. Use existing Reward purpose 24051 and stable state/identity ordering without replacement across all simultaneous Curio victory grants. Enhanced holdings exclude their base identity. Apply the same active suppression states as ordinary post-battle Blessings; do not apply sealing-wax offer weights. Empty pools grant/draw nothing. Commit holdings and Equation refresh before ordinary offer generation with verified carry, later stages and all RNG in one rollback boundary. Loss/fault/rejected/duplicate results never grant. Battle settlement does not consume the domain allowance.",
        "Low confidence for pool/rarity, unit weights, suppression scope, grant ordering and exhausted-pool behavior. Alternatives include fixed or weighted rarity pools, duplicate conversion, unsuppressed intrinsic rewards and once-per-domain timing. Replace each field independently when released programs or reproducible observations establish it; retain exact one random victory Blessing and three-domain limit. Public Tawot text cross-checks operands but requests Arcadian Chronicles verification. Require actual inventory/Equation changes, offer exclusion, nonvictory/zero/destroyed controls, fresh paid-acquisition replay and separate expiry vectors; no Boss placement or full public three-entry claim.",
        "58|59|60",
    ])
