# Research Notes — Telegraph Hackathon

Compiled 2026-08-26 from live node APIs, whitepaper, and official docs
(`telegraph-docs` repo, `tg-miner-integration`, `telegraph-examples`).

## Canonical intents (45 total, 25 unserved at build time)

Served (have miners): AI_TEXT_DETECTION, CHAT_COMPLETION, CONTENT_EXTRACTION,
CONTENT_MODERATION, CONTENT_VERIFICATION, CURRENCY_EXCHANGE, DEEPFAKE_DETECTION,
FRAUD_DETECTION, IMAGE_VERIFICATION, LANGUAGE_GENERATION, LANGUAGE_TRANSLATION,
MEDIA_AUTHENTICITY_CHECK, ONCHAIN_TX_LOOKUP, RESEARCH_QUERY, SENTIMENT_ANALYSIS,
SSL_VERIFICATION, STOCK_PRICE, TASK_COMPLETION, TEXT_CLASSIFICATION,
TEXT_GENERATION, TOKEN_HOLDER_COUNT, URL_SCAN, VIDEO_VERIFICATION,
WALLET_BALANCE_CHECK, WEATHER_CHECK.

**Unserved (wide open):** STORM_ALERT, WEATHER_FORECAST, GAME_RESULT,
AGENT_TASK, WEB_SEARCH, TWITTER_SEARCH, NEWS_SEARCH, RESEARCH_SYNTHESIS,
FACT_CHECK, TEXT_AUTHENTICITY_CHECK, GAS_PRICE, CRYPTO_PRICE, FINANCIAL_DATA,
ACADEMIC_SEARCH, TVL_LOOKUP, IP_GEOLOCATION, NEWS_HEADLINES, CVE_LOOKUP,
TELEGRAPH_KNOWLEDGE, SPORTS_SCORE.

We chose CRYPTO_PRICE, TVL_LOOKUP, GAS_PRICE (all 3 were unserved).

## Scoring tiers

- **Tier A — Deterministic:** WASM exact match. One right answer. Used for
  CRYPTO_PRICE, TVL_LOOKUP, GAS_PRICE, STOCK_PRICE, etc.
- **Tier B — LLM-Judge:** open-ended; LLM context + WASM scores quality. Used
  for CHAT_COMPLETION, WEB_SEARCH, etc.

## Why most miners score ~0

Tier A exact-match needs the answer in a clean, mapped field. Miners with
broken upstreams, wrong `signal_mapping` paths, or 200-status error bodies
return a blob the scorer can't parse → ~0. *Our edge: proven field paths.*

## Routing & economics

- Routing weighted 70% (#1) / 20% (#2) / 10% (#3) by leaderboard score.
- New miner: 7-day grace period, 5% equal-share, no leaderboard score.
- After grace: ranked; earnings = floor_price × demand_multiplier × 0.98
  (settled to MACHINA via TWAP). Floor = 0.01 USDC (immutable).
- Spot checks ~every 20s; >20% score drop → routing revocation until next epoch.

## Miner registration

- No bond/stake. Gas only (Base Sepolia). Wizard at integrate.telegraphprotocol.com.
- Contract `registerMiner(yamlUrl, yamlHash, feeAddress, minPriceUsdc,
  supportedIntents)`; Diamond `0xac683bFa8F1C892E23e8300d14c20678C6FC0CA3`.
- No update fn — edit = deregister + re-register.

## Scoring module (Track 2, phase 2)

- Rust `no_std` WASM, exports `alloc`/`dealloc`/`rank_answer` (q/gt/ma ptrs).
- Must beat incumbent champion on benchmark (worst_self_match ≥ 0.75, separate
  good-from-bad at least as well). Free to register (gas only).
- Test locally with wazero before registering.
