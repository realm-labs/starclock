"""Data-only overrides for late frozen character partitions.

Facts here come from the prepared Version 4.4 pack. Keeping them separate
prevents the deterministic workbook author from exceeding the 1,200-line
handwritten-source policy as the final partitions are promoted.
"""

C09_DAMAGE = {
    "character.sunday.ability.gleaming-admonition.normal": [("Primary", 1, 1)],
    "character.sushang.ability.cloudfencer-art-mountainfall.bpskill": [("Primary", 1, 1)],
    "character.sushang.ability.cloudfencer-art-starshine.normal": [("Primary", 1, 1)],
    "character.sushang.ability.cloudfencer-art-warcry.maze": [("All", 1, 1)],
    "character.sushang.ability.shape-of-taixu-dawn-herald.ultra": [("Primary", 1, 1)],
    "character.the-dahlia.ability.fiddle-fissured-memory.normal": [("Primary", 1, 1)],
    "character.the-dahlia.ability.lick-enkindled-betrayal.bpskill": [("Primary", 1, 1), ("Adjacent", 1, 1)],
    "character.the-dahlia.ability.wallow-entombed-ash.ultra": [("All", 1, 1)],
    "character.the-dahlia.ability.whos-afraid-of-constance.skillp01": [("BounceDraw", 1, 5)],
    "character.the-herta.ability.big-brain-energy.bpskill": [
        ("Primary", 1, 1), ("Primary", 1, 1), ("Adjacent", 1, 1),
        ("Primary", 1, 1), ("Adjacent", 1, 1),
    ],
    "character.the-herta.ability.did-you-get-it.normal": [("Primary", 1, 1)],
    "character.the-herta.ability.hear-me-out.bpskill": [
        ("Primary", 1, 1), ("Primary", 1, 1), ("Adjacent", 1, 1),
        ("Primary", 1, 1), ("Adjacent", 1, 1), ("All", 3, 1),
    ],
    "character.the-herta.ability.told-ya-magic-happens.ultra": [("All", 1, 1)],
    "character.tingyun.ability.dislodged.normal": [("Primary", 1, 1)],
    "character.tingyun.ability.violet-sparknado.skillp01": [("Primary", 1, 1)],
    "character.topaz-numby.ability.deficit.normal": [("Primary", 1, 1)],
    "character.topaz-numby.ability.difficulty-paying.bpskill": [("Primary", 1, 1)],
    "character.topaz-numby.ability.trotter-market.skillp01": [("Primary", 2, 1)],
    "character.trailblazer.destruction.ability.blowout-farewell-hit.ultra": [("Primary", 1, 1)],
    "character.trailblazer.destruction.ability.blowout-rip-home-run.ultra": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.trailblazer.destruction.ability.farewell-hit.normal": [("Primary", 1, 1)],
    "character.trailblazer.destruction.ability.rip-home-run.bpskill": [("Primary", 1, 1), ("Adjacent", 1, 1)],
    "character.trailblazer.elation.ability.i-said-elation-did-i-stutter.elationdamage": [("BounceDraw", 2, 8), ("All", 3, 1)],
    "character.trailblazer.elation.ability.let-the-storm-rage-on.bpskill": [("All", 1, 1)],
    "character.trailblazer.elation.ability.make-some-noise.normal": [("Primary", 1, 1)],
}

DAMAGE_BY_PARTITION = {"C09": C09_DAMAGE}

TARGET_OVERRIDES = {
    "character.the-herta.ability.hear-me-out.bpskill": "Blast",
    "character.tingyun.ability.violet-sparknado.skillp01": "SingleTarget",
    "character.topaz-numby.ability.difficulty-paying.bpskill": "SingleTarget",
    "character.trailblazer.elation.ability.i-said-elation-did-i-stutter.elationdamage": "Bounce",
}

ABILITY_KIND_OVERRIDES = {
    "character.the-dahlia.ability.whos-afraid-of-constance.skillp01": "FollowUp",
    "character.the-herta.ability.hear-me-out.bpskill": "EnhancedSkill",
    "character.topaz-numby.ability.trotter-market.skillp01": "Summon",
    "character.trailblazer.destruction.ability.blowout-farewell-hit.ultra": "Passive",
    "character.trailblazer.destruction.ability.blowout-rip-home-run.ultra": "Passive",
}

ABILITY_TAG_MASK_OVERRIDES = {
    "character.sushang.ability.cloudfencer-art-mountainfall.bpskill": 1 << 8,
    "character.tingyun.ability.soothing-melody.bpskill": 1 << 8,
    "character.tingyun.ability.violet-sparknado.skillp01": 1 << 8,
    "character.topaz-numby.ability.difficulty-paying.bpskill": 1 << 4,
    "character.topaz-numby.ability.trotter-market.skillp01": 1 << 4,
    "character.trailblazer.destruction.ability.blowout-farewell-hit.ultra": 1 << 3,
    "character.trailblazer.destruction.ability.blowout-rip-home-run.ultra": 1 << 3,
    "character.trailblazer.elation.ability.i-said-elation-did-i-stutter.elationdamage": 1 << 10,
}

ABILITY_TAG_MASK_REPLACEMENTS = {
    "character.sunday.ability.benison-of-paper-and-rites.bpskill": 1 << 2,
    "character.sunday.ability.ode-to-caress-and-cicatrix.ultra": 1 << 3,
    "character.sunday.ability.the-glorious-mysteries.maze": 0,
    "character.the-dahlia.ability.the-heart-makes-the-finest-tomb.maze": 0,
    "character.the-herta.ability.hand-them-over.skillp01": 0,
    "character.the-herta.ability.vibe-checker.maze": 0,
    "character.tingyun.ability.amidst-the-rejoicing-clouds.ultra": 1 << 3,
    "character.tingyun.ability.soothing-melody.bpskill": (1 << 2) | (1 << 8),
    "character.topaz-numby.ability.turn-a-profit.ultra": 1 << 3,
    "character.trailblazer.elation.ability.may-the-trailblaze-fly-you-starward.ultra": 1 << 3,
    "character.trailblazer.elation.ability.that-smile-hits-different.skillp01": (1 << 0) | (1 << 8) | (1 << 10),
}

CHARACTER_RESOURCES = {
    "character.the-herta": [("inspiration", "4", "0")],
}

CHARACTER_RESOURCE_COSTS = {
    "character.the-herta.ability.hear-me-out.bpskill": [("inspiration", "1")],
}

CHARACTER_RESOURCE_GAINS = {
    "character.the-herta.ability.told-ya-magic-happens.ultra": [("inspiration", "1")],
}

TEAM_RESOURCE_GAINS = {
    "character.trailblazer.elation.ability.may-the-trailblaze-fly-you-starward.ultra": [("shared.punchline", "5")],
}

ENERGY_GAIN_OVERRIDES = {
    "character.tingyun.ability.gentle-breeze.maze": "50",
}
