# Optivis Telegraph Miners

A suite of **verifiable financial-data miners** for the [Telegraph Protocol](https://telegraphprotocol.com)
autonomous-inference network, built for the **Telegraph Hackathon (Season I, H1)**
by [Optivis Labs](https://optivislabs.vercel.app).

Three miners, one branded suite, all wrapped from **free, keyless, public APIs**
and proven field-by-field against live data:

| Miner | Slug | Intent | Upstream (keyless) |
|---|---|---|---|
| Optivis Crypto Price | `optivis-crypto-price` | `CRYPTO_PRICE` | CoinGecko + DexScreener |
| Optivis TVL Oracle | `optivis-tvl` | `TVL_LOOKUP` | DefiLlama |
| Optivis Gas Oracle | `optivis-gas` | `GAS_PRICE` | Ethereum JSON-RPC (public) |

## Why this wins (not just another YAML)

A Telegraph miner is a declarative YAML that points the network at an upstream
API — anyone can paste one. What separates a winning miner from the ~0-scoring
mass is **integration discipline**, which this repo makes auditable:

1. **Every field path is proven, not guessed.** `tools/verify_upstream.py`
   hits each live API, asserts the exact `source_path` our YAML maps, and saves
   the raw response into `miners/<slug>/fixtures/` as evidence. When the
   Tier A (Deterministic) scorer exact-matches the answer, our response
   *contains the number in the mapped field* — competitors who guessed
   `price` instead of `sol.usd` score ~0.
2. **Multi-source answers.** `optivis-crypto-price` returns price **and** best
   DEX pair liquidity/volume/FDV in one call. Agents prefer the miner that
   answers the whole question → more requests served.
3. **Solana-first.** Pair lookups default to Solana-native tokens, our
   signature for the Solana + AI audience, while staying chain-agnostic.
4. **Reliability wired in.** `cache_ttl_sec`, rate limits, and circuit
   breakers are set per endpoint so a transient upstream failure can't trigger
   the >20% spot-check revocation.

## Repository layout

```
optivis-telegraph-miners/
├── miners/
│   ├── optivis-crypto-price/   miner.yaml · VERIFY.md · fixtures/
│   ├── optivis-tvl/            miner.yaml · VERIFY.md · fixtures/
│   └── optivis-gas/            miner.yaml · VERIFY.md · fixtures/
├── tools/
│   ├── verify_upstream.py      # prove every field path vs live APIs
│   └── validate_yaml.py        # static check vs Telegraph closed-set rules
├── docs/
│   ├── research/               # canonical intents, scoring mechanics
│   ├── decisions.md            # locked decisions log
│   └── post-mortem-zero-score-miners.md
├── BUILD.md                    # handoff / resume doc
└── README.md
```

## Local verification

```bash
python3 tools/verify_upstream.py   # 6/6 live upstream checks
python3 tools/validate_yaml.py     # YAML structure check
```

## Registration

Miners are registered through the official wizard at
[integrate.telegraphprotocol.com](https://integrate.telegraphprotocol.com)
on **Base Sepolia** (no bond, gas only). The wizard sandbox-tests each
endpoint against the live upstream before pinning the YAML to IPFS and
submitting `registerMiner(...)`. See `BUILD.md` for the step-by-step runbook
and current status of each miner.

## Track fit

- **Track 1 (Miner):** all three miners target empty / underserved intents
  (CRYPTO_PRICE, TVL_LOOKUP, GAS_PRICE had no dedicated, well-wired competitor
  at build time), so each is `#1` by default and captures up to 70% of routed
  traffic for its intent.
- **Track 2 (Script Author):** a deterministic numeric-tolerance scorer for
  CRYPTO_PRICE is the planned phase-2 extension (see BUILD.md).

---
Built by Optivis Labs · X: [@vip_ultr](https://x.com/vip_ultr)
