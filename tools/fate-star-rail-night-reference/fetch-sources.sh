#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 CACHE_ROOT [CLEAN_SEED_ROOT]" >&2
  exit 2
fi

cache_root="$1"
seed_root="${2:-}"
turn_remote="https://gitlab.com/Dimbreath/turnbasedgamedata.git"
res_remote="https://github.com/Mar-7th/StarRailRes.git"
turn_revision="fd978d6ef09f941fba644c731ab54abd6f7c3568"
res_revision="7b349e39ee0f6f3bf814567995829b99c95e7a93"
turn_target="${cache_root}/turnbasedgamedata"
res_target="${cache_root}/StarRailRes"

copy_seed() {
  local seed="$1"
  local target="$2"
  local revision="$3"
  local remote="$4"

  [[ -d "${seed}/.git" ]]
  [[ "$(git -C "${seed}" rev-parse HEAD)" == "${revision}" ]]
  [[ "$(git -C "${seed}" remote get-url origin)" == "${remote}" ]]
  [[ -z "$(git -C "${seed}" status --porcelain)" ]]
  git -C "${seed}" cat-file -e "${revision}^{commit}"
  git -C "${seed}" fsck --connectivity-only --no-dangling

  mkdir -p "${target}"
  if [[ "$(uname -s)" == "Darwin" ]]; then
    cp -cR "${seed}/." "${target}"
  else
    cp -a --reflink=auto "${seed}/." "${target}"
  fi
}

mkdir -p "${cache_root}"

if [[ ! -d "${turn_target}/.git" ]]; then
  if [[ -n "${seed_root}" ]]; then
    copy_seed "${seed_root}/turnbasedgamedata" "${turn_target}" \
      "${turn_revision}" "${turn_remote}"
  else
    git -c http.version=HTTP/1.1 clone --filter=blob:none --no-checkout \
      "${turn_remote}" "${turn_target}"
  fi
fi

git -C "${turn_target}" config http.version HTTP/1.1
[[ "$(git -C "${turn_target}" remote get-url origin)" == "${turn_remote}" ]]
[[ -z "$(git -C "${turn_target}" status --porcelain)" ]]
if ! git -C "${turn_target}" cat-file -e "${turn_revision}^{commit}"; then
  git -C "${turn_target}" fetch origin "${turn_revision}" --depth 1
fi
git -C "${turn_target}" sparse-checkout init --no-cone
git -C "${turn_target}" sparse-checkout set --no-cone \
  '/README.md' \
  '/ExcelOutput/Fate*.json' \
  '/ExcelOutput/StageConfig.json' \
  '/ExcelOutput/BattleArea.json' \
  '/ExcelOutput/BattleAreaUnifiedConfig.json' \
  '/ExcelOutput/BattleEventConfig.json' \
  '/ExcelOutput/BattleTargetConfig.json' \
  '/ExcelOutput/MazeBuff.json' \
  '/ExcelOutput/MonsterConfig.json' \
  '/ExcelOutput/MonsterTemplateConfig.json' \
  '/ExcelOutput/MonsterSkillConfig.json' \
  '/ExcelOutput/MonsterStatusConfig.json' \
  '/TextMap/TextMapCHS.json' \
  '/TextMap/TextMapEN.json' \
  '/Config/Gameplays/Fate/' \
  '/Config/ConfigAI/*FateRin*.json' \
  '/Config/ConfigCharacter/BattleEvent/Activity_FateRin_*.json' \
  '/Config/ConfigAbility/Monster/*FateRin*.json' \
  '/Config/ConfigAnimEvents/Monster/Designer/*FateRin*.json' \
  '/Config/ConfigCharacter/Monster/Monster_XP_Minion01_00_Config.json' \
  '/Config/ConfigCharacter/Monster/Monster_XP_Minion04_00_Config.json' \
  '/Config/ConfigCharacter/Monster/Monster_XP_Elite01_00_Config.json' \
  '/Config/ConfigCharacter/Monster/Monster_AML_Minion02_00_Config.json' \
  '/Config/ConfigCharacter/Monster/Monster_AML_Minion03_00_Config.json' \
  '/Config/ConfigAI/Monster_Common_SequenceThree_AI.json' \
  '/Config/ConfigAI/Monster_XP_Minion04_00_AI.json' \
  '/Config/ConfigAI/Monster_XP_Elite01_00_AI.json' \
  '/Config/ConfigAI/Monster_AML_Minion02_00_AI.json' \
  '/Config/ConfigAI/Monster_AML_Minion03_00_AI.json'
git -C "${turn_target}" checkout --detach "${turn_revision}"
git -C "${turn_target}" sparse-checkout reapply

if [[ ! -d "${res_target}/.git" ]]; then
  if [[ -n "${seed_root}" ]]; then
    copy_seed "${seed_root}/StarRailRes" "${res_target}" \
      "${res_revision}" "${res_remote}"
  else
    git -c http.version=HTTP/1.1 clone --filter=blob:none --no-checkout \
      "${res_remote}" "${res_target}"
  fi
fi

git -C "${res_target}" config http.version HTTP/1.1
[[ "$(git -C "${res_target}" remote get-url origin)" == "${res_remote}" ]]
[[ -z "$(git -C "${res_target}" status --porcelain)" ]]
if ! git -C "${res_target}" cat-file -e "${res_revision}^{commit}"; then
  git -C "${res_target}" fetch origin "${res_revision}" --depth 1
fi
git -C "${res_target}" sparse-checkout init --no-cone
git -C "${res_target}" sparse-checkout set --no-cone \
  '/README.md' '/LICENSE' '/info.json' '/index_new/cn/' '/index_new/en/'
git -C "${res_target}" checkout --detach "${res_revision}"
git -C "${res_target}" sparse-checkout reapply

for repository in "${turn_target}" "${res_target}"; do
  [[ -z "$(git -C "${repository}" status --porcelain)" ]]
  [[ -z "$(git -C "${repository}" symbolic-ref --quiet --short HEAD || true)" ]]
  git -C "${repository}" cat-file -e 'HEAD^{commit}'
  git -C "${repository}" fsck --connectivity-only --no-dangling
done

[[ "$(git -C "${turn_target}" rev-parse HEAD)" == "${turn_revision}" ]]
[[ "$(git -C "${res_target}" rev-parse HEAD)" == "${res_revision}" ]]

echo "Pinned Goal 19 source cache is ready at ${cache_root}"
