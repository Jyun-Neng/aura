use crate::db::{CashFlow, Db};
use crate::recurring::process_due_recurring_transactions;
use crate::validate::is_date;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

pub async fn list_cash_flows(State(db): State<Db>) -> impl IntoResponse {
    // Catch up any due recurring transactions before returning the list, so
    // the registry always reflects auto-logged entries as soon as they're due.
    if let Err(e) = process_due_recurring_transactions(&db.pool).await {
        eprintln!("Failed to process recurring transactions: {}", e);
    }

    match sqlx::query_as::<_, CashFlow>("SELECT id, date, description, category, amount, currency, type FROM cash_flow ORDER BY date DESC")
        .fetch_all(&db.pool).await
    {
        Ok(flows) => (StatusCode::OK, Json(flows)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}

pub async fn create_cash_flow(
    State(db): State<Db>,
    Json(mut flow): Json<CashFlow>,
) -> impl IntoResponse {
    if !is_date(&flow.date) {
        return (
            StatusCode::BAD_REQUEST,
            Json(format!("date must be YYYY-MM-DD, got {:?}", flow.date)),
        )
            .into_response();
    }
    if flow.id.is_empty() {
        flow.id = Uuid::new_v4().to_string();
    }
    let res = sqlx::query("INSERT INTO cash_flow (id, date, description, category, amount, currency, type) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&flow.id)
        .bind(&flow.date)
        .bind(&flow.description)
        .bind(&flow.category)
        .bind(flow.amount)
        .bind(&flow.currency)
        .bind(&flow.r#type)
        .execute(&db.pool).await;

    match res {
        Ok(_) => (StatusCode::CREATED, Json(flow)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
    }
}

pub async fn update_cash_flow(
    State(db): State<Db>,
    Path(id): Path<String>,
    Json(flow): Json<CashFlow>,
) -> impl IntoResponse {
    if !is_date(&flow.date) {
        return (
            StatusCode::BAD_REQUEST,
            Json(format!("date must be YYYY-MM-DD, got {:?}", flow.date)),
        )
            .into_response();
    }
    let res = sqlx::query("UPDATE cash_flow SET date = ?, description = ?, category = ?, amount = ?, currency = ?, type = ? WHERE id = ?")
        .bind(&flow.date)
        .bind(&flow.description)
        .bind(&flow.category)
        .bind(flow.amount)
        .bind(&flow.currency)
        .bind(&flow.r#type)
        .bind(&id)
        .execute(&db.pool).await;

    match res {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
    }
}

pub async fn delete_cash_flow(State(db): State<Db>, Path(id): Path<String>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM cash_flow WHERE id = ?")
        .bind(&id)
        .execute(&db.pool)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}
