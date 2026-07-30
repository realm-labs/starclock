#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const outputRoot = path.resolve(option("--cache")
  ?? path.join(root, ".cache/galactic-baseballer-source/public-revisions"));
const offline = process.argv.includes("--offline");
const inventory = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "public-source-inventory.json",
), "utf8"));

function option(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  if (value === undefined || value.startsWith("--"))
    throw new Error(`${name} requires a path`);
  return value;
}

function hash(algorithm, content) {
  return createHash(algorithm).update(content).digest("hex");
}

async function fetchRevision(receipt) {
  const endpoint = new URL("https://honkai-star-rail.fandom.com/api.php");
  endpoint.search = new URLSearchParams({
    action: "query",
    prop: "revisions",
    revids: String(receipt.revision_id),
    rvprop: "ids|timestamp|sha1|content",
    rvslots: "main",
    formatversion: "2",
    format: "json",
    origin: "*",
  }).toString();
  const response = await fetch(endpoint, {
    headers: { "user-agent": "Starclock-Reference-Audit/1.0" },
  });
  if (!response.ok)
    throw new Error(`${receipt.title}: HTTP ${response.status}`);
  const payload = await response.json();
  const page = payload.query?.pages?.[0];
  const revision = page?.revisions?.[0];
  const content = revision?.slots?.main?.content;
  if (revision?.revid !== receipt.revision_id || typeof content !== "string")
    throw new Error(`${receipt.title}: pinned revision content missing`);
  if (revision.parentid !== receipt.parent_revision_id
    || revision.timestamp !== receipt.revision_timestamp
    || revision.sha1 !== receipt.mediawiki_sha1)
    throw new Error(`${receipt.title}: MediaWiki revision metadata drift`);
  return content;
}

function verifyContent(receipt, content) {
  if (Buffer.byteLength(content, "utf8") !== receipt.content_bytes)
    throw new Error(`${receipt.title}: byte count drift`);
  if (hash("sha1", content) !== receipt.mediawiki_sha1)
    throw new Error(`${receipt.title}: SHA-1 drift`);
  if (hash("sha256", content) !== receipt.content_sha256)
    throw new Error(`${receipt.title}: SHA-256 drift`);
}

await mkdir(outputRoot, { recursive: true });
for (const receipt of inventory.community_pages) {
  const target = path.join(outputRoot, `${receipt.revision_id}.wikitext`);
  let content;
  if (offline) {
    content = await readFile(target, "utf8");
  } else {
    content = await fetchRevision(receipt);
    verifyContent(receipt, content);
    await writeFile(target, content);
  }
  verifyContent(receipt, content);
}

console.log(
  `Galactic Baseballer public revisions ${offline ? "verified" : "cached"}: `
  + `${inventory.community_pages.length} pinned MediaWiki revisions at `
  + outputRoot,
);
