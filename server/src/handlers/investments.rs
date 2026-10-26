use crate::db::{Db, Investment};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

pub async fn list_investments(State(db): State<Db>) -> impl IntoResponse {
    match sqlx::query_as::<_, Investment>(
        "SELECT id, ticker, name, type, shares, cost_basis, currency FROM investments",
    )
    .fetch_all(&db.pool)
    .await
    {
        Ok(investments) => (StatusCode::OK, Json(investments)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}

pub async fn create_investment(
    State(db): State<Db>,
    Json(mut inv): Json<Investment>,
) -> impl IntoResponse {
    if inv.id.is_empty() {
        inv.id = Uuid::new_v4().to_string();
    }
    let res = sqlx::query("INSERT INTO investments (id, ticker, name, type, shares, cost_basis, currency) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&inv.id)
        .bind(&inv.ticker)
        .bind(&inv.name)
        .bind(&inv.r#type)
        .bind(inv.shares)
        .bind(inv.cost_basis)
        .bind(&inv.currency)
        .execute(&db.pool).await;

    match res {
        Ok(_) => (StatusCode::CREATED, Json(inv)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
    }
}

pub async fn update_investment(
    State(db): State<Db>,
    Path(id): Path<String>,
    Json(inv): Json<Investment>,
) -> impl IntoResponse {
    let res = sqlx::query("UPDATE investments SET ticker = ?, name = ?, type = ?, shares = ?, cost_basis = ?, currency = ? WHERE id = ?")
        .bind(&inv.ticker)
        .bind(&inv.name)
        .bind(&inv.r#type)
        .bind(inv.shares)
        .bind(inv.cost_basis)
        .bind(&inv.currency)
        .bind(&id)
        .execute(&db.pool).await;

    match res {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
    }
}

pub async fn delete_investment(State(db): State<Db>, Path(id): Path<String>) -> impl IntoResponse {
    match sqlx::query("DELETE FROM investments WHERE id = ?")
        .bind(&id)
        .execute(&db.pool)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND.into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}
