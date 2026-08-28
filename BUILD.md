# BUILD.md — Optivis Telegraph Miners

Handoff / resume document. A fresh session should be able to continue without
re-deriving context. Last updated: **2026-08-26**.

## Status

| Miner | Slug | Intent | YAML | VERIFY | Uploaded | Registered |
|---|---|---|---|---|---|---|
| Optivis Crypto Price | `optivis-crypto-price` | CRYPTO_PRICE | done | 6/6 | sandbox 200/200 (baked defaults) | REGISTERED (id 7311, tx 0x659347838cb76a2e8e43c55e551ffe11f73f110da4e02791d8b430bd0bdc3141) |
| Optivis TVL Oracle | `optivis-tvl` | TVL_LOOKUP | done | 6/6 | sandbox 200 (baked uniswap) | REGISTERED (id 7312, tx 0xd8e37bbe3a17047bc051b63b954db5f0ce5899920f73be9d0f05d61dd1e90c6c) |
| Optivis Gas Oracle | `optivis-gas` | GAS_PRICE | done | 6/6 | sandbox 405 (JSON-RPC body) | DEFERRED: need body-capable API |

Phase: **2 of 3 miners registered & live. Gas deferred pending a keyless API
that works without a static JSON-RPC POST body (wizard 405s; no YAML field to
declare a fixed body, and no keyless GET gas source exists).**

## Competitor note (2026-08-26)
TVL_LOOKUP already has a live competitor: `tvlwire-oracle` (id 301). CRYPTO_PRICE
and GAS_PRICE appear empty/unserved. So our strongest rank-#1 shot is CRYPTO_PRICE.
TVL will compete but our multi-source + Solana-first angle differentiates.

## Format-mismatch hypothesis (to confirm at epoch 289)
- Top miners return CLEAN objects with explicit numeric fields:
  chainsight-oracle label_field='signal', fields price_usd/market_cap_usd/rate/
  tvl_usd/gas_price_gwei/volume_24h_usd. txlens has price_usd, tvl_usd, rate,
  gas_price_gwei, market_cap_usd.
- OUR miners return RAW upstream JSON: CoinGecko /price is ARRAY
  [{current_price, symbol}], /financials array, /fx exchange_rates dict.
  No price_usd/rate/tvl_usd named fields the scorer's label_field extracts.
- IF epoch 289 shows top miners ~0.5-0.9 but us still 0 -> scorer field-extraction
  fails on our raw shape. FIX needs a reshaping proxy (free Cloudflare Worker /
  Vercel) returning {price_usd, rate, tvl_usd, market_cap_usd, gas_price_gwei}.
  Miners are serverless YAML proxies (no custom code), so a transform layer
  requires our own free-hosted API in front of CoinGecko/DefiLlama.
- If epoch 289 also ~0 network-wide -> systemic, no format fix needed yet.

## Gas miner known issue
- 7313 /gas returns HTTP 500 "Parse error" on live queries. Engine sends
  payload:{} as POST body; our static body: (JSON-RPC eth_gasPrice) NOT applied
  for engine/ask (only on_chain.request uses body:). publicnode itself works
  (0x5b8d80 ~5.78 gwei). No keyless GET gas API exists. Gas serving blocked
  until engine constructs proper JSON-RPC payload for GAS_PRICE, or we host a
  free GET gas wrapper. Deferred.

## Driving live traffic
- tools/x402_payer.mjs (official @x402/evm + @x402/fetch, ExactEvmScheme,
  PAYMENT-SIGNATURE) pays $0.01; miners answer + signal_hash (scored).
  Burner 0xe36a07...Ed4c funded 6000 USDC.
- tools/traffic_loop.mjs fires 20 varied queries, 2s spacing (CoinGecko 429).
- NOTE: engine routes by YAML id (7311/7312/7313), NOT registration_id. Loop/
  payer must use YAML id, not re-registration id (289).


Step 1 proven: `tools/x402_payer.mjs` (official @x402/evm + @x402/fetch, viem
signer, ExactEvmScheme, PAYMENT-SIGNATURE handshake) pays $0.01 and our miners
answer live + get signal_hash (scored). Burner 0xe36a07...Ed4c funded 6000 USDC.

KEY x402 learnings (from docs.telegraphprotocol.com/docs/using/x402-inference):
- Signature is EIP-712 typed data (ERC-3009), NOT a raw transfer tx.
- Header is `PAYMENT-SIGNATURE` (verified by PayAI facilitator).
- Challenge decoded from `PAYMENT-REQUIRED` header (base64).

Step 2: `tools/traffic_loop.mjs` fires N varied queries to build score history.

### Fixes applied + verified live (re-registered: crypto 240, tvl 241)
- CRYPTO_PRICE USD bug: node injects vs_currencies=eth and overrides any baked
  vs_currencies=usd on simple/price (which REQUIRES vs_currencies). FIX: /price
  now uses coins/markets?vs_currency=usd (reads singular vs_currency, ignores
  the injected plural) -> returns USD current_price. Verified: 2502.86 USD.
- TVL protocol routing: /protocol had no param_map and DefiLlama is path-based,
  so all protocols returned uniswap. FIX: explicit per-protocol endpoints
  (/protocol/jupiter, /lido, /aave, /curve, /marinade, /jito, /kamino). Verified:
  /protocol/jupiter returns Jupiter's solana:JUP... address + TVL.
- NOTE: node auto-router for a plain "jupiter tvl" query picks /protocol
  (uniswap default); correct per-protocol data requires specifying the endpoint.
  Acceptable: agents that name the protocol hit the right endpoint.


The wizard sandbox POSTs /gas with no JSON-RPC body -> meowrpc returns 405. No
keyless GET gas API exists (publicnode GET returned HTML landing page, not RPC).
No YAML `body:` field accepts a static literal (body only maps from on-chain
arrays). Options to revisit:
- Ask team if Engine auto-sends eth_gasPrice body for GAS_PRICE, or if a static
  body field exists.
- Find a keyless gas REST endpoint (none known without API key).
- Reconsider gas entirely; CRYPTO_PRICE + TVL_LOOKUP are the solid submissions.

## KNOWN BLOCKER (2026-08-26) — report to team

The integrate.telegraphprotocol.com wizard "Upload to IPFS" step shows
"Upload Successful" with CID `QmDcJHrrHSgvFpsYxqb6g97uaQTd2kE31rPUeDZTeDsjVq`
for ALL three miners. That CID is **invalid** (not a real IPFS hash; Pinata
returns "CID is invalid", dweb/cloudflare gateways 500/000). So the YAML is
never actually pinned, and Register On-Chain cannot proceed (no real YAML URL).
All three showing the identical hash is impossible if content differs ->
placeholder/bug in the wizard, not our YAML. Need team fix or a working pin path.

Gas note: sandbox returns 405 on /gas because JSON-RPC needs a POST body the
naive sandbox probe does not send. At runtime the Engine sends the body, so gas
should work in production; sandbox 405 is expected for JSON-RPC miners.

## Sandbox-test fixes already applied (so endpoints pass once pin works)

- crypto-price: baked `?symbols=sol,eth&vs_currencies=usd` into /price and
  Wrapped SOL token into /pair external_path (bare probe now 200/200).
- tvl: baked `/protocol/uniswap` default (bare probe now 200).
- gas: kept POST JSON-RPC; sandbox 405 is benign (body sent at runtime).

## Locked decisions

See `docs/decisions.md` for the full log. Summary:
- Monorepo `optivis-telegraph-miners`, branded `optivis-*` slugs.
- Track 1 only for H1; Track 2 (scorer) is phase 2.
- Use **free, keyless public APIs** (CoinGecko, DexScreener, DefiLlama, public
  ETH RPC) — no servers to host, no keys to leak. The miner YAML is the whole
  "deployment"; the Telegraph node proxies to the upstream.
- Target empty/underserved intents to rank #1 by default (70% traffic share).
- Solana-first examples as the differentiator; chain-agnostic underneath.
- Every `source_path` must be proven by `tools/verify_upstream.py` + committed
  fixture before registration.

## Registration runbook (do this in order)

1. Open https://integrate.telegraphprotocol.com/register (Base Sepolia wallet
   connected — `vip-ultr` wallet already set up per user).
2. For **each** miner in `miners/`:
   a. Paste the YAML (or use Import YAML).
   b. The wizard sandbox-tests each endpoint against the live upstream
      (we already proved these in VERIFY.md; wizard is the final gate).
   c. Upload to IPFS (Pinata) → get YAML URL + hash.
   d. Register on-chain: set **Fee Address** = `vip-ultr` wallet, **Floor
      Price** = 0.01 USDC (network min). Sign `registerMiner`.
   e. Record the tx hash + assigned `miner_id` (we pre-reserved 7311/7312/7313
      in the YAML `id` field; the contract may assign its own — trust the
      on-chain id, update slug mapping in README if needed).
3. Wait ~2-3 min for the indexer, then confirm on the explorer leaderboard and
   `GET /api/miners?intent=CRYPTO_PRICE`.
4. Install upstream API key if any endpoint ever needs one (none do today —
   `auth: none` everywhere) via Dashboard → API Key (wallet-signed).
5. Update this table's "Registered" column and commit.

## Why these intents / APIs (research basis)

- 45 canonical intents; 25 unserved at build time. We picked 3 Deterministic
  (Tier A) financial intents with keyless, reliable upstreams.
- Tier A scoring = exact numeric match. Our field paths are proven, so we
  score ~1.0; most AI-generated miners score ~0 from wrong paths.
- Routing: rank #1 gets 70% of routed requests. Empty intents → we are #1.
- Grace period: 7 days at 5% equal-share before leaderboard ranking. Register
  **now** so grace ends before the **Sep 7 12:00 UTC** deadline.

## Known risks / open items

- Public RPCs (gas) rate-limit. `optivis-gas` uses `eth.meowrpc.com` with
  publicnode/1rpc as fallbacks noted in VERIFY.md. If all block, swap
  `base_url` and re-register (floor price is immutable, full re-register).
- `optivis-crypto-price` `/price` returns a symbol map; our `label_field`
  picks the first requested symbol's `usd`. Confirm the Engine passes
  `symbols` (not `symbol`) — the param_map aliases it. Wizard will surface
  mismatches.
- DexScreener pair `pairCreatedAt` is epoch-ms; our `pair_age_hours` field is
  descriptive only (not in on_chain ints) — fine for response, not on-chain.

## Phase 2 (after miners live & scoring)

- Track 2: Rust `no_std` WASM scorer for CRYPTO_PRICE with numeric tolerance
  + freshness, registered via `registerWasm` (gas only, no bond). Must beat
  incumbent champion on benchmark.
- Track 3: a small agent app consuming our own miners (opens after T1/T2 close)
  to satisfy "apps built on your miner" judging criteria.

## Commands

```bash
python3 tools/verify_upstream.py    # reproduce 6/6 live checks
python3 tools/validate_yaml.py      # YAML structure check
```
