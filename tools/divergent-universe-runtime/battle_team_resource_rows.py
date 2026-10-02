"""Released shared-resource evidence and replaceable mode assembly policy."""


def append_battle_team_resources(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (111, "global-cap", "Config/GlobalConfig/GameCoreConstValue.json; ElationPointMax=9999",
         "5511ff36c631da99925c8aadae8ae46f50d620f4f937dbbed952fd61b16b80e3"),
        (112, "text-en", "TextMap/TextMapEN.json; hash=8389201339365092983; Punchline team sharing",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (113, "text-zh", "TextMap/TextMapCHS.json; hash=8389201339365092983; Punchline team sharing",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.battle-team-resource.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-02",
            locator, digest, "ExactStructured",
            "Exact global field or released bilingual team-sharing text. Mapping the global field to this resource and its initialization/lifecycle remains VersionedProjectPolicy. Activity-specific ElationBattle_MaxPower=99 is not this field.",
        ])
    data["BattleTeamResources"] = [[
        1, "du.battle-team-resource.punchline", "shared.punchline", "Elation", 0, 9999,
        "OriginalElationZeroClampPersist",
        "Punchline is a team resource. A mapped original Elation party enables one player-side binding.",
        "笑点为队伍共享资源；映射后的原始队伍含欢愉角色时装配一个我方资源。",
        "VersionedProjectPolicy: original mapped party Path membership enables one shared.punchline player binding independently of equipped Curios, life/presence changes and linked actors. Start each battle at zero; map the released global ElationPointMax field to maximum 9999. Generic Gain clamps to maximum with attempted/effective/overflow events, Spend rejects insufficient balance without mutation, and Set follows generic bounds. Persist across waves. Reconstruct at zero for a fresh battle or retry; no Activity inventory or cross-battle carry. Source fact is the global field value and team-sharing text, not observed initialization or lifecycle parity. No Aha actor, Certified Banger, automatic gain, damage scaling or meter reset trigger is implied.",
        "Low confidence for the field-to-resource mapping, enabling condition, zero initial value, cap overflow and wave/battle carry. Alternatives: unconditional binding, entry gain by Elation roster count, wave reset, checked overflow or cross-battle carry. Command tests battle_team_resources cover production resource assembly, the pending non-cost Skill delta lowering boundary, controlled overflow/spending/Set, wave carry, rejection and fresh reconstruction in both run families. Replace each field independently when a released executable binding or reproducible current Divergent Universe observation establishes it. ElationBattle_MaxPower=99 belongs to another activity and is excluded.",
        "111|112|113",
    ]]
