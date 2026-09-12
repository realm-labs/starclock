#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const arguments_ = process.argv.slice(2);
assert(arguments_.length <= 3,
  "usage: render-visual-review.mjs [ROOT] [OUTPUT] [BROWSER]");
const root = path.resolve(arguments_[0]
  ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."));
const output = path.resolve(root, arguments_[1]
  ?? ".cache/divergent-universe-visual-review");
const browser = locateBrowser(arguments_[2]);
const python = process.env.STARCLOCK_PYTHON
  ?? (process.platform === "win32" ? "python" : "python3");
const result = spawnSync(python, [
  path.join(
    root,
    "tools/divergent-universe-reference/render_visual_review_html.py",
  ),
  root,
  output,
  "--browser",
  browser,
], {
  cwd: root,
  env: { ...process.env, PYTHONDONTWRITEBYTECODE: "1" },
  stdio: "inherit",
});
if (result.error) throw result.error;
assert(result.status === 0, `visual renderer exited with ${result.status}`);

function locateBrowser(explicit) {
  const candidates = [
    explicit,
    process.env.STARCLOCK_BROWSER,
    ...(process.platform === "win32" ? [
      "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
      "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
      "C:/Program Files/Google/Chrome/Application/chrome.exe",
      "C:/Program Files (x86)/Google/Chrome/Application/chrome.exe",
    ] : process.platform === "darwin" ? [
      "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
      "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ] : [
      "/usr/bin/microsoft-edge",
      "/usr/bin/google-chrome",
      "/usr/bin/chromium",
    ]),
  ].filter(Boolean).map((candidate) => path.resolve(candidate));
  const result = candidates.find((candidate) => fs.existsSync(candidate));
  assert(result,
    "Edge/Chrome is unavailable; set STARCLOCK_BROWSER or pass BROWSER");
  return result;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
