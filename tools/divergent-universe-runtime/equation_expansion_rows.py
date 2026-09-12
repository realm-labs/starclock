"""Exact expansion-trigger operands with independently replaceable scheduling."""


def append_equation_expansion_rewards(data: dict[str, list[list[object]]]) -> None:
    for ordinal, suffix, locator, digest in [
        (61, "state", "ExcelOutput/RogueTournMiracle.json; MiracleID=9074; TournMode=Tourn3; HandbookMiracleID=9068; MiracleEffectID=2074",
         "176b17030bdf212920d8a59142cf982916349ee986b574b3e2d3bbdeff81c438"),
        (62, "parameters", "ExcelOutput/RogueMiracleEffect.json; MiracleEffectID=2074; ParamList[0]=1; ParamList[1]=3",
         "fd40b0b35b2735875356681597a532537fb4a98fbcddded501e8d4b4e3eede35"),
        (63, "text", "TextMap/TextMapEN.json; hash=1970395831290142958; expanded Equation grants random Blessing; discard after three triggers",
         "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789"),
    ]:
        data["Sources"].append([
            ordinal, f"du.source.equation-expansion.{suffix}",
            "https://gitlab.com/Dimbreath/turnbasedgamedata",
            "fd978d6ef09f941fba644c731ab54abd6f7c3568", "4.4", "2026-09-12",
            locator, digest, "ExactStructured",
            "Exact current state/effect join, one random Blessing per Equation expansion and three-trigger discard. Pool, timing, cascades and empty-pool handling remain project policy.",
        ])
    data["EquationExpansionRewards"] = [[
        1, "du.equation-expansion-reward.9074", "divergent-universe.curio-state.9074", "2074",
        1, 1, 2, 3, 1, 3, "ActivePreStateUniformUnownedBoundedCascade",
        "An expanded Equation grants one random Blessing; discard after three triggers.",
        "方程展开时获得一个随机祝福，触发三次后丢弃。",
        "VersionedProjectPolicy: snapshot active holding and positive remaining allowance before the owning inventory command. Acquiring this Curio does not retroactively reward expanded Equations. Coalesce each originating ownership batch, including Trailblaze Wax grants, into one final Equation/Blessing input. Queue absent/unexpanded-to-expanded identities in stable order; removed/collapsed identities do not trigger, later re-expansion does. Each queued edge consumes one allowance even if the reward pool is empty. Uniformly sample an unowned current base Blessing across the policy-selected 1-3-star pool, excluding enhanced holdings and identities reserved by an unrelated pending offer. Do not inherit wax weights or battle-reward suppression. Each reward updates recipe contributions and appends newly expanded identities to the FIFO queue. Remaining allowance bounds all cascades; discard holding and counters immediately after the limiting trigger. Destroyed states pause; repair resumes; replacement/reacquisition refill. Commit ownership, final progress, allowance, discard, events and Reward RNG purpose 24141 with the original authenticated command; every failure restores all of them.",
        "Low confidence for pool/rarity/weights, pending-offer reservation, pre-command holding snapshot, coalesced input timing, FIFO cascades, empty-trigger consumption and lifecycle reset rules. Alternatives include weighted rarity, independent reward suppression, intermediate acquisition triggers, no reward-induced cascades and empty-pool no-spend. The chosen finite queue preserves expansion rewards without recursive mutation or unbounded draws. Replace each field with released execution or reproducible current observations; retain exact one reward and three-trigger limit. Required fixtures: all ownership producers, simultaneous and reward-induced expansions, no enhancement retrigger, collapse/re-expansion, lifecycle controls, exhausted and reserved pools, rollback and fresh paid-acquisition replay. Table compilation alone is not runtime completion.",
        "61|62|63",
    ]]
