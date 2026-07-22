import { spawn } from "node:child_process";
import readline from "node:readline";

const binary = process.argv[2];
if (!binary) throw new Error("usage: node examples/agent-control/stdio-discovery.mjs PATH_TO_STARCLOCK_BINARY");

const child = spawn(binary, ["mcp", "serve", "--transport", "stdio"], {
  stdio: ["pipe", "pipe", "pipe"],
});
const frames = readline.createInterface({ input: child.stdout, crlfDelay: Infinity });
const pending = new Map();
let nextId = 1;
frames.on("line", (line) => {
  const message = JSON.parse(line);
  const callback = pending.get(message.id);
  if (callback) {
    pending.delete(message.id);
    callback(message);
  }
});

function write(message) {
  child.stdin.write(`${JSON.stringify(message)}\n`);
}

function request(method, params) {
  const id = nextId++;
  write({ jsonrpc: "2.0", id, method, params });
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error(`${method} timed out`)), 10_000);
    pending.set(id, (message) => {
      clearTimeout(timeout);
      if (message.error) reject(new Error(JSON.stringify(message.error)));
      else resolve(message.result);
    });
  });
}

const initialized = await request("initialize", {
  protocolVersion: "2025-11-25",
  capabilities: {},
  clientInfo: { name: "starclock-stdio-example", version: "1" },
});
write({ jsonrpc: "2.0", method: "notifications/initialized", params: {} });
const listed = await request("tools/list", {});
const names = listed.tools.map((tool) => tool.name).sort();
if (initialized.serverInfo.name !== "starclock-mcp" || names.length !== 7) {
  throw new Error("unexpected frozen MCP contract");
}
console.log(`${initialized.serverInfo.name} ${initialized.protocolVersion}: ${names.join(", ")}`);
child.stdin.end();
const exitCode = await new Promise((resolve) => child.once("close", resolve));
if (exitCode !== 0) {
  throw new Error(`server exited ${exitCode}: ${await streamText(child.stderr)}`);
}

async function streamText(stream) {
  let value = "";
  for await (const chunk of stream) value += chunk;
  return value;
}
