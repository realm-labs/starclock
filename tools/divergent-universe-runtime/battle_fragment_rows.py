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
        "VersionedProjectPolicy: after each verified won battle, credit the authored fixed base amount for the authenticated domain before separate Curio victory grants and Blessing generation. Combat and Aberration proxies grant 40; Elite proxies grant 100. These amounts are engineering choices, not observed game values: they provide a nonzero repeatable combat income with an explicit higher Elite reward while original drop distributions remain unavailable. Apply the existing global fragment-gain pipeline to this credit independently of each Curio credit. Loss/fault grants nothing, Blessing suppression does not suppress fragments, and no reward RNG is drawn. Result verification, carry, all reward stages and automatic advance roll back together. Do not infer domain from source elite flags, enemy count or account reward displays. Boss uses a separate explicit-policy row without enlarging the legacy three-label domain-choice route."
    )
    replacement = (
        "Replace each domain amount, fixed versus random distribution, per-battle versus enemy/drop timing, difficulty scaling, occurrence reward stacking and ordering independently when released current-profile drop programs or reproducible observations establish them. Alternatives include per-enemy rewards and integer range sampling. Amount/distribution confidence is low. Current constants and stage inspection do not prove parity; do not credit original program completion from this policy alone."
    )
    data["BattleFragments"] = [
        [ordinal, f"du.battle-fragments.{domain.lower()}", domain, amount,
         "FixedVerifiedDomainCredit", note, replacement, "14|24|39|41"]
        for ordinal, domain, amount in [(1, "Combat", 40), (2, "Elite", 100), (3, "Aberration", 40)]
    ]
    data["BattleFragments"].append([
        4, "du.battle-fragments.boss", "Boss", 100, "FixedVerifiedDomainCredit",
        "VersionedProjectPolicy: credit 100 base fragments after a verified win in an explicitly bound Boss reward domain, before separate Curio victory grants, battle lifetimes and the existing provisional Blessing offer. This is an engineering choice, not an observed game amount or a claim of original Boss eligibility, AI, phases or drops. Use the same nonzero base as the existing Elite proxy to extend the bounded reward pipeline without inventing a higher Boss multiplier; the domains remain independently replaceable. Select the domain only from the immutable room binding, never from preset level, stage elite flags or an adapter label. Apply current global fragment-gain modifiers to this credit separately from each Curio credit. Loss/fault grants nothing; Blessing suppression does not suppress fragments; the fixed base draws no RNG. Verified result, carry, all grants and next-node initialization commit or roll back together. The three-label legacy domain-choice route remains unchanged.",
        "Replace Boss amount, fixed versus random distribution, per-battle versus per-enemy timing, difficulty scaling, base Blessing pool and reward ordering independently when released current-profile drop programs or reproducible observations establish them. Alternatives are a zero Boss base, a larger Boss-specific constant or integer range sampling; none is proven by the inspected constants, source labels or public domain overview. This explicit 100-fragment policy has low parity confidence, preserves independent Boss-domain authentication, and must not terminalize original Boss programs or source obligations.",
        "14|24|41|66",
    ])
