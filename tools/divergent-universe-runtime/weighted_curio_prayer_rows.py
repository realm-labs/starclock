"""Released Road of Prayers values, with independent resource/lifecycle policy."""


def append_weighted_curio_prayer(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (89, "hex", "ExcelOutput/RogueTournHex.json; HexID=1004; TournMode=Tourn3; MazeBuffID=633404; AvatarType=Mage|Warlock; DisplayID=1022",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (90, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633404; Lv=1; ParamList[0].Value=0.6; ParamList[1].Value=0.15; ParamList[2].Value=0.25; InBattleBindingKey=StageAbility_633404",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (91, "text-en", "TextMap/TextMapEN.json; hash=16252001468855081658; Erudition Nihility maximum HP bonus and turn-start current HP consumption followed by maximum-HP shield",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (92, "text-zh", "TextMap/TextMapCHS.json; hash=16252001468855081658; Erudition Nihility maximum HP bonus and turn-start current HP consumption followed by maximum-HP shield",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-prayer.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-01",
            locator, digest, "ExactStructured",
            "Released Tourn3 join and exact 0.6 maximum HP, 0.15 current HP consumption and 0.25 maximum HP shield operands. Hidden stacking, entry adjustment, phase, floor and shield lifetime remain policy.",
        ])
    data["WeightedCurioPrayers"] = [[
        1, "du.weighted-curio-prayer.road-of-prayers", "divergent-universe.weighted-curio.1004",
        "633404", "Mage|Warlock", 1, "0.6", 2, "0.15", 3, "0.25", "EntryHpAndTurnStartConsumeShield",
        "Erudition and Nihility characters gain 60% maximum HP; their turn starts consume 15% current HP and grant a shield for 25% maximum HP.",
        "智识、虚无角色生命上限提高 60%；回合开始时消耗 15% 当前生命值并获得等同于 25% 生命上限的护盾。",
        "VersionedProjectPolicy: use the immutable resolved entry HP capacity as the bonus basis, adding floor(base times 0.6) exactly once to a fresh sealed ParticipantSpec. Preserve every build field and locked build digest; do not add a second HP modifier. No extra healing of carried HP: first entry uses explicit initial state or full new capacity; later entry follows the declared Activity carry policies, including clamping on unequip. Bind mapped Erudition/Nihility player characters, with no linked/summon inheritance. Owner TurnStarted AfterEvent, once per Turn, priority zero and alive/present owner selector: consume floor(current HP times 0.15) with HP floor one, then remove only this source-owned shield/effect and apply one non-dispellable battle-lifetime replacement shield. Shield uses the live HP resource capacity times 0.25 with floor, shared shield modifiers and shared operations; no HP-stat substitute. Refresh on the next owner's turn, not per hit or Ultimate interrupt. Source operations retain ordered events and snapshot reads; no battle mutation of Activity. Down/absent actors do not trigger; teardown belongs to battle.",
        "Hidden base/additive stacking, entry HP adjustment, phase, consume rounding and lethal floor, duration/refresh/dispel and summon inheritance have low policy confidence, not observed parity. Alternatives: pure base-character HP, additive live HP modifiers, proportional current-HP scaling, lethal drain, target-turn shield expiry or accumulating layers. Selected entry-capacity policy executes all three exact operands through real resources and bounded shared operations without pretending a stat-only bonus changes HP capacity. Affected tests weighted_curio_prayer and production data rejection fixtures. Replace each field independently when released executable programs or reproducible current traces establish it. Forge offers, equipment-command replay and complete runs remain pending.",
        "89|90|91|92",
    ]]
