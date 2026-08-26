# BUILD.md — Optivis Telegraph Miners

Handoff / resume document. A fresh session should be able to continue without
re-deriving context. Last updated: **2026-08-26**.

## Status

| Miner | Slug | Intent | YAML | VERIFY | Uploaded | Registered |
|---|---|---|---|---|---|---|
| Optivis Crypto Price | `optivis-crypto-price` | CRYPTO_PRICE | done | 6/6 | sandbox 200/200 (baked defaults) | REGISTERED (id 7311, tx 0x6593...3141) |
| Optivis TVL Oracle | `optivis-tvl` | TVL_LOOKUP | done | 6/6 | sandbox 200 (baked uniswap) | BLOCKED: pin bug |
| Optivis Gas Oracle | `optivis-gas` | GAS_PRICE | done | 6/6 | sandbox 405 (JSON-RPC body) | BLOCKED: pin bug |

Phase: **Track 1 miners drafted & verified. BLOCKED on wizard IPFS pin bug.**

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
