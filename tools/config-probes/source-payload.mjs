import crypto from "node:crypto";
import fs from "node:fs";

export function verifyOptionalSourcePayload(file, expectedSha256, label) {
  if (!fs.existsSync(file)) return expectedSha256;
  const actual = crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
  if (actual !== expectedSha256) throw new Error(`${label} prepared source payload drifted`);
  return actual;
}
