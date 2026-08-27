// x402 payer using the official @x402/evm + @x402/fetch libraries (viem signer).
// Handles the full 402 handshake: EIP-712 typed-data signing (ERC-3009 exact),
// PAYMENT-SIGNATURE header, facilitator verification. No hand-rolled envelope.
//
// Usage:
//   node tools/x402_payer.mjs 7311 "price of SOL in usd"
//   node tools/x402_payer.mjs            (runs the default probe set)
//
// Reads BASE_SEPOLIA_PRIVATE_KEY from .env (gitignored). Key never printed.
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
} catch (e) {
  console.error("could not read .env:", e.message);
}

const NODE = "https://devnode.telegraphprotocol.com";
const PROBES = [
  [7311, "What is the price of SOL in USD?"],
  [7311, "price of ETH and BONK in usd"],
  [7312, "What is the TVL of uniswap?"],
  [7312, "TVL of jupiter"],
];

const key = process.env.BASE_SEPOLIA_PRIVATE_KEY;
if (!key || key === "0x" || key.includes("PASTE")) {
  console.error("ERROR: set BASE_SEPOLIA_PRIVATE_KEY in .env");
  process.exit(1);
}

const account = privateKeyToAccount(key);
// Build an x402-compatible signer: needs { address, signTypedData }.
// viem's privateKeyToAccount has both, but the wrapper wants them surfaced.
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

async function ask(minerId, query) {
  // Direct-routed: force our miner. Body shape per docs:
  // { method, endpoint, payload }. For CRYPTO_PRICE use /price; TVL use /protocol.
  const endpoint = minerId === 7311 ? "/price" : "/protocol";
  const payload =
    minerId === 7311
      ? { symbols: "sol,eth", vs_currencies: "usd" }
      : { protocol: "uniswap" };
  const url = `${NODE}/engine/v1/ask/${minerId}`;
  let res, text;
  for (let attempt = 1; attempt <= 4; attempt++) {
    try {
      res = await fetchWithPayment(url, {
        method: "POST",
        headers: { "Content-Type": "application/json", "Connection": "keep-alive" },
        body: JSON.stringify({ method: "GET", endpoint, payload }),
      });
      text = await res.text();
      break;
    } catch (e) {
      if (attempt === 4) throw e;
      await new Promise((r) => setTimeout(r, 2000));
    }
  }
  console.log(`=== miner ${minerId}: ${query} ===`);
  console.log("HTTP", res.status);
  try {
    console.log(JSON.stringify(JSON.parse(text), null, 1).slice(0, 900));
  } catch {
    console.log(text.slice(0, 900));
  }
  console.log("-".repeat(50));
}

const args = process.argv.slice(2);
const probes = args.length >= 2 ? [[parseInt(args[0]), args[1]]] : PROBES;
for (const [id, q] of probes) {
  await ask(id, q);
}
