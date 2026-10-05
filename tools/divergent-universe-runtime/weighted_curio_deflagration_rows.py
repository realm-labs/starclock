"""Exact released Deflagration operands, never a native execution claim."""
from hashlib import sha256
from weighted_curio_overflow_rows import BASE_HP_RATIOS

BASE_POLICY_NOTE = "VersionedProjectPolicy: use the selected enemy's own level to choose released Group 1 HPRatio, multiply fixed operand 100 by that ratio, then by one plus immutable Protocol maximum-HP increase, flooring each checked Scalar multiplication to six places. The hidden postfix, Group 1 selection, target-level correspondence, Protocol interpretation, operator order and rounding remain independently replaceable low-confidence fields, not decoded or observed parity. Preserve level/Protocol dependence rather than substituting owner ATK or a level-independent constant."
BASE_REPLACEMENT = "Alternatives include stage-level scaling, a different hard-level group, a separately decoded difficulty factor or different operation/rounding order. Group 1 is the existing deterministic source-backed curve policy, not source membership evidence for this mechanic. Replace each field independently when released typed postfix semantics or reproducible current numerical traces establish it. Data tests check exact-once 95 rows, canonical decimals, policy/source isolation and operand drift; native all-level and Protocol command consumers remain pending."
RUNTIME_POLICY_NOTE = "VersionedProjectPolicy: canonical Basic scaling-damage element proves each original entry-form Fire member; original party formations zero through three only, excluding owned linked units. At BattleStarted capture the alive/present original Fire count once, selecting the factual one/two/three/four-member Burn fractions; no recapture after defeat, revival, transformation or waves. Each eligible original Unit attack applies Deflagration once at complete ActionResolved AfterAction priority zero to distinct living/present opposing EventTargets, with Guaranteed application, no chance roll. Replace the prior target Deflagration across casters, capture twice the selected target's authored level/Protocol base, one dispellable Burn stack, SourceSnapshotTargetDynamic, TurnStart tick, TargetTurnEnd two-turn lifetime. Only own-source DotTick DamageApplied AfterEvent priority zero once per event detonates other classified Burns, including periodic Break Burns, in shared stable effect-instance order; exclude all Deflagration instances and preserve original other-Burn sources/lifetimes. DotDetonation never triggers other Burns. Timing, chance, cross-caster replacement, source snapshot, recipient presence, original-actor borrowing and count retention are independent low-confidence fields, not hidden callback parity."
RUNTIME_REPLACEMENT = "Alternatives include hit-end application, source ReplaceByCaster coexistence, enemy-owned effects, resistible application, live Fire counts, inherited actors or other native reaction priorities. Complete-action application and single cross-caster effect use the existing bounded command/effect pipeline without introducing a second state machine. Replace each field independently with released typed semantics or reproducible current traces; retain the source-backed natural-tick versus custom-event distinction. Native tests must cover one through four originals, no-Fire admission, duplicate hits/targets, multi-caster replacement, two real turns, ordinary/Break Burns, external detonation nonrecursion, linked and transformed actors, rejection, fresh payloads/hashes and unequip in both families. Forge, equipment-command replay and complete-run release remain separate pending work."


def append_weighted_curio_deflagration(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (141, "hex", "ExcelOutput/RogueTournHex.json; HexID=1012; TournMode=Tourn3; MazeBuffID=633412; AvatarDamageType=Fire", "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455"),
        (142, "maze-buff", "ExcelOutput/MazeBuff.json; ID=633412; Lv=1; ParamList=0.5,1,1.5,2,2,2; StageAbility_633412", "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac"),
        (143, "text-en", "TextMap/TextMapEN.json; hash=8602529829111077351; Deflagration and one-through-four Fire clauses", "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
        (144, "text-zh", "TextMap/TextMapCHS.json; hash=8602529829111077351; Deflagration and one-through-four Fire clauses", "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147"),
        (145, "program", "Config/ConfigAbility/Level/Rogue_S7/Level_RogueBuff_Ability_HEX_S7.json; Name=StageAbility_633412; OnEnterBattle Fire count; SuperBurn OnPhase1 versus OnCustomEvent; HPRatio and fixed operand 100", "5ad6c48751023184f5572e0069c5d02c841aef4de3cec092d0c2bc4913d17e47"),
        (146, "hard-level", "ExcelOutput/HardLevelGroup.json; HardLevelGroup=1; Level=1..95; HPRatio.Value", "d185c09b5388f4eeb368199276ee8a815b406fd9902744027b72a5011b962978"),
    ]:
        data["Sources"].append([ordinal, f"du.source.weighted-curio-deflagration.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata", "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-05",
            locator, digest, "ExactStructured", "Exact released joins, operands, curve values and named callbacks. Hidden postfix and Starclock timing/ownership parity are not inferred."])
    for ordinal, suffix, note, anchor in [(147, "base-policy", BASE_POLICY_NOTE, "base-damage-policy"), (148, "runtime-policy", RUNTIME_POLICY_NOTE, "execution-policy")]:
        data["Sources"].append([ordinal, f"du.source.weighted-curio-deflagration.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata", "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-10-05",
            f"docs/divergent-universe-weighted-curio-deflagration.md#{anchor}", sha256(note.encode("utf-8")).hexdigest(), "ProjectPolicy",
            f"Digest binds the corresponding {suffix} note in WeightedCurioDeflagrations, not an upstream blob. Replace fields independently; native execution remains pending."])
    data["WeightedCurioDeflagrations"] = [[1, "du.weighted-curio-deflagration.most-raucous", "divergent-universe.weighted-curio.1012", "633412", "Fire",
        "0.5|1|1.5|2", 5, "2", 6, 2, "100", 1, "AuthoredOperandsPendingNative",
        "OriginalFireAfterActionNaturalTickBurns", "TargetGroupOneHpRatioProtocolHpFloor",
        "Fire originals apply two-turn Deflagration, counted as Burn; natural ticks deal twice base damage and detonate other Burns at the original Fire-count fraction.",
        "火属性原角色施加两回合爆燃（视为灼烧）；自然周期结算造成两倍基础伤害，并按入场火角色人数比例引爆其他灼烧。",
        RUNTIME_POLICY_NOTE, RUNTIME_REPLACEMENT, BASE_POLICY_NOTE, BASE_REPLACEMENT, "141|142|143|144|145|146|147|148"]]
    data["WeightedCurioDeflagrationLevels"] = [[level, f"du.weighted-curio-deflagration.level.{level:03}",
        "divergent-universe.weighted-curio.1012", 1, level, ratio, "146"]
        for level, ratio in enumerate(BASE_HP_RATIOS, 1)]
