use crate::db::Db;
use crate::pricing::{
    FxRate, TickerPrice, fetch_live_ticker_prices, fetch_usd_twd_rate, http_client,
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

// Valuation Data — fetched live on every request, nothing is cached in the DB.
#[derive(serde::Serialize)]
struct ValuationResponse {
    rates: Vec<FxRate>,
    prices: Vec<TickerPrice>,
}

pub async fn get_valuation(State(db): State<Db>) -> impl IntoResponse {
    let client = http_client();
    let rate = fetch_usd_twd_rate(&client).await;
    let prices = fetch_live_ticker_prices(&client, &db.pool).await;

    let rates = vec![FxRate {
        pair: "USD_TWD".to_string(),
        rate,
    }];

    (StatusCode::OK, Json(ValuationResponse { rates, prices })).into_response()
}
