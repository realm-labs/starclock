param(
    [string]$CacheRoot = ".cache/content-reference"
)

$ErrorActionPreference = "Stop"

$turnRevision = "fd978d6ef09f941fba644c731ab54abd6f7c3568"
$resRevision = "7b349e39ee0f6f3bf814567995829b99c95e7a93"

function Initialize-SourceRepository {
    param(
        [string]$Remote,
        [string]$Revision,
        [string]$Target,
        [string[]]$SparsePatterns
    )

    if (Test-Path -LiteralPath $Target) {
        if (-not (Test-Path -LiteralPath (Join-Path $Target ".git"))) {
            throw "Source-cache target exists but is not a Git repository: $Target"
        }

        $changes = git -C $Target status --porcelain
        if ($changes) {
            throw "Source-cache repository has local changes; preserve or discard them explicitly: $Target"
        }
    }
    else {
        git clone --filter=blob:none --no-checkout $Remote $Target
    }

    git -C $Target fetch origin $Revision --depth 1
    git -C $Target sparse-checkout init --no-cone
    git -C $Target sparse-checkout set --no-cone $SparsePatterns
    git -C $Target checkout --detach $Revision

    $actual = git -C $Target rev-parse HEAD
    if ($actual.Trim() -ne $Revision) {
        throw "Revision mismatch for $Target. Expected $Revision, got $actual"
    }
}

$resolvedCache = [System.IO.Path]::GetFullPath($CacheRoot)
New-Item -ItemType Directory -Force -Path $resolvedCache | Out-Null

$turnPatterns = @(
    "/README.md",
    "/ExcelOutput/AvatarConfig.json",
    "/ExcelOutput/AvatarPromotionConfig.json",
    "/ExcelOutput/AvatarRankConfig.json",
    "/ExcelOutput/AvatarSkillConfig.json",
    "/ExcelOutput/AvatarSkillTreeConfig.json",
    "/ExcelOutput/EquipmentConfig.json",
    "/ExcelOutput/EquipmentPromotionConfig.json",
    "/ExcelOutput/EquipmentSkillConfig.json",
    "/ExcelOutput/MonsterConfig.json",
    "/ExcelOutput/MonsterTemplateConfig.json",
    "/ExcelOutput/MonsterSkillConfig.json",
    "/ExcelOutput/MonsterStatusConfig.json",
    "/ExcelOutput/StageConfig.json",
    "/ExcelOutput/ActivityRogue*.json",
    "/ExcelOutput/ConstValueRogue.json",
    "/ExcelOutput/FinishWayRogue.json",
    "/ExcelOutput/GuideRogue*.json",
    "/ExcelOutput/Rogue*.json",
    "/ExcelOutput/ScheduleDataRogue.json",
    "/TextMap/TextMapEN.json",
    "/TextMap/TextMapCHS.json",
    "/Config/ConfigCharacter/Avatar/",
    "/Config/ConfigCharacter/Monster/",
    "/Config/ConfigAbility/Avatar/",
    "/Config/ConfigAbility/Monster/",
    "/Config/ConfigAbility/BattleEvent/*Rogue*.json",
    "/Config/ConfigAbility/Level/Level_*Rogue*.json",
    "/Config/ConfigAI/"
)

Initialize-SourceRepository `
    -Remote "https://gitlab.com/Dimbreath/turnbasedgamedata.git" `
    -Revision $turnRevision `
    -Target (Join-Path $resolvedCache "turnbasedgamedata") `
    -SparsePatterns $turnPatterns

Initialize-SourceRepository `
    -Remote "https://github.com/Mar-7th/StarRailRes.git" `
    -Revision $resRevision `
    -Target (Join-Path $resolvedCache "StarRailRes") `
    -SparsePatterns @("/README.md", "/LICENSE", "/info.json", "/index_new/")

Write-Output "Pinned source cache is ready at $resolvedCache"
