"""Separate source-bound victory rewards; normal proxy domains remain excluded."""


def append_sage_victory_blessings(data: dict[str, list[list[object]]]) -> None:
    repository = "https://gitlab.com/Dimbreath/turnbasedgamedata"
    revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568"
    for source, name, locator, digest, note in [
        (36, "effects", "ExcelOutput/RogueMiracleEffect.json; MiracleEffectID=2192,2193,2194; ParamList[1]=1,2,3",
         "fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35",
         "Exact victory counts; ParamList[0] is the separate acquisition count."),
        (37, "text", "TextMap/TextMapEN.json; hashes=18402293723530033109,6941038067972437758,1542688008986925663; victory room_comp_type:2 and room_comp_type:4; all forms use 1-3-star victory Blessings",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
         "Exact domain locators and rarity range. Acquisition rarity is not the victory rarity filter; no normal Combat or Boss domain is listed."),
        (38, "domains", "ExcelOutput/RoguePersonaRoomCompType.json; LLICIMBCNPF=2,4; LHLKJIDFLIN=Elite,Encounter; BAAOGIMCALN.Hash=2832393198030600814,2381374495552649823; pinned EN names Elite,Aberration",
         "c0223603c6e5252278dac5224d766f1c4ed60ebe5845b9839b9e3533a8ad76a5",
         "Exact referenced display joins, not room placement or current-profile encounter membership. Shared source table does not promote other rows to reachable content."),
    ]:
        data["Sources"].append([source, f"du.source.sage-victory-{name}", repository, revision,
                                "4.4", "2026-09-12", locator, digest, "ExactStructured", note])
    data["CurioVictoryBlessings"] = [
        [ordinal, f"du.curio-victory-blessings.{state}", f"divergent-universe.curio-state.{state}",
         str(effect), 2, count, 1, 3, "Elite|Aberration", "BoundDomainUniformUnownedAvailableSubset",
         f"After victory in an Elite or Aberration domain, active Sage's Leaf Robe grants {count} random 1-3-star Blessings.",
         f"精英或异常区域战斗胜利后，生效的贤人叶袍获得 {count} 个随机 1 至 3 星祝福。",
         "VersionedProjectPolicy: require verified victory and the mode-owned domain binding; never infer a domain from enemy stage flags or the generic Encounter decision kind. Only the current active Curio form grants. Apply the same authored post-battle Blessing suppression states as the normal offer. After fragment victory grants and before normal offer generation, sample current unowned base Blessing identities within the exact rarity range, using unit weights in stable identity order without replacement across the entire victory-grant batch. Process grant states in stable order; use the available subset on exhaustion, with zero draws and no substitution when empty. Do not apply sealing-wax offer weights. Commit the final inventory and Equation refresh with all settlement stages and RNG. Production baseline domains remain Combat, so positive original Elite/Aberration producers remain pending.",
         "Replace domain placement, membership, unit weights, state ordering, suppression scope, exhausted-pool subset behavior and settlement timing independently when released graphs or reproducible current-profile observations establish them. Alternatives include per-domain rather than per-battle grants, weighted rarity, fixed reward pools, duplicate conversion and suppression of base drops only. Confidence in hidden distribution/interactions is low; source counts, eligible labels and 1-3-star range remain exact.",
         "32|36|37|38|12|13|14"]
        for ordinal, state, effect, count in [(1, 9192, 2192, 1), (2, 9193, 2193, 2), (3, 9194, 2194, 3)]
    ]
