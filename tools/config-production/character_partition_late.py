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

C10_DAMAGE = {
    "character.trailblazer.harmony.ability.halftime-to-make-it-rain.bpskill": [("Primary", 1, 1), ("BounceDraw", 1, 4)],
    "character.trailblazer.harmony.ability.swing-dance-etiquette.normal": [("Primary", 1, 1)],
    "character.trailblazer.preservation.ability.ice-breaking-light.normal": [("Primary", 1, 1)],
    "character.trailblazer.preservation.ability.war-flaming-lance.ultra": [("All", 2, 1)],
    "character.trailblazer.remembrance.ability.leave-it-to-me.normal": [("Primary", 1, 1)],
    "character.trailblazer.remembrance.ability.memories-back-as-echoes.maze": [("All", 3, 1)],
    "character.trailblazer.remembrance.ability.together-mem.ultra": [("All", 1, 1)],
    "character.trailblazer.remembrance.ability.together-we-script-tomorrow.normal": [("All", 1, 1), ("All", 2, 1)],
    "character.tribbie.ability.busy-as-tribbie.skillp01": [("All", 1, 1)],
    "character.tribbie.ability.guess-who-lives-here.ultra": [("All", 1, 1)],
    "character.tribbie.ability.hundred-rockets.normal": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.welt.ability.edge-of-the-void.bpskill": [("Primary", 1, 1), ("BounceDraw", 1, 2)],
    "character.welt.ability.gravity-suppression.normal": [("Primary", 1, 1)],
    "character.welt.ability.synthetic-black-hole.ultra": [("All", 1, 1)],
    "character.welt.ability.time-distortion.skillp01": [("Primary", 1, 1)],
    "character.xueyi.ability.divine-castigation.ultra": [("Primary", 1, 1)],
    "character.xueyi.ability.iniquity-obliteration.bpskill": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.xueyi.ability.karmic-perpetuation.skillp01": [("BounceDraw", 2, 3)],
    "character.xueyi.ability.mara-sunder-awl.normal": [("Primary", 1, 1)],
    "character.xueyi.ability.summary-execution.maze": [("All", 1, 1)],
    "character.yanqing.ability.amidst-the-raining-bliss.ultra": [("Primary", 3, 1)],
    "character.yanqing.ability.darting-ironthorn.bpskill": [("Primary", 1, 1)],
    "character.yanqing.ability.frost-thorn.normal": [("Primary", 1, 1)],
    "character.yanqing.ability.one-with-the-sword.skillp01": [("Primary", 4, 1)],
    "character.yao-guang.ability.behold-wherever-light-unfolds.skillp01": [("Primary", 1, 1)],
    "character.yao-guang.ability.let-thy-fortune-burst-in-flames.elationdamage": [("All", 2, 1), ("BounceDraw", 6, 5)],
    "character.yao-guang.ability.whistlebolt-sings-joy.normal": [("Primary", 1, 1), ("Adjacent", 2, 1)],
}

C11_DAMAGE = {
    "character.yukong.ability.arrowslinger.normal": [("Primary", 1, 1)],
    "character.yukong.ability.diving-kestrel.ultra": [("Primary", 1, 1)],
    "character.yukong.ability.seven-layers-one-arrow.skillp01": [("Primary", 1, 1)],
    "character.yunli.ability.bladeborne-quake.bpskill": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.yunli.ability.flashforge.skillp01": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.yunli.ability.galespin-summersault.normal": [("Primary", 1, 1)],
}

DAMAGE_BY_PARTITION = {"C09": C09_DAMAGE, "C10": C10_DAMAGE, "C11": C11_DAMAGE}

TARGET_OVERRIDES = {
    "character.the-herta.ability.hear-me-out.bpskill": "Blast",
    "character.tingyun.ability.violet-sparknado.skillp01": "SingleTarget",
    "character.topaz-numby.ability.difficulty-paying.bpskill": "SingleTarget",
    "character.trailblazer.elation.ability.i-said-elation-did-i-stutter.elationdamage": "Bounce",
    "character.welt.ability.time-distortion.skillp01": "SingleTarget",
    "character.yanqing.ability.one-with-the-sword.skillp01": "SingleTarget",
    "character.yao-guang.ability.behold-wherever-light-unfolds.skillp01": "SingleTarget",
    "character.yao-guang.ability.let-thy-fortune-burst-in-flames.elationdamage": "Bounce",
    "character.yao-guang.ability.whistlebolt-sings-joy.normal": "Blast",
    "character.yukong.ability.seven-layers-one-arrow.skillp01": "SingleTarget",
    "character.yunli.ability.earthbind-etherbreak.ultra": "Enhance",
}

ABILITY_KIND_OVERRIDES = {
    "character.the-dahlia.ability.whos-afraid-of-constance.skillp01": "FollowUp",
    "character.the-herta.ability.hear-me-out.bpskill": "EnhancedSkill",
    "character.topaz-numby.ability.trotter-market.skillp01": "Summon",
    "character.trailblazer.destruction.ability.blowout-farewell-hit.ultra": "Passive",
    "character.trailblazer.destruction.ability.blowout-rip-home-run.ultra": "Passive",
    "character.trailblazer.remembrance.ability.together-we-script-tomorrow.normal": "EnhancedBasic",
    "character.tribbie.ability.busy-as-tribbie.skillp01": "FollowUp",
    "character.xueyi.ability.karmic-perpetuation.skillp01": "FollowUp",
    "character.yanqing.ability.one-with-the-sword.skillp01": "FollowUp",
    "character.yunli.ability.flashforge.skillp01": "Counter",
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
    "character.trailblazer.remembrance.ability.i-choose-you.bpskill": 1 << 7,
    "character.trailblazer.remembrance.ability.together-mem.ultra": 1 << 7,
    "character.trailblazer.remembrance.ability.together-we-script-tomorrow.normal": 1 << 9,
    "character.welt.ability.time-distortion.skillp01": 1 << 8,
    "character.yao-guang.ability.behold-wherever-light-unfolds.skillp01": 1 << 8,
    "character.yao-guang.ability.let-thy-fortune-burst-in-flames.elationdamage": 1 << 10,
    "character.yukong.ability.seven-layers-one-arrow.skillp01": 1 << 8,
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
    "character.trailblazer.harmony.ability.all-out-footlight-parade.ultra": 1 << 3,
    "character.trailblazer.harmony.ability.full-on-aerial-dance.skillp01": 0,
    "character.trailblazer.harmony.ability.now-im-the-band.maze": 0,
    "character.trailblazer.preservation.ability.call-of-the-guardian.maze": 0,
    "character.trailblazer.preservation.ability.ever-burning-amber.bpskill": 1 << 2,
    "character.trailblazer.preservation.ability.treasure-of-the-architects.skillp01": 0,
    "character.trailblazer.remembrance.ability.almighty-companion.skillp01": 1 << 7,
    "character.trailblazer.remembrance.ability.i-choose-you.bpskill": (1 << 2) | (1 << 7),
    "character.trailblazer.remembrance.ability.together-mem.ultra": (1 << 0) | (1 << 3) | (1 << 7),
    "character.trailblazer.remembrance.ability.together-we-script-tomorrow.normal": (1 << 0) | (1 << 1) | (1 << 9),
    "character.tribbie.ability.if-youre-happy-and-you-know-it.maze": 0,
    "character.tribbie.ability.whered-the-gifts-go.bpskill": 1 << 2,
    "character.welt.ability.gravitational-imprisonment.maze": 0,
    "character.yanqing.ability.the-one-true-sword.maze": 0,
    "character.yao-guang.ability.behold-wherever-light-unfolds.skillp01": 1 << 8,
    "character.yao-guang.ability.decalight-unveils-all.bpskill": 1 << 2,
    "character.yao-guang.ability.hexagram-of-feathered-fortune.ultra": 1 << 3,
    "character.yao-guang.ability.let-thy-fortune-burst-in-flames.elationdamage": (1 << 0) | (1 << 10),
    "character.yao-guang.ability.untethered-glimmer-sails-far.maze": 0,
    "character.yukong.ability.emboldening-salvo.bpskill": 1 << 2,
    "character.yukong.ability.seven-layers-one-arrow.skillp01": (1 << 0) | (1 << 8),
    "character.yukong.ability.windchaser.maze": 0,
    "character.yunli.ability.earthbind-etherbreak.ultra": 1 << 3,
    "character.yunli.ability.posterior-precedence.maze": 0,
}

CHARACTER_RESOURCES = {
    "character.the-herta": [("inspiration", "4", "0")],
    "character.xueyi": [("karma", "8", "0")],
    "character.yukong": [("roaring-bowstrings", "2", "0")],
}

CHARACTER_RESOURCE_COSTS = {
    "character.the-herta.ability.hear-me-out.bpskill": [("inspiration", "1")],
    "character.xueyi.ability.karmic-perpetuation.skillp01": [("karma", "8")],
}

CHARACTER_RESOURCE_GAINS = {
    "character.the-herta.ability.told-ya-magic-happens.ultra": [("inspiration", "1")],
    "character.yukong.ability.emboldening-salvo.bpskill": [("roaring-bowstrings", "2")],
}

TEAM_RESOURCE_GAINS = {
    "character.trailblazer.elation.ability.may-the-trailblaze-fly-you-starward.ultra": [("shared.punchline", "5")],
    "character.yao-guang.ability.decalight-unveils-all.bpskill": [("shared.punchline", "3")],
    "character.yao-guang.ability.hexagram-of-feathered-fortune.ultra": [("shared.punchline", "5")],
}

ENERGY_GAIN_OVERRIDES = {
    "character.tingyun.ability.gentle-breeze.maze": "50",
}

ENERGY_COST_OVERRIDES = {
    "character.yunli.ability.earthbind-etherbreak.ultra": "120",
}

SCALING_STATS = {
    "character.trailblazer.preservation.ability.war-flaming-lance.ultra": "Def",
    "character.tribbie.ability.busy-as-tribbie.skillp01": "Hp",
    "character.tribbie.ability.guess-who-lives-here.ultra": "Hp",
    "character.tribbie.ability.hundred-rockets.normal": "Hp",
}

DAMAGE_CLASSES = {
    "character.yao-guang.ability.behold-wherever-light-unfolds.skillp01": "Elation",
}

IGNORES_WEAKNESS = {
    "character.xueyi.ability.divine-castigation.ultra",
}
