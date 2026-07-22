import path from "node:path";
import process from "node:process";
import { createContext, writeOrCheck } from "./lib/common.mjs";
import { topology } from "./lib/topology.mjs";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(args.find((argument) => !argument.startsWith("--")) ?? ".");
const ctx = await createContext(root);
const outputs = await topology(ctx);
outputs.set("sources.json", [...ctx.evidence.values()].sort((left, right) => left.id.localeCompare(right.id)));
await writeOrCheck(ctx, outputs, check);
console.log(`Standard universe reference pack ${check ? "verified" : "generated"}: ${outputs.size} files, ${[...outputs.values()].reduce((sum, records) => sum + records.length, 0)} rows.`);
