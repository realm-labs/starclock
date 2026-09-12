"""Authored provisional base battle drops; no exact amount claim."""


def append_battle_fragments(data: dict[str, list[list[object]]]) -> None:
    data["Sources"].append([
        41, "du.source.battle-fragments.constants-inspection",
        "https://gitlab.com/Dimbreath/turnbasedgamedata",
        "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-12",
        "ExcelOutput/RogueTournConstCommon.json; all 34 ConstValueName rows inspected; no base battle-fragment amount is declared",
        "3ad1c238e4adbf89bb614cecba90327cdd1e4fa25386821b13854fab0c3b7901",
        "ExactStructured",
        "Evidence of this inspected table only, not proof that no other source defines drops. Stage markers and account-reward display IDs do not establish run fragment amounts. All authored base amounts are explicit project policy.",
    ])
    note = (
        "VersionedProjectPolicy: after each verified won battle, credit the authored fixed base amount for the authenticated domain before separate Curio victory grants and Blessing generation. Combat and Aberration proxies grant 40; Elite proxies grant 100. These amounts are engineering choices, not observed game values: they provide a nonzero repeatable combat income with an explicit higher Elite reward while original drop distributions remain unavailable. Apply the existing global fragment-gain pipeline to this credit independently of each Curio credit. Loss/fault grants nothing, Blessing suppression does not suppress fragments, and no reward RNG is drawn. Result verification, carry, all reward stages and automatic advance roll back together. Do not infer domain from source elite flags, enemy count or account reward displays. No boss policy is declared until that route exists."
    )
    replacement = (
        "Replace each domain amount, fixed versus random distribution, per-battle versus enemy/drop timing, difficulty scaling, occurrence reward stacking and ordering independently when released current-profile drop programs or reproducible observations establish them. Alternatives include per-enemy rewards and integer range sampling. Amount/distribution confidence is low. Current constants and stage inspection do not prove parity; do not credit original program completion from this policy alone."
    )
    data["BattleFragments"] = [
        [ordinal, f"du.battle-fragments.{domain.lower()}", domain, amount,
         "FixedVerifiedDomainCredit", note, replacement, "14|24|39|41"]
        for ordinal, domain, amount in [(1, "Combat", 40), (2, "Elite", 100), (3, "Aberration", 40)]
    ]
