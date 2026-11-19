# ANT/USDC relay

Prepared 2026-11-19 from the deployed relay revision `82a1d4c` in an isolated
checkout. The original checkout's pending SOL failover/monitoring work is intact.

Run a separate ANT relay using `config.ant.example.toml` as the complete
`RATE_RELAY_CONFIG_TOML` environment value. Set `JUPITER_API_KEY` and
`PRICE_RPC_URL` through the deployment platform. The latter must be a Solana
mainnet RPC supporting `getBlockTime`. Never commit credentials.

`GET /api/v1/price/ANT/USDC` returns `usdPrice(ANT) / usdPrice(USDC)` and the
older of the two observation timestamps. Each timestamp comes from Jupiter's
`blockId`, resolved with `getBlockTime`. Re-fetching a cached price cannot renew
its age. Missing or invalid prices, HTTP failures and missing block times do not
produce a fresh sample. The endpoint returns 503 once the last observation is
older than the configured 30-second freshness limit.

The source is the actual Solana token, not Nasdaq's last trade. `stockData.price`
and `createdAt` are not observation timestamps for the token price and are unused.
The configured ANT mint is `ANTBTZQ1EUuTxUs3tfeampxdqTWPHmwyJmcWj4xoAyNe`.
Both ANT and USDC have six decimals. No independent fallback is configured.
The maker currently requires ANT's current and scheduled scaled-UI multipliers
to remain 1; non-unit scaling needs a price-unit conversion before operation.

The 2.5-second poll fits Jupiter's keyless testing limit; production should use
the existing API key and an RPC with sufficient request capacity. A separate
service avoids changing the currently deployed SOL feed during ANT setup.

References: [Jupiter Price API](https://developers.jup.ag/docs/price),
[Backpack directory](https://backpack.exchange/tokenized-stocks/assets).
