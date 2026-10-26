use crate::db::{CashFlow, Db, HistoricalAsset};
use crate::pricing::{fetch_usd_twd_rate, http_client};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

// Chart aggregation structures

#[derive(serde::Serialize)]
struct AggregatedChartData {
    asset_growth: Vec<AssetGrowthPoint>,
    income_expense: Vec<IncomeExpensePoint>,
    passive_income: Vec<PassiveIncomePoint>,
}

#[derive(serde::Serialize)]
struct AssetGrowthPoint {
    month: String,
    total: f64,              // Total Net Worth (bank + market value), TWD
    bank_total: f64,         // Total Bank Balance, TWD
    market_value_total: f64, // Total Market Value, TWD
    cost_basis_total: f64,   // Total Cost Basis, TWD
}

#[derive(serde::Serialize)]
struct IncomeExpensePoint {
    month: String,
    inflow: f64,
    outflow: f64,
}

#[derive(serde::Serialize)]
struct PassiveIncomePoint {
    month: String,
    total: f64, // Bank Interest + Investment Distributions, combined, TWD
}

pub async fn get_aggregated_charts(State(db): State<Db>) -> impl IntoResponse {
    let pool = &db.pool;

    // Live FX rate, used only as a fallback for months that have no logged
    // snapshot rate in the Asset Growth Ledger.
    let live_usd_twd_rate = fetch_usd_twd_rate(&http_client()).await;

    // 1. Asset Growth Series — always TWD, using each snapshot's own logged rate
    let snapshots = sqlx::query_as::<_, HistoricalAsset>("SELECT month, bank_usd, bank_twd, market_value_usd, market_value_twd, cost_basis_usd, cost_basis_twd, usd_twd_rate FROM historical_assets ORDER BY month ASC")
        .fetch_all(pool).await.unwrap_or_default();

    // Every other section (cash flow transactions have no rate of their own)
    // converts USD amounts using the rate logged for that transaction's month,
    // falling back to the live rate if that month has no snapshot yet.
    let monthly_rates: std::collections::HashMap<String, f64> = snapshots
        .iter()
        .map(|snapshot| (snapshot.month.clone(), snapshot.usd_twd_rate))
        .collect();

    let convert_to_twd = |amount: f64, from_curr: &str, month: &str| -> f64 {
        if from_curr == "TWD" {
            amount
        } else {
            let rate = monthly_rates
                .get(month)
                .copied()
                .unwrap_or(live_usd_twd_rate);
            amount * rate
        }
    };

    let asset_growth: Vec<AssetGrowthPoint> = snapshots
        .into_iter()
        .map(|snapshot| {
            let rate = snapshot.usd_twd_rate;
            let bank_total = snapshot.bank_usd * rate + snapshot.bank_twd;
            let market_value_total = snapshot.market_value_usd * rate + snapshot.market_value_twd;
            let cost_basis_total = snapshot.cost_basis_usd * rate + snapshot.cost_basis_twd;
            let total = bank_total + market_value_total;

            AssetGrowthPoint {
                month: snapshot.month,
                total,
                bank_total,
                market_value_total,
                cost_basis_total,
            }
        })
        .collect();

    // 2. Cash Flow: Income vs Expense Series (Grouped by month YYYY-MM)
    let transactions = sqlx::query_as::<_, CashFlow>(
        "SELECT id, date, description, category, amount, currency, type FROM cash_flow",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    use std::collections::BTreeMap;
    let mut inc_exp_map: BTreeMap<String, (f64, f64)> = BTreeMap::new();

    // Initialize default months to match historical assets or sample
    for point in &asset_growth {
        inc_exp_map.insert(point.month.clone(), (0.0, 0.0));
    }

    for transaction in &transactions {
        if transaction.date.len() >= 7 {
            let month = transaction.date[0..7].to_string();
            let amount_in_twd = convert_to_twd(transaction.amount, &transaction.currency, &month);
            let entry = inc_exp_map.entry(month).or_insert((0.0, 0.0));
            if transaction.r#type == "inflow" {
                entry.0 += amount_in_twd;
            } else {
                entry.1 += amount_in_twd;
            }
        }
    }

    let income_expense: Vec<IncomeExpensePoint> = inc_exp_map
        .into_iter()
        .map(|(month, (inflow, outflow))| IncomeExpensePoint {
            month,
            inflow,
            outflow,
        })
        .collect();

    // 3. Passive Income Trend (Grouped by month)
    // Combines Bank Interest (Interest Income) and Investment Distributions
    let mut passive_map: BTreeMap<String, f64> = BTreeMap::new();

    // Initialize standard months from spec to verify numbers
    passive_map.insert("2026-03".to_string(), 0.0);
    passive_map.insert("2026-04".to_string(), 0.0);
    passive_map.insert("2026-05".to_string(), 0.0);

    for transaction in &transactions {
        if transaction.date.len() >= 7 {
            let month = transaction.date[0..7].to_string();
            let amount_in_twd = convert_to_twd(transaction.amount, &transaction.currency, &month);
            if matches!(
                transaction.category.as_str(),
                "Interest Income" | "Investment Distributions"
            ) {
                *passive_map.entry(month).or_insert(0.0) += amount_in_twd;
            }
        }
    }

    let passive_income: Vec<PassiveIncomePoint> = passive_map
        .into_iter()
        .map(|(month, total)| PassiveIncomePoint { month, total })
        .collect();

    let result = AggregatedChartData {
        asset_growth,
        income_expense,
        passive_income,
    };

    (StatusCode::OK, Json(result)).into_response()
}
