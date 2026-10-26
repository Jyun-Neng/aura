use crate::db::{Db, HistoricalAsset};
use crate::validate::is_month;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

pub async fn list_historical_assets(State(db): State<Db>) -> impl IntoResponse {
    match sqlx::query_as::<_, HistoricalAsset>("SELECT month, bank_usd, bank_twd, market_value_usd, market_value_twd, cost_basis_usd, cost_basis_twd, usd_twd_rate FROM historical_assets ORDER BY month DESC")
        .fetch_all(&db.pool).await
    {
        Ok(assets) => (StatusCode::OK, Json(assets)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}

pub async fn save_historical_asset(
    State(db): State<Db>,
    Json(asset): Json<HistoricalAsset>,
) -> impl IntoResponse {
    if !is_month(&asset.month) {
        return (
            StatusCode::BAD_REQUEST,
            Json(format!("month must be YYYY-MM, got {:?}", asset.month)),
        )
            .into_response();
    }
    let res = sqlx::query(
        "INSERT INTO historical_assets (month, bank_usd, bank_twd, market_value_usd, market_value_twd, cost_basis_usd, cost_basis_twd, usd_twd_rate)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(month) DO UPDATE SET
            bank_usd = excluded.bank_usd,
            bank_twd = excluded.bank_twd,
            market_value_usd = excluded.market_value_usd,
            market_value_twd = excluded.market_value_twd,
            cost_basis_usd = excluded.cost_basis_usd,
            cost_basis_twd = excluded.cost_basis_twd,
            usd_twd_rate = excluded.usd_twd_rate",
    )
    .bind(&asset.month)
    .bind(asset.bank_usd)
    .bind(asset.bank_twd)
    .bind(asset.market_value_usd)
    .bind(asset.market_value_twd)
    .bind(asset.cost_basis_usd)
    .bind(asset.cost_basis_twd)
    .bind(asset.usd_twd_rate)
    .execute(&db.pool)
    .await;

    match res {
        Ok(_) => (StatusCode::OK, Json(asset)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
    }
}

pub async fn delete_historical_asset(
    State(db): State<Db>,
    Path(month): Path<String>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM historical_assets WHERE month = ?")
        .bind(&month)
        .execute(&db.pool)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}
