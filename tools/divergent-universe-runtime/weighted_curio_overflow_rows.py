"""Walkie-Talkie released operands and callback evidence, not native admission."""


def append_weighted_curio_overflow(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (126, "hex", "ExcelOutput/RogueTournHex.json; HexID=1016; TournMode=Tourn3; MazeBuffID=633416; AvatarType=Mage,Rogue; DisplayID=1007",
         "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (127, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633416; Lv=1; ParamList=10,1,0.8; StageAbilityBeforeCharacterBorn; StageAbility_633416",
         "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (128, "text-en", "TextMap/TextMapEN.json; hash=12104045893670599658; attack increase and attack-kill overflow conversion",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (129, "text-zh", "TextMap/TextMapCHS.json; hash=12104045893670599658; attack increase and attack-kill overflow conversion",
         "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
        (130, "program", "Config/ConfigAbility/Level/Rogue_S7/Level_RogueBuff_Ability_HEX_S7.json; Name=StageAbility_633416; Modifier_StageAbility_633416_Character; OnAfterHitAll,OnTriggerDeath,OnTriggerDeathrattle,OnBeforeAttack,OnAfterAttackEnd",
         "5ad6c48751023184f5572e0069c5d02c841aef4de3cec092d0c2bc4913d17e47"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.weighted-curio-overflow.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-03",
            locator, digest, "ExactStructured",
            "Exact current membership, operands and named callback structure. The merged source program exists; no claim that opaque postfix semantics, hidden difficulty query or the native consumer is implemented.",
        ])
    data["WeightedCurioOverflows"] = [[
        1, "du.weighted-curio-overflow.parallel-universe-walkie-talkie",
        "divergent-universe.weighted-curio.1016", "633416", "Mage|Rogue",
        1, "10", 2, "1", 3, "0.8", "PendingNativeDeathCallbackAndBaseDamage",
        "Erudition and Hunt characters gain 80% ATK. Attack-kill overflow converts into True damage of ten times base damage plus the overflow, aimed at the highest-current-HP attacked enemy.",
        "智识和巡猎角色攻击力提高80%。攻击消灭敌人后的溢出值转化为10倍基础伤害加溢出值的真实伤害，目标为已攻击敌人中当前生命值最高者。",
        "Released structure: character-create Path filtering, a Replace AttackAddedRatio modifier, attack-start reset, hit-target marks and zero-current-HP damage observation, distinct death/deathrattle conversion callbacks, marked-current-HP maximum retarget with random ties, TrueDamage/ByBaseDamage conversion, then accumulator and mark cleanup. Attack-end clears During_Attack. No postfix decoding is inferred from opcode bytes or numeric hashes.",
        "Unimplemented: native death/deathrattle callback correspondence, hit/attack observation ordering, accumulator expression decoding, linked-actor ownership, tied-target labeled RNG and last-kill behavior. The base-DMG expression reads HPRatio, an opaque difficulty query and unresolved postfix operands; its exact level/difficulty formula is not established. Do not substitute character ATK, a constant or zero and claim parity. Replace each uncertainty with released typed semantics or a separately reviewed deterministic VersionedProjectPolicy preserving the known constraints. Bounded public cross-check on 2026-10-03: https://honkai-star-rail.fandom.com/wiki/Parallel_Universe_Walkie-Talkie corroborates current Arcadian Chronicles terms and level/Threshold-Protocol dependence, not the hidden formula. Ordinary/Cyclical consumer, Forge, replay and terminal coverage remain pending.",
        "126|127|128|129|130",
    ]]
