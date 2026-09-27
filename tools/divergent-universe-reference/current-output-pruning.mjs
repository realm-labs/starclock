// Remove only unchanged, index-owned outputs absent from the current contract.
import fs from "node:fs/promises";
import path from "node:path";
import { sha256 } from "./lib/common.mjs";

export async function pruneObsoletePackFiles(root, contracts, check) {
  const packRoot = path.resolve(root, "content-reference/divergent-universe-v1");
  const indexPath = path.join(packRoot, "pack-index.json");
  if (!(await fs.stat(indexPath).catch((error) => {
    if (error.code === "ENOENT") return null;
    throw error;
  }))) return [];
  const [index] = JSON.parse(await fs.readFile(indexPath, "utf8"));
  if (!Array.isArray(index?.file_digests))
    throw new Error("current pack index has no owned file digests");
  const current = new Set(contracts.map(({ file }) => file));
  const obsolete = index.file_digests.filter(({ file }) => !current.has(file));
  const validated = [];
  for (const entry of obsolete.sort((a, b) => a.file.localeCompare(b.file))) {
    if (!/^[a-z0-9-]+\.json$/u.test(entry.file))
      throw new Error(`unsafe indexed output ${entry.file}`);
    const target = path.resolve(packRoot, entry.file);
    if (path.dirname(target) !== packRoot)
      throw new Error(`indexed output escapes pack root: ${entry.file}`);
    const bytes = await fs.readFile(target);
    if (sha256(bytes) !== entry.sha256 || bytes.length !== entry.bytes)
      throw new Error(`refusing to remove edited generated output ${entry.file}`);
    if (check) throw new Error(`obsolete generated output ${entry.file}`);
    validated.push({ file: entry.file, target });
  }
  // Validate every target before deleting any. Unindexed files are never removed.
  for (const { target } of validated) await fs.unlink(target);
  return validated.map(({ file }) => file);
}
