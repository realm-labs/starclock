#!/usr/bin/env bash
set -euo pipefail

cache_root="${1:-.cache/content-reference}"
turn_remote="https://gitlab.com/Dimbreath/turnbasedgamedata.git"
res_remote="https://github.com/Mar-7th/StarRailRes.git"
turn_revision="fd978d6ef09f941fba644c731ab54abd6f7c3568"
res_revision="7b349e39ee0f6f3bf814567995829b99c95e7a93"
turn_target="${cache_root}/turnbasedgamedata"
res_target="${cache_root}/StarRailRes"

mkdir -p "${cache_root}"

if [[ ! -d "${turn_target}/.git" ]]; then
  git -c http.version=HTTP/1.1 clone --filter=blob:none --no-checkout \
    "${turn_remote}" "${turn_target}"
fi
git -C "${turn_target}" config http.version HTTP/1.1
[[ "$(git -C "${turn_target}" remote get-url origin)" == "${turn_remote}" ]]
if [[ "$(git -C "${turn_target}" config --bool core.sparseCheckout || true)" == "true" ]] \
  && [[ -n "$(git -C "${turn_target}" status --porcelain)" ]]; then
  echo "source cache has local changes: ${turn_target}" >&2
  exit 1
fi
git -C "${turn_target}" fetch origin "${turn_revision}" --depth 1
git -C "${turn_target}" sparse-checkout init --no-cone
{
  printf '%s\n' \
    '/README.md' \
    '/ExcelOutput/GuideRogueData.json' \
    '/ExcelOutput/GuideRogueTab.json' \
    '/ExcelOutput/RogueActivityResidentConfig.json' \
    '/ExcelOutput/RoguePersona*.json' \
    '/ExcelOutput/RogueTourn*.json' \
    '/ExcelOutput/StageConfig.json' \
    '/TextMap/TextMapEN.json' \
    '/TextMap/TextMapCHS.json' \
    '/Config/ConfigAdventureModifier/AdventureModifier_Rogue_S3.json' \
    '/Config/ConfigAdventureModifier/AdventureModifier_Rogue_Tourn1.json' \
    '/Config/Level/GroupTemplateGraph/03_Rogue/RogueTourn230/*.json' \
    '/Config/Level/Maze/MazeRogue/RogueTourn/*.json' \
    '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_Ability_S3.json' \
    '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_Ability_S3.layout.json' \
    '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_HEX_S3.json' \
    '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_HEX_S3.layout.json' \
    '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_Miracle_S3.json' \
    '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_Miracle_S3.layout.json' \
    '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_Recipe_S3.json' \
    '/Config/ConfigAbility/Level/Level_RogueBuff_Ability_Recipe_S3.layout.json'
  git -C "${turn_target}" ls-tree -r --name-only "${turn_revision}" |
    awk 'BEGIN { IGNORECASE=1 } /GridFight/ { print "/" $0 }'
} | git -C "${turn_target}" sparse-checkout set --no-cone --stdin
git -C "${turn_target}" checkout --detach "${turn_revision}"

if [[ ! -d "${res_target}/.git" ]]; then
  git -c http.version=HTTP/1.1 clone --filter=blob:none --no-checkout \
    "${res_remote}" "${res_target}"
fi
git -C "${res_target}" config http.version HTTP/1.1
[[ "$(git -C "${res_target}" remote get-url origin)" == "${res_remote}" ]]
if [[ "$(git -C "${res_target}" config --bool core.sparseCheckout || true)" == "true" ]] \
  && [[ -n "$(git -C "${res_target}" status --porcelain)" ]]; then
  echo "source cache has local changes: ${res_target}" >&2
  exit 1
fi
git -C "${res_target}" fetch origin "${res_revision}" --depth 1
git -C "${res_target}" sparse-checkout init --no-cone
git -C "${res_target}" sparse-checkout set --no-cone \
  '/README.md' '/LICENSE' '/info.json' '/index_new/'
git -C "${res_target}" checkout --detach "${res_revision}"

for repository in "${turn_target}" "${res_target}"; do
  [[ -z "$(git -C "${repository}" status --porcelain)" ]]
  git -C "${repository}" fsck --connectivity-only --no-dangling
done
[[ "$(git -C "${turn_target}" rev-parse HEAD)" == "${turn_revision}" ]]
[[ "$(git -C "${res_target}" rev-parse HEAD)" == "${res_revision}" ]]

echo "Pinned Goal 12 source cache is ready at ${cache_root}"
