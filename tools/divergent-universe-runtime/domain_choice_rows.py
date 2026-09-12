"""Explicit public domain choices, without promoting retained room candidates."""


def append_domain_choices(data: dict[str, list[list[object]]]) -> None:
    repository = "https://gitlab.com/Dimbreath/turnbasedgamedata"
    revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568"
    for ordinal, name, locator, digest, note in [
        (39, "domain-choice-labels", "ExcelOutput/RoguePersonaRoomCompType.json; LLICIMBCNPF=3,2,4; LHLKJIDFLIN=Battle,Elite,Encounter; display hashes=7892844363747881325,2832393198030600814,2381374495552649823",
         "c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5",
         "Exact Combat/Elite/Aberration labels referenced by released text; not evidence of a current-profile selectable room pool."),
        (40, "initial-combat-domain", "ExcelOutput/RogueTournArea.json; HILINOJPLGA=Tourn3; DOKMKLJDCEK.LHLKJIDFLIN=Battle; ordered GLNDIILFKBN layers have no matching RogueTournLayerRoom rows",
         "756510177a468130464c27ef382d9ab33a7098dc993022cd8647366cc7c46db8",
         "Exact initial Combat label and layer ordering only. Later domain choices, placement and proxy encounters are independently replaceable policy."),
    ]:
        data["Sources"].append([ordinal, f"du.source.{name}", repository, revision, "4.4",
                                "2026-09-12", locator, digest, "ExactStructured", note])
    note = (
        "VersionedProjectPolicy: bind the first baseline battle to Combat. Before each later layer's encounter, after any Treasure events, offer the three explicitly authored domain labels as a required public Route choice. Set the selected domain once for that logical node, preserve it through battle and reward, and reset it on the next node. Do not promote any source room ID or Tourn2 candidate. Keep the current one-proxy-battle-per-layer route and the separately authored encounter candidate pool. The selected label controls domain-gated effects, including Sage victory grants. All current proxy domains retain the existing provisional Common/Rare base Blessing pool; original domain-specific base rewards, enemy programs, events, room multiplicity and Boss placement remain pending. Raw stage elite flags never select the domain. No adapter-supplied free-form label is accepted."
    )
    replacement = (
        "Replace later-domain membership, per-layer choices/order, room multiplicity, occurrence scheduling, encounter assignment and base reward policy independently when released current-profile selectors or reproducible observations establish them. Alternatives include random fixed domains, weighted room doors and multi-room planes. Hidden layout and distribution confidence is low; current behavior is an explicit selectable proxy policy, not original room parity."
    )
    data["DomainChoices"] = [
        [ordinal, f"du.domain-choice.{domain.lower()}", domain, name, name_zh,
         "ExplicitLaterLayerChoices", note, replacement, "18|24|37|39|40"]
        for ordinal, domain, name, name_zh in [
            (1, "Combat", "Combat Domain", "战斗区域"),
            (2, "Elite", "Elite Domain", "精英区域"),
            (3, "Aberration", "Aberration Domain", "异常区域"),
        ]
    ]
    data["BattleRoutes"][0][-3:] = [note, replacement, "18|39|40"]
    data["EncounterPools"][0][6:10] = [
        "Use the first proxy encounter, then draw one candidate per later domain selected by the player.",
        "首层使用固定代理遭遇，后续按玩家选择的区域独立抽取一个候选遭遇。",
        "VersionedProjectPolicy: the current stage pool supplies explicit proxies for Combat, Elite and Aberration choices, not original domain-specific encounters. Fix the first stage; draw later stages uniformly in stable order with replacement on shared Encounter RNG. Domain selection is authenticated separately and never inferred from source elite flags. Original enemy programs, room membership and weekly boss selection remain pending.",
        replacement,
    ]
    # The existing pool is a provisional base reward across current proxies,
    # not a completed Elite/Aberration drop table.
    data["BattleBlessings"][0][-3] += (
        " The current public domain-choice route applies this provisional base pool to Combat, Elite and Aberration proxies. Domain-specific higher-rarity base pools and other drops remain pending; separate exact Curio domain rewards execute before this offer."
    )
    for row in data["CurioVictoryBlessings"]:
        row[-3] = row[-3].replace(
            "Production baseline domains remain Combat, so positive original Elite/Aberration producers remain pending.",
            "The production DomainChoices policy enables positive Elite/Aberration contexts through authenticated public choices; original room placement and enemy programs remain pending.",
        )
