#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const refresh = args.includes("--refresh");
const root = path.resolve(".");
const output = path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "public-source-inventory.json",
);
const wikiPages = [
  "The Legend of Galactic Baseballer",
  "The Legend of Galactic Baseballer/Adventure Index",
  "The Legend of Galactic Baseballer/Cosmic Reputation",
  "The Legend of Galactic Baseballer/Cosmic Store",
  "The Legend of Galactic Baseballer/Planets",
  "Legend of the Galactic Baseballer: Demon King",
  "Legend of the Galactic Baseballer: Demon King/Adventure Index",
  "Legend of the Galactic Baseballer: Demon King/Adventure Strategy",
  "Legend of the Galactic Baseballer: Demon King/Cosmic Reputation",
  "Legend of the Galactic Baseballer: Demon King/Cosmic Store",
  "Legend of the Galactic Baseballer: Demon King/Planets",
];
const officialPages = [
  {
    id: "hoyolab-version-2.2-update",
    url: "https://www.hoyolab.com/article/28286762",
    role: "original-release-version-entry-and-unlock",
  },
  {
    id: "hoyolab-original-event-notice",
    url: "https://www.hoyolab.com/article/29125952",
    role: "original-released-rules-and-event-window",
  },
  {
    id: "hoyowiki-original-entry-2508",
    url: "https://wiki.hoyolab.com/pc/hsr/entry/2508?lang=en-us",
    role: "original-released-mechanic-cross-check",
  },
  {
    id: "hoyolab-version-3.3-update",
    url: "https://www.hoyolab.com/article/38894296",
    role: "demon-king-release-version-entry-and-window",
  },
  {
    id: "hoyolab-version-3.4-update",
    url: "https://www.hoyolab.com/article/39751178",
    role: "released-demon-king-corrections",
  },
];

function sha256(text) {
  return createHash("sha256").update(text).digest("hex");
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

async function wikiReceipt(title) {
  const endpoint = new URL("https://honkai-star-rail.fandom.com/api.php");
  endpoint.search = new URLSearchParams({
    action: "query",
    prop: "revisions",
    titles: title,
    rvprop: "ids|timestamp|sha1|content",
    rvslots: "main",
    formatversion: "2",
    format: "json",
    origin: "*",
  }).toString();
  const response = await fetch(endpoint, {
    headers: { "user-agent": "Starclock-Reference-Audit/1.0" },
  });
  if (!response.ok) throw new Error(`${title}: HTTP ${response.status}`);
  const payload = await response.json();
  const page = payload.query?.pages?.[0];
  const revision = page?.revisions?.[0];
  const content = revision?.slots?.main?.content;
  if (page?.missing || revision === undefined || typeof content !== "string")
    throw new Error(`${title}: page/revision content missing`);
  return {
    provider: "Honkai Star Rail Wiki (Fandom)",
    quality: "PublicCommunityCrossCheck",
    title,
    page_id: page.pageid,
    url: `https://honkai-star-rail.fandom.com/wiki/${encodeURIComponent(
      title.replaceAll(" ", "_"),
    )}`,
    revision_id: revision.revid,
    parent_revision_id: revision.parentid,
    revision_timestamp: revision.timestamp,
    mediawiki_sha1: revision.sha1,
    content_bytes: Buffer.byteLength(content, "utf8"),
    content_sha256: sha256(content),
  };
}

let wikiReceipts;
if (refresh) {
  wikiReceipts = await Promise.all(wikiPages.map(wikiReceipt));
  wikiReceipts.sort((left, right) => compareText(left.title, right.title));
} else {
  const existing = JSON.parse(await readFile(output, "utf8"));
  wikiReceipts = existing.community_pages;
}
const payload = {
  schema_revision: "starclock.galactic-baseballer-public-source-inventory.v1",
  access_date: "2026-07-30",
  official_pages: officialPages.map((page) => ({
    ...page,
    provider: "HoYoverse / HoYoLAB",
    quality: "ExactPublicText",
    digest_boundary: "per-claim receipt freezes in G16-P0-B3",
  })),
  community_pages: wikiReceipts,
  boundary: {
    public_pages_are_cross_checks_not_structured_membership_authority: true,
    official_claim_digests_freeze_with_normalized_facts: true,
    story_gallery_and_reward_payload_pages_excluded: true,
  },
};
if (payload.official_pages.length !== 5 || payload.community_pages.length !== 11)
  throw new Error("Goal 16 public source denominator drift");
payload.canonical_sha256 = sha256(JSON.stringify({
  official_pages: payload.official_pages,
  community_pages: payload.community_pages,
}));

if (check) {
  const expected = JSON.parse(await readFile(output, "utf8"));
  if (JSON.stringify(expected) !== JSON.stringify(payload))
    throw new Error("Galactic Baseballer public source inventory drift");
  console.log(
    `Galactic Baseballer public sources verified ` +
    `(${officialPages.length} official, ${wikiReceipts.length} community).`,
  );
} else {
  if (!refresh) throw new Error("initial generation requires --refresh");
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, `${JSON.stringify(payload, null, 2)}\n`);
  console.log(
    `Wrote Galactic Baseballer public sources ` +
    `(${officialPages.length} official, ${wikiReceipts.length} community).`,
  );
}
