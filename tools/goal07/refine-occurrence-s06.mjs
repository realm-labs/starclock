#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const write = process.argv.slice(2).includes("--write");
assert(
  process.argv.slice(2).every((argument) => argument === "--write"),
  "usage: node tools/goal07/refine-occurrence-s06.mjs [--write]",
);

const packRoot = path.join(root, "content-reference", "standard-universe-v1");
const choicesPath = path.join(packRoot, "occurrence-choices.json");
const indexPath = path.join(packRoot, "pack-index.json");
const reviewPath = path.join(
  root,
  "evidence",
  "standard-universe-mechanics-complete-v1",
  "source-reviews",
  "G07-P4-M13-S06.json",
);
const originalChoices = fs.readFileSync(choicesPath, "utf8");
const choices = JSON.parse(originalChoices);
const byId = new Map(choices.map((choice) => [choice.id, choice]));

const rewardPaths = "universe.occurrence-battle.reward.paths";
const level = "universe.occurrence-battle.level.60";
const progressive = (key) => `universe.occurrence-progressive.key.${key}`;
const stage = (key) => `universe.occurrence-battle.stage.${key}`;
const fixedReward = (count) =>
  `universe.occurrence-battle.reward.fixed-blessings.${count}`;
const cycleReward = (cycles, base, bonus) =>
  `universe.occurrence-battle.reward.within-cycles.${cycles}.base.${base}.bonus.${bonus}`;
const wave = (index) => `universe.occurrence-battle.wave.${index}`;
const enemy = {
  tramp: "enemy.voidranger-trampler.elite.variant.01",
  guardian: "enemy.guardian-shadow.elite.variant.01",
  reaver: "enemy.voidranger-reaver.minionlv2.variant.01",
  distorter: "enemy.voidranger-distorter.minionlv2.variant.01",
  beetle: "enemy.automaton-beetle.minionlv2.variant.01",
  spider: "enemy.automaton-spider.minionlv2.variant.01",
  shadewalker: "enemy.everwinter-shadewalker.minionlv2.variant.01",
  imaginary: "enemy.imaginary-weaver.minionlv2.variant.01",
  decaying: "enemy.decaying-shadow.elite.variant.01",
  grizzly: "enemy.automaton-grizzly.elite.variant.01",
  hound: "enemy.automaton-hound.minionlv2.variant.01",
  direwolf: "enemy.automaton-direwolf.elite.variant.01",
  frostspawn: "enemy.frostspawn.minion.variant.01",
  ice: "enemy.ice-out-of-space.elite.variant.01",
  flamespawn: "enemy.flamespawn.minion.variant.01",
  blaze: "enemy.blaze-out-of-space.elite.variant.01",
  gatekeeper: "enemy.aurumaton-gatekeeper.elite.variant.01",
};

setOutcome("universe.occurrence.31.variant.13502.choice.01", {
  kinds: ["Obtain", "Battle", "Special"],
  targets: ["Curio"],
  numeric_literals: ["1"],
  parameter_refs: [
    progressive(13502),
    stage("nildis-wildboar"),
    level,
    enemy.tramp,
    enemy.guardian,
    fixedReward(1),
  ],
  chance_percentages: [
    "48",
    "40",
    "12",
    "32",
    "60",
    "8",
    "16",
    "80",
    "4",
    "0",
    "100",
    "0",
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.31.variant.13502.choice.02", {
  kinds: ["Special"],
  targets: [],
  parameter_refs: [progressive(13502)],
});

setOutcome("universe.occurrence.32.variant.13503.choice.01", {
  kinds: ["Obtain", "Battle", "Special"],
  targets: ["CosmicFragments"],
  numeric_literals: ["100"],
  parameter_refs: [
    progressive(13503),
    stage("nildis-robot"),
    level,
    enemy.tramp,
    enemy.guardian,
    fixedReward(1),
  ],
  chance_percentages: [
    "60",
    "25",
    "15",
    "40",
    "50",
    "10",
    "20",
    "75",
    "5",
    "0",
    "100",
    "0",
  ],
  unspecified_random_policy: "StableUniformOrderedCandidates",
});
setOutcome("universe.occurrence.32.variant.13503.choice.02", {
  kinds: ["Special"],
  targets: [],
  parameter_refs: [progressive(13503)],
});

const rockBattle = {
  kinds: ["Battle"],
  targets: [],
  parameter_refs: [
    stage("rock-paper-scissors"),
    level,
    wave(1),
    enemy.reaver,
    enemy.reaver,
    enemy.distorter,
    wave(2),
    enemy.beetle,
    enemy.beetle,
    enemy.spider,
    fixedReward(2),
    rewardPaths,
  ],
};
for (const variant of [13401, 13402]) {
  setOutcome(`universe.occurrence.33.variant.${variant}.choice.01`, rockBattle);
  setOutcome(`universe.occurrence.33.variant.${variant}.choice.02`, {
    kinds: ["Lose"],
    targets: ["CosmicFragments"],
    numeric_literals: ["100"],
  });
}

const tavern = [
  {
    variant: 13601,
    paths: ["universe.path.preservation", "universe.path.nihility"],
    first: [enemy.shadewalker, enemy.shadewalker, enemy.guardian],
    second: [enemy.imaginary, enemy.imaginary, enemy.decaying],
    both: [enemy.guardian, enemy.decaying],
  },
  {
    variant: 13602,
    paths: ["universe.path.elation", "universe.path.hunt"],
    first: [enemy.grizzly],
    second: [enemy.hound, enemy.hound, enemy.direwolf],
    both: [enemy.direwolf, enemy.grizzly],
  },
  {
    variant: 13603,
    paths: ["universe.path.remembrance", "universe.path.destruction"],
    first: [enemy.frostspawn, enemy.frostspawn, enemy.ice],
    second: [enemy.flamespawn, enemy.flamespawn, enemy.blaze],
    both: [enemy.ice, enemy.blaze],
  },
];
for (const entry of tavern) {
  for (const [index, enemies, paths] of [
    [1, entry.first, [entry.paths[0]]],
    [2, entry.second, [entry.paths[1]]],
    [3, entry.both, entry.paths],
  ]) {
    setOutcome(
      `universe.occurrence.34.variant.${entry.variant}.choice.${String(index).padStart(2, "0")}`,
      {
        kinds: ["Battle"],
        targets: [],
        parameter_refs: [
          stage(`tavern-${entry.variant}-${index}`),
          level,
          ...enemies,
          fixedReward(index === 3 ? 2 : 1),
          rewardPaths,
          ...paths,
        ],
      },
    );
  }
}

setOutcome("universe.occurrence.35.variant.13701.choice.01", {
  kinds: ["Battle"],
  targets: [],
  parameter_refs: [
    stage("periodic-demon-lord"),
    level,
    enemy.gatekeeper,
    cycleReward(4, 1, 1),
    rewardPaths,
  ],
});

const sourceReview = {
  schema_revision: "starclock.goal07-occurrence-source-review.v1",
  partition_id: "G07-P4-M13-S06",
  reviewed_on: "2026-07-28",
  frozen_source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  sources: [
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Nildis_(Wildboar)",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "cf2ed899a8f7f386ee6e51fda4264cf673801a92f01e53eeea86218471067d9a",
      facts: [
        "attempt-1:curio-48:battle-40:blank-12",
        "attempt-2:curio-32:battle-60:blank-8",
        "attempt-3:curio-16:battle-80:blank-4",
        "attempt-4:battle-100",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Nildis_(Robot)",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "b7385048a01a6e08cd97130d7b75831d54714c38062d7705a99270532ed372a7",
      facts: [
        "attempt-1:fragments-60:battle-25:blank-15",
        "attempt-2:fragments-40:battle-50:blank-10",
        "attempt-3:fragments-20:battle-75:blank-5",
        "attempt-4:battle-100",
        "success-reward:cosmic-fragments-100",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Rock%2C_Paper%2C_Scissors",
      section: "Possible Outcomes and Simulated Universe battle waves",
      evidence_sha256:
        "18ab9501977065d352dd1cdb79e119bb9942c47ba23910ccae3f21baff0e810c",
      facts: [
        "battle-victory:blessings-2",
        "security-cost:cosmic-fragments-100",
        "battle:waves-2:reaver-distorter:beetle-spider",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Tavern",
      section: "Possible Outcomes and Simulated Universe variants",
      evidence_sha256:
        "429a3dc8184284890be1dcf2ead475054ee4c3bd65a4d914647416077e6186a6",
      facts: [
        "variant-13601:preservation-or-nihility",
        "variant-13602:elation-or-hunt",
        "variant-13603:remembrance-or-destruction",
        "single-team:battle:path-blessing-1",
        "both-teams:battle:path-blessings-2",
      ],
    },
    {
      url: "https://honkai-star-rail.fandom.com/wiki/Periodic_Demon_Lord",
      section: "Possible Outcomes — Simulated Universe",
      evidence_sha256:
        "0aed86d08e00c83338d816373653287cab98fdd5c2d4af3010002a5dc9ec83a9",
      facts: [
        "battle-victory:blessing-1",
        "victory-within-cycles-4:additional-blessing-1",
        "variant-1:aurumaton-gatekeeper",
      ],
    },
    ...[
      ["13401", "64fadc6ed3b20f907986430c68e1bce494d68d034f6f5b6b008cc8b42e18b923"],
      ["13502", "edf7c7e2f2893572ab737c7ae928d147a927356fb20d8f8940fc1d55e81c7fbf"],
      ["13503", "8e62d9129401c64d6da2b2d59ce44cf5d1bf472c0a4e70d7f3f5508174a9808d"],
      ["13601", "1f2bdb0badf14e527ccd0ff0cb6d6207d13fdc2c68d4fe6a80edd090bbfb032e"],
      ["13701", "c58776c8f9ed3a4746020d200d56e61be660def360b0b90365399fbd9c03aaef"],
    ].map(([option, digest]) => ({
      url: `https://raw.githubusercontent.com/Dimbreath/turnbasedgamedata/fd978d6ef09f941fba644c731ab54abd6f7c3568/Config/Level/Rogue/RogueDialogue/Event00${option}/Opt00${option}.json`,
      section: "OptionList",
      evidence_sha256: digest,
      facts: [`source-option:${option}`],
    })),
  ],
};

const encodedChoices = `${JSON.stringify(choices, null, 2)}\n`;
const encodedReview = `${JSON.stringify(sourceReview, null, 2)}\n`;
const encodedIndex = `${JSON.stringify(buildIndex(), null, 2)}\n`;
if (write) {
  fs.writeFileSync(choicesPath, encodedChoices);
  fs.mkdirSync(path.dirname(reviewPath), { recursive: true });
  fs.writeFileSync(reviewPath, encodedReview);
  fs.writeFileSync(indexPath, encodedIndex);
  console.log("Refined Goal 07 Occurrence S06 source evidence and pack index.");
} else {
  assert(originalChoices === encodedChoices, "Occurrence S06 normalized choices drifted");
  assert(fs.readFileSync(reviewPath, "utf8") === encodedReview, "Occurrence S06 source review drifted");
  assert(fs.readFileSync(indexPath, "utf8") === encodedIndex, "Occurrence S06 pack index drifted");
  console.log("Goal 07 Occurrence S06 source refinement is stable.");
}

function setOutcome(id, patch) {
  const choice = byId.get(id);
  assert(choice, `missing ${id}`);
  choice.costs = [];
  choice.outcomes[0] = {
    kinds: patch.kinds,
    targets: patch.targets,
    numeric_literals: patch.numeric_literals ?? [],
    parameter_refs: patch.parameter_refs ?? [],
    chance_percentages: patch.chance_percentages ?? [],
    unspecified_random_policy: patch.unspecified_random_policy ?? "",
  };
}

function buildIndex() {
  const existing = JSON.parse(fs.readFileSync(indexPath, "utf8"));
  const files = existing.files
    .map(({ file, rows }) => {
      const bytes =
        file === "occurrence-choices.json"
          ? Buffer.from(encodedChoices)
          : fs.readFileSync(path.join(packRoot, file));
      const parsed = JSON.parse(bytes);
      return {
        file,
        bytes: bytes.length,
        rows: Array.isArray(parsed) ? parsed.length : rows,
        sha256: sha256(bytes),
      };
    })
    .sort((left, right) => compare(left.file, right.file));
  return {
    schema: existing.schema,
    files,
    pack_sha256: sha256(
      Buffer.from(
        files.map(({ file, sha256: digest }) => `${file}\0${digest}`).join("\n"),
      ),
    ),
  };
}
function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}
function compare(left, right) {
  return left.localeCompare(right);
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
