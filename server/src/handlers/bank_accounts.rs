use crate::db::{BankAccount, Db};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

pub async fn list_bank_accounts(State(db): State<Db>) -> impl IntoResponse {
    match sqlx::query_as::<_, BankAccount>("SELECT id, name, currency, balance FROM bank_accounts")
        .fetch_all(&db.pool)
        .await
    {
        Ok(accounts) => (StatusCode::OK, Json(accounts)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}

pub async fn create_bank_account(
    State(db): State<Db>,
    Json(mut acc): Json<BankAccount>,
) -> impl IntoResponse {
    if acc.id.is_empty() {
        acc.id = Uuid::new_v4().to_string();
    }
    let res =
        sqlx::query("INSERT INTO bank_accounts (id, name, currency, balance) VALUES (?, ?, ?, ?)")
            .bind(&acc.id)
            .bind(&acc.name)
            .bind(&acc.currency)
            .bind(acc.balance)
            .execute(&db.pool)
            .await;

    match res {
        Ok(_) => (StatusCode::CREATED, Json(acc)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
    }
}

pub async fn update_bank_account(
    State(db): State<Db>,
    Path(id): Path<String>,
    Json(acc): Json<BankAccount>,
) -> impl IntoResponse {
    let res =
        sqlx::query("UPDATE bank_accounts SET name = ?, currency = ?, balance = ? WHERE id = ?")
            .bind(&acc.name)
            .bind(&acc.currency)
            .bind(acc.balance)
            .bind(&id)
            .execute(&db.pool)
            .await;

    match res {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
    }
}

pub async fn delete_bank_account(
    State(db): State<Db>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM bank_accounts WHERE id = ?")
        .bind(&id)
        .execute(&db.pool)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}
