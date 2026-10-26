// Live FX rate and ticker price fetching. Nothing here is cached in the DB —
// every call hits the external API fresh, per the app's live-fetch design.

const FALLBACK_USD_TWD_RATE: f64 = 32.5;

#[derive(serde::Serialize, Clone)]
pub struct FxRate {
    pub pair: String,
    pub rate: f64,
}

#[derive(serde::Serialize, Clone)]
pub struct TickerPrice {
    pub ticker: String,
    pub price: f64,
    pub currency: String,
}

pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default()
}

// Live USD -> TWD exchange rate, fetched fresh on every call (no DB caching).
pub async fn fetch_usd_twd_rate(client: &reqwest::Client) -> f64 {
    match client
        .get("https://open.er-api.com/v6/latest/USD")
        .send()
        .await
    {
        Ok(resp) => resp
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|json| json["rates"]["TWD"].as_f64())
            .unwrap_or_else(|| {
                eprintln!("Live FX fetch returned no TWD rate. Using fallback.");
                FALLBACK_USD_TWD_RATE
            }),
        Err(e) => {
            eprintln!("Live FX fetch failed: {}. Using fallback.", e);
            FALLBACK_USD_TWD_RATE
        }
    }
}

pub fn yahoo_symbol(ticker: &str, currency: &str) -> String {
    if currency == "TWD" && !ticker.contains('.') {
        format!("{}.TW", ticker)
    } else {
        ticker.to_string()
    }
}

// Live closing price for a single ticker; returns None if the fetch fails so
// callers can fall back to cost-basis instead of showing a fabricated price.
pub async fn fetch_ticker_price(
    client: &reqwest::Client,
    ticker: &str,
    currency: &str,
) -> Option<f64> {
    let symbol = yahoo_symbol(ticker, currency);
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}",
        symbol
    );
    match client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        )
        .send()
        .await
    {
        Ok(resp) => resp
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|json| json["chart"]["result"][0]["meta"]["regularMarketPrice"].as_f64()),
        Err(e) => {
            eprintln!("Live price fetch failed for ticker {}: {}", ticker, e);
            None
        }
    }
}

// Fetches a live closing price for every distinct ticker currently held in the
// portfolio. Tickers whose fetch fails are simply omitted from the result.
pub async fn fetch_live_ticker_prices(
    client: &reqwest::Client,
    pool: &sqlx::SqlitePool,
) -> Vec<TickerPrice> {
    let holdings: Vec<(String, String)> =
        sqlx::query_as("SELECT DISTINCT ticker, currency FROM investments")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let mut handles = Vec::with_capacity(holdings.len());
    for (ticker, currency) in holdings {
        let client = client.clone();
        handles.push(tokio::spawn(async move {
            let price = fetch_ticker_price(&client, &ticker, &currency).await;
            price.map(|price| TickerPrice {
                ticker,
                price,
                currency,
            })
        }));
    }

    let mut prices = Vec::new();
    for handle in handles {
        if let Ok(Some(price)) = handle.await {
            prices.push(price);
        }
    }
    prices
}
