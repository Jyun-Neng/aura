use crate::db::{Db, RecurringTransaction};
use crate::recurring::day_of_month;
use crate::validate::is_date;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

pub async fn list_recurring_transactions(State(db): State<Db>) -> impl IntoResponse {
    match sqlx::query_as::<_, RecurringTransaction>(
        "SELECT id, description, category, amount, currency, type, next_run_date FROM recurring_transactions ORDER BY next_run_date ASC",
    )
    .fetch_all(&db.pool)
    .await
    {
        Ok(rows) => (StatusCode::OK, Json(rows)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}

pub async fn create_recurring_transaction(
    State(db): State<Db>,
    Json(mut recurring_transaction): Json<RecurringTransaction>,
) -> impl IntoResponse {
    if !is_date(&recurring_transaction.next_run_date) {
        return (
            StatusCode::BAD_REQUEST,
            Json(format!(
                "next_run_date must be YYYY-MM-DD, got {:?}",
                recurring_transaction.next_run_date
            )),
        )
            .into_response();
    }
    if recurring_transaction.id.is_empty() {
        recurring_transaction.id = Uuid::new_v4().to_string();
    }
    let res = sqlx::query(
        "INSERT INTO recurring_transactions (id, description, category, amount, currency, type, next_run_date, anchor_day) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&recurring_transaction.id)
    .bind(&recurring_transaction.description)
    .bind(&recurring_transaction.category)
    .bind(recurring_transaction.amount)
    .bind(&recurring_transaction.currency)
    .bind(&recurring_transaction.r#type)
    .bind(&recurring_transaction.next_run_date)
    .bind(day_of_month(&recurring_transaction.next_run_date))
    .execute(&db.pool)
    .await;

    match res {
        Ok(_) => (StatusCode::CREATED, Json(recurring_transaction)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
    }
}

pub async fn update_recurring_transaction(
    State(db): State<Db>,
    Path(id): Path<String>,
    Json(recurring_transaction): Json<RecurringTransaction>,
) -> impl IntoResponse {
    if !is_date(&recurring_transaction.next_run_date) {
        return (
            StatusCode::BAD_REQUEST,
            Json(format!(
                "next_run_date must be YYYY-MM-DD, got {:?}",
                recurring_transaction.next_run_date
            )),
        )
            .into_response();
    }
    let res = sqlx::query(
        // The anchor day follows next_run_date only when it changes, so editing
        // other fields of a rule on the 30th (now clamped to the 28th) keeps the 30th.
        "UPDATE recurring_transactions SET description = ?, category = ?, amount = ?, currency = ?, type = ?,
            anchor_day = CASE WHEN next_run_date = ? THEN anchor_day ELSE ? END,
            next_run_date = ? WHERE id = ?",
    )
    .bind(&recurring_transaction.description)
    .bind(&recurring_transaction.category)
    .bind(recurring_transaction.amount)
    .bind(&recurring_transaction.currency)
    .bind(&recurring_transaction.r#type)
    .bind(&recurring_transaction.next_run_date)
    .bind(day_of_month(&recurring_transaction.next_run_date))
    .bind(&recurring_transaction.next_run_date)
    .bind(&id)
    .execute(&db.pool)
    .await;

    match res {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
    }
}

pub async fn delete_recurring_transaction(
    State(db): State<Db>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM recurring_transactions WHERE id = ?")
        .bind(&id)
        .execute(&db.pool)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}
