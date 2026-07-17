# Production configuration

`project.toml` is the only production Sora project. `data/*.xlsx` is the
designer-authoritative source, `generated/config.sora` is the runtime bundle,
and generated diagnostic JSON is review evidence only. The normalized Version
4.4 JSON pack is accepted only by the one-time/no-overwrite workbook bootstrap;
it is never opened by runtime code.

Run `node tools/config-production/bootstrap.mjs --output config/data` only when
the output directory does not exist. Normal regeneration uses
`node tools/config-production/verify.mjs`; it never writes designer workbooks.
