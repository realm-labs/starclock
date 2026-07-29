import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const root = path.resolve(process.argv[2] ?? ".");
const output = path.resolve(
  process.argv[3] ?? "evidence/unknowable-domain-reference-v1/rendered",
);
assert(!fs.existsSync(output), `refusing to overwrite ${output}`);
fs.mkdirSync(output, { recursive: true });

const require = createRequire(import.meta.url);
let artifactEntry;
try {
  artifactEntry = require.resolve("@oai/artifact-tool");
} catch {
  throw new Error(
    "@oai/artifact-tool is unavailable; set NODE_PATH to the bundled " +
      "workspace dependency node_modules directory",
  );
}
const artifactPackage = JSON.parse(
  fs.readFileSync(
    path.resolve(path.dirname(artifactEntry), "..", "package.json"),
    "utf8",
  ),
);
const { FileBlob, SpreadsheetFile } = await import(
  pathToFileURL(artifactEntry).href
);
const schema = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "config",
      "unknowable-domain-generated",
      "schema.lock",
    ),
    "utf8",
  ),
).schema;
const workbookNames = [
  "UnknowableDomain.xlsx",
  "UnknowableDomainBindings.xlsx",
  "UnknowableDomainReview.xlsx",
];
const rendered = [];
let ordinal = 0;

for (const workbookName of workbookNames) {
  const input = await FileBlob.load(
    path.join(root, "config", "unknowable-domain", "data", workbookName),
  );
  const workbook = await SpreadsheetFile.importXlsx(input);
  const tables = schema.tables.filter(
    (table) => table.source.file === workbookName,
  );
  for (const table of tables) {
    ordinal += 1;
    const range = `A1:${columnName(table.fields.length)}12`;
    const image = await workbook.render({
      sheetName: table.source.sheet,
      range,
      format: "png",
      scale: 1,
      headers: false,
    });
    const bytes = Buffer.from(await image.arrayBuffer());
    const filename =
      `${String(ordinal).padStart(2, "0")}-` +
      `${workbookName.replace(/\.xlsx$/u, "")}-` +
      `${table.source.sheet}.png`;
    fs.writeFileSync(path.join(output, filename), bytes);
    rendered.push({
      file: workbookName,
      sheet: table.source.sheet,
      range,
      image: filename,
      sha256: sha256(bytes),
    });
  }
}
assert(rendered.length === 65, "visual-review sheet denominator differs");
const manifest = {
  schema_revision: "starclock.unknowable-domain-visual-render.v1",
  renderer: {
    name: "@oai/artifact-tool",
    version: artifactPackage.version,
    range_policy: "rows 1-12 across every used schema column",
  },
  sheet_count: rendered.length,
  sheets: rendered,
};
const manifestBytes = Buffer.from(
  `${JSON.stringify(manifest, null, 2)}\n`,
  "utf8",
);
fs.writeFileSync(path.join(output, "render-manifest.json"), manifestBytes);
console.log(
  `Rendered ${rendered.length} Unknowable Domain sheets with ` +
    `@oai/artifact-tool ${artifactPackage.version}; manifest ` +
    `${sha256(manifestBytes)}.`,
);

function columnName(index) {
  let value = index;
  let result = "";
  while (value > 0) {
    value -= 1;
    result = String.fromCharCode(65 + (value % 26)) + result;
    value = Math.floor(value / 26);
  }
  return result;
}

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}
