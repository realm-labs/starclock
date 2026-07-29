# Isolated Sora system schema

The Goal 13 project uses the repository-pinned, checksum-bound Sora 0.3.0
binary and writes only beneath `config/anomaly-arbitration/` and
`config/anomaly-arbitration-generated/`.

The first schema partition defines eight primary authoring tables for profile,
period, stages, terminal outcomes, participant policies, team slots, loadout
records and progress records. Private integer workbook keys are separated from
stable content keys. Period and stage relationships use typed Sora references;
every table retains bilingual summaries, quality labels, manifest/source IDs,
tags, a complete canonical JSON payload and the explicit non-runtime flag.
