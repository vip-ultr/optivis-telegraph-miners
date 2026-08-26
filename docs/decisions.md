# Decisions Log — Optivis Telegraph Miners

Immutable record of locked decisions. Each entry: date, decision, rationale.

## 2026-08-26

**D1. Repo structure: monorepo `optivis-telegraph-miners`.**
Rationale: 3 miners share tooling (verify/validate), one coherent hackathon
story, one README = stronger submission than 3 scattered repos.

**D2. Branding: `optivis-*` slugs (`optivis-crypto-price`, `optivis-tvl`,
`optivis-gas`).**
Rationale: recognizable suite on leaderboard; builds Optivis Labs identity;
helps "apps built on your miner" + branding judging.

**D3. Track 1 (Miner) for H1; Track 2 (Script) is phase 2.**
Rationale: miners are the supply layer and the fastest path to rank #1 on
empty intents. Scorer extends later.

**D4. Use free, keyless public APIs only (CoinGecko, DexScreener, DefiLlama,
public ETH RPC).**
Rationale: $0 cost, no rate-limit pain, no secrets in YAML. The miner is a
declarative YAML; the Telegraph node proxies to the upstream — no server to
deploy, no bond to post.

**D5. Target empty / underserved Deterministic (Tier A) intents: CRYPTO_PRICE,
TVL_LOOKUP, GAS_PRICE.**
Rationale: Tier A = exact numeric match → proven field paths score ~1.0.
Empty intents → default rank #1 → up to 70% of routed traffic.

**D6. Every `source_path` must be proven by `tools/verify_upstream.py` against
the live API and the raw response committed as a fixture before registration.**
Rationale: this is the single biggest differentiator vs AI-generated miners
that guess paths and score ~0. Makes quality auditable.

**D7. Solana-first examples (default pair lookups to Solana tokens).**
Rationale: differentiator aligned with the founder's Solana + AI audience;
chain-agnostic underneath. Keeps X identity consistent.

**D8. Pre-reserve miner IDs 7311/7312/7313 (clear range on testnet).**
Rationale: stable references in YAML + docs; contract may assign its own id at
registration — trust on-chain id, keep slug as identity.

**D9. Register ASAP (before 7-day grace period eats into the run-up to the
Sep 7 12:00 UTC deadline).**
Rationale: grace period = 5% equal-share, no leaderboard score; real ranking
+ 70/20/10 distribution only after. Need ranking history before judging.

**D10. No update function on Telegraph — edits = deregister + re-register
(floor price immutable per registration).**
Rationale: design YAMLs carefully up front; changing `base_url`/intents/
floor = full re-register. Documented in BUILD.md runbook.
