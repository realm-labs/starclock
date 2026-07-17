const MILLION = 1_000_000n;
const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;
const GRAMMAR = /^(-?)(0|[1-9][0-9]*)(?:\.([0-9]{0,5}[1-9]))?$/u;

export function canonicalDecimalToMillionths(value) {
  if (typeof value !== "string") throw new Error("canonical decimal must be a string");
  const match = GRAMMAR.exec(value);
  if (!match || value === "-0") throw new Error(`invalid canonical decimal: ${value}`);
  const fraction = match[3] ?? "";
  const magnitude = BigInt(match[2]) * MILLION + BigInt(fraction.padEnd(6, "0") || "0");
  const raw = match[1] === "-" ? -magnitude : magnitude;
  if (raw < I64_MIN || raw > I64_MAX) throw new Error(`canonical decimal exceeds i64 millionths: ${value}`);
  return raw;
}

export function isCanonicalDecimal(value) {
  try {
    canonicalDecimalToMillionths(value);
    return true;
  } catch {
    return false;
  }
}
