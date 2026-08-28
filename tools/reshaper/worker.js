// Free reshaping proxy for Telegraph miners.
// Deploys as a Cloudflare Worker (or Vercel/Deno serverless). No API key.
// Purpose: reshape raw CoinGecko/DefiLlama JSON into the clean named-field
// shape the incumbent scorer's label_field extraction expects
// (price_usd, rate, tvl_usd, market_cap_usd, gas_price_gwei, volume_24h_usd).
//
// Routes:
//   /price?symbols=sol,eth        -> CoinGecko coins/markets, reshaped to
//                                    {price_usd, market_cap_usd, volume_24h_usd, symbol}
//   /fx?base=usd&target=eur        -> CoinGecko exchange_rates, reshaped to {rate}
//   /financials?symbols=sol,eth    -> coins/markets, reshaped (market_cap_usd etc)
//   /tvl?protocol=uniswap          -> DefiLlama /protocol/{slug}, reshaped to {tvl_usd}
//
// The miner YAML would point endpoint_base_url at this worker and call these
// paths, so the result the scorer sees is the clean object.

const CG = "https://api.coingecko.com/api/v3";
const DL = "https://api.llama.fi";

function jSend(obj) {
  return new Response(JSON.stringify(obj), {
    headers: { "content-type": "application/json", "access-control-allow-origin": "*" },
  });
}

async function price(symbols) {
  const url = `${CG}/coins/markets?vs_currency=usd&symbols=${encodeURIComponent(symbols)}&per_page=50`;
  const r = await fetch(url);
  const arr = await r.json();
  const out = arr.map((c) => ({
    symbol: (c.symbol || "").toUpperCase(),
    price_usd: c.current_price,
    market_cap_usd: c.market_cap,
    volume_24h_usd: c.total_volume,
    change_24h_pct: c.price_change_percentage_24h,
  }));
  return jSend(out);
}

async function fx(base, target) {
  const url = `${CG}/exchange_rates`;
  const r = await fetch(url);
  const d = await r.json();
  const rates = d.rates || {};
  const b = rates[(base || "usd").toLowerCase()];
  const t = rates[(target || "eur").toLowerCase()];
  if (!b || !t) return jSend({ error: "unknown currency" });
  const rate = t.value / b.value; // 1 base = rate target
  return jSend({ base: (base || "usd").toUpperCase(), target: (target || "eur").toUpperCase(), rate });
}

async function financials(symbols) {
  return price(symbols); // same source, clean fields
}

async function tvl(protocol) {
  const slug = (protocol || "uniswap").toLowerCase();
  const r = await fetch(`${DL}/protocol/${slug}`);
  const d = await r.json();
  return jSend({ protocol: slug, tvl_usd: d.currentChainTvls ? d.currentChainTvls["Ethereum"]?.tvl ?? d.tvl : d.tvl, name: d.name });
}

export default {
  async fetch(req) {
    const u = new URL(req.url);
    const p = u.pathname;
    try {
      if (p === "/price") return await price(u.searchParams.get("symbols") || "sol,eth");
      if (p === "/fx") return await fx(u.searchParams.get("base"), u.searchParams.get("target"));
      if (p === "/financials") return await financials(u.searchParams.get("symbols") || "sol,eth");
      if (p === "/tvl") return await tvl(u.searchParams.get("protocol"));
      return jSend({ error: "unknown route", routes: ["/price", "/fx", "/financials", "/tvl"] });
    } catch (e) {
      return jSend({ error: String(e) });
    }
  },
};
