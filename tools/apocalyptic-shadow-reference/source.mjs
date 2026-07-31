import { execFileSync } from "node:child_process";
import { access, readFile } from "node:fs/promises";
import path from "node:path";

export const root = path.resolve(".");
export const sourceRoot = path.join(
  root,
  ".cache/content-reference/turnbasedgamedata",
);
export const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";

export async function sourceBytes(relative) {
  const local = path.join(sourceRoot, relative);
  if (await access(local).then(() => true).catch(() => false)) {
    return readFile(local);
  }
  return execFileSync("git", ["-C", sourceRoot, "show", `HEAD:${relative}`], {
    maxBuffer: 256 * 1024 * 1024,
  });
}

export async function sourceJson(relative) {
  return JSON.parse((await sourceBytes(relative)).toString("utf8"));
}

export function sourcePaths() {
  return execFileSync("git", [
    "-C", sourceRoot, "ls-tree", "-r", "--name-only", "HEAD",
  ], { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 })
    .trim().split("\n").filter(Boolean);
}
