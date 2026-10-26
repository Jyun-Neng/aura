use axum::{
    Router,
    http::Method,
    routing::{delete, get, put},
};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};

mod config;
mod db;
mod handlers;
mod pricing;
mod recurring;
mod validate;

use db::Database;
use handlers::*;
use recurring::process_due_recurring_transactions;

#[tokio::main]
async fn main() {
    let db_path = match config::database_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    println!("Connecting to database: {}", db_path.display());

    let db = match Database::init(&db_path).await {
        Ok(d) => Arc::new(d),
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Catch up on any recurring transactions due since the server was last running
    if let Err(e) = process_due_recurring_transactions(&db.pool).await {
        eprintln!("Failed to process recurring transactions on startup: {}", e);
    }

    // Build the Axum router
    let app = Router::new()
        // Bank Accounts CRUD
        .route(
            "/api/bank-accounts",
            get(list_bank_accounts).post(create_bank_account),
        )
        .route(
            "/api/bank-accounts/:id",
            put(update_bank_account).delete(delete_bank_account),
        )
        // Investments CRUD
        .route(
            "/api/investments",
            get(list_investments).post(create_investment),
        )
        .route(
            "/api/investments/:id",
            put(update_investment).delete(delete_investment),
        )
        // Cash Flow CRUD
        .route(
            "/api/cash-flow",
            get(list_cash_flows).post(create_cash_flow),
        )
        .route(
            "/api/cash-flow/:id",
            put(update_cash_flow).delete(delete_cash_flow),
        )
        // Recurring Transactions CRUD
        .route(
            "/api/recurring-transactions",
            get(list_recurring_transactions).post(create_recurring_transaction),
        )
        .route(
            "/api/recurring-transactions/:id",
            put(update_recurring_transaction).delete(delete_recurring_transaction),
        )
        // Historical Assets CRUD
        .route(
            "/api/historical-assets",
            get(list_historical_assets).post(save_historical_asset),
        )
        .route(
            "/api/historical-assets/:month",
            delete(delete_historical_asset),
        )
        // Valuation Engine
        .route("/api/valuation", get(get_valuation))
        // Aggregator Charting Pipeline
        .route("/api/charts/aggregated", get(get_aggregated_charts))
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(|origin, _| {
                    origin.to_str().is_ok_and(is_local_origin)
                }))
                .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(tower_http::cors::Any),
        )
        .with_state(db);

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:3000").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "Can't listen on 127.0.0.1:3000: {}. Is another AURA server already running?",
                e
            );
            std::process::exit(1);
        }
    };
    println!("Server running on http://127.0.0.1:3000");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

// Browsers may call the API only from pages served on this machine (the
// Angular dev server, on whatever port it picked). Any other website is
// refused, so a page open in the browser can't read or change the data.
// Non-browser clients such as curl don't send or check CORS headers.
fn is_local_origin(origin: &str) -> bool {
    let Some(rest) = origin.strip_prefix("http://") else {
        return false;
    };
    let (host, port) = rest.split_once(':').unwrap_or((rest, ""));
    matches!(host, "localhost" | "127.0.0.1") && port.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::is_local_origin;

    #[test]
    fn local_origins_only() {
        assert!(is_local_origin("http://localhost:4200"));
        assert!(is_local_origin("http://localhost:4201"));
        assert!(is_local_origin("http://127.0.0.1:4200"));
        assert!(is_local_origin("http://localhost"));
        assert!(!is_local_origin("https://example.com"));
        assert!(!is_local_origin("http://localhost.evil.com"));
        assert!(!is_local_origin("http://localhost:4200.evil.com"));
        assert!(!is_local_origin("http://127.0.0.1.evil.com:80"));
        assert!(!is_local_origin("null"));
    }
}
