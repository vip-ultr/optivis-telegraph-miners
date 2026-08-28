// Traffic loop: fire many varied queries at our miners to build scoring history.
// Reuses the official @x402/evm + @x402/fetch payment handshake.
//
// Usage:
//   node tools/traffic_loop.mjs              (default: 20 queries/mode)
//   node tools/traffic_loop.mjs 50           (50 queries total, round-robin)
//
// Reads BASE_SEPOLIA_PRIVATE_KEY from .env. Key never printed.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { privateKeyToAccount } from "viem/accounts";
import { ExactEvmScheme } from "@x402/evm";
import { wrapFetchWithPaymentFromConfig } from "@x402/fetch";

const __dirname = dirname(fileURLToPath(import.meta.url));
const envPath = join(__dirname, "..", ".env");
try {
  const txt = readFileSync(envPath, "utf8");
  for (const line of txt.split("\n")) {
    const m = line.match(/^\s*([A-Z0-9_]+)\s*=\s*(.*)\s*$/);
    if (m && !process.env[m[1]]) process.env[m[1]] = m[2];
  }
} catch {}

const NODE = "https://devnode.telegraphprotocol.com";

// Varied real queries to exercise both miners across symbols/chains.
const QUERIES = [
  [7311, "/price", { symbols: "sol,eth", vs_currencies: "usd" }, "sol,eth price"],
  [7311, "/price", { symbols: "bonk,jto", vs_currencies: "usd" }, "bonk,jto price"],
  [7311, "/price", { symbols: "usdc,usdt", vs_currencies: "usd" }, "usdc,usdt price"],
  [7311, "/pair", { token: "So11111111111111111111111111111111111111112" }, "SOL pair"],
  [7311, "/pair", { token: "J1toso1uCk3shnUQ4mAYiT4LWQH7z2Z6i2T3bY9p1x" }, "JTO pair"],
  [7312, "/protocol", {}, "uniswap tvl"],
  [7312, "/protocol/jupiter", {}, "jupiter tvl"],
  [7312, "/protocol/lido", {}, "lido tvl"],
  [7312, "/protocol/marinade", {}, "marinade tvl"],
  [7312, "/protocol/jito", {}, "jito tvl"],
  [7312, "/chain", { chain: "Solana" }, "solana chain tvl"],
  [7312, "/yields", { chain: "Solana" }, "solana yields"],
  [289, "/fx", { base: "usd", target: "eur" }, "usd to eur fx"],
  [289, "/fx", { base: "usd", target: "ngn" }, "usd to ngn fx"],
  [289, "/financials", { symbols: "sol,eth", vs_currencies: "usd" }, "sol,eth financials"],
  [7313, "/gas", {}, "base gas price"],
];

const key = process.env.BASE_SEPOLIA_PRIVATE_KEY;
if (!key || key === "0x" || key.includes("PASTE")) {
  console.error("ERROR: set BASE_SEPOLIA_PRIVATE_KEY in .env");
  process.exit(1);
}
const account = privateKeyToAccount(key);
const signer = {
  address: account.address,
  signTypedData: (msg) => account.signTypedData(msg),
  signTransaction: (args) => account.signTransaction(args),
  getTransactionCount: () => Promise.resolve(0),
};
const scheme = new ExactEvmScheme(signer);
const config = {
  schemes: [{ network: "eip155:84532", x402Version: 2, client: scheme }],
};
const fetchWithPayment = wrapFetchWithPaymentFromConfig(globalThis.fetch, config);

async function ask(minerId, endpoint, payload, label) {
  const url = `${NODE}/engine/v1/ask/${minerId}`;
  for (let attempt = 1; attempt <= 4; attempt++) {
    try {
      const res = await fetchWithPayment(url, {
        method: "POST",
        headers: { "Content-Type": "application/json", "Connection": "keep-alive" },
        body: JSON.stringify({ method: "GET", endpoint, payload }),
      });
      const text = await res.text();
      let ok = res.status === 200;
      let snippet = "";
      try {
        const j = JSON.parse(text);
        snippet = JSON.stringify(j.result).slice(0, 160);
        if (j.result && j.cost_usd) ok = true;
      } catch {
        snippet = text.slice(0, 160);
      }
      console.log(`[${ok ? "OK" : "FAIL"}] ${minerId} ${endpoint} (${label}) HTTP ${res.status} ${snippet}`);
      return ok;
    } catch (e) {
      if (attempt === 4) {
        console.log(`[ERR] ${minerId} ${endpoint} (${label}): ${e.message}`);
        return false;
      }
      await new Promise((r) => setTimeout(r, 2000));
    }
  }
}

const total = parseInt(process.argv[2] || "20", 10);
let done = 0,
  okCount = 0;
while (done < total) {
  for (const [id, ep, pl, label] of QUERIES) {
    if (done >= total) break;
    if (await ask(id, ep, pl, label)) okCount++;
    done++;
    await new Promise((r) => setTimeout(r, 2000)); // space out to avoid CoinGecko 429
  }
}
console.log(`\nDONE: ${done} queries, ${okCount} OK`);
