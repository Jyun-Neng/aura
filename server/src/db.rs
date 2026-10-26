use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::path::Path;
use std::sync::Arc;

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone, Debug)]
pub struct BankAccount {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub currency: String,
    pub balance: f64,
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone, Debug)]
pub struct Investment {
    #[serde(default)]
    pub id: String,
    pub ticker: String,
    pub name: String,
    pub r#type: String, // 'stock', 'etf', or 'fund'
    pub shares: f64,
    pub cost_basis: f64,
    pub currency: String, // 'USD' or 'TWD' - also determines the trading market
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone, Debug)]
pub struct CashFlow {
    #[serde(default)]
    pub id: String,
    pub date: String,
    pub description: String,
    pub category: String,
    pub amount: f64,
    pub currency: String,
    pub r#type: String, // 'inflow' or 'outflow'
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone, Debug)]
pub struct RecurringTransaction {
    #[serde(default)]
    pub id: String,
    pub description: String,
    pub category: String,
    pub amount: f64,
    pub currency: String,
    pub r#type: String,        // 'inflow' or 'outflow'
    pub next_run_date: String, // YYYY-MM-DD, next date this will auto-log
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone, Debug)]
pub struct HistoricalAsset {
    pub month: String,
    pub bank_usd: f64,
    pub bank_twd: f64,
    pub market_value_usd: f64,
    pub market_value_twd: f64,
    #[serde(default)]
    pub cost_basis_usd: f64,
    #[serde(default)]
    pub cost_basis_twd: f64,
    #[serde(default = "default_usd_twd_rate")]
    pub usd_twd_rate: f64,
}

fn default_usd_twd_rate() -> f64 {
    32.5
}

pub struct Database {
    pub pool: SqlitePool,
}

pub type Db = Arc<Database>;

impl Database {
    pub async fn init(path: &Path) -> Result<Self, sqlx::Error> {
        // Never create the file here: a missing path (e.g. Google Drive not mounted)
        // must fail instead of silently starting a fresh, empty database.
        // DELETE journal mode keeps everything in one file, so a synced folder
        // never uploads the main file without its WAL.
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(false)
            .journal_mode(SqliteJournalMode::Delete);
        let pool = SqlitePool::connect_with(options).await?;
        create_schema(&pool).await?;
        Ok(Self { pool })
    }
}

/// Creates any missing tables and columns (no-op on an up-to-date database).
pub async fn create_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS bank_accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                currency TEXT NOT NULL CHECK (currency IN ('USD', 'TWD')),
                balance REAL NOT NULL DEFAULT 0.0
            );",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS investments (
                id TEXT PRIMARY KEY,
                ticker TEXT NOT NULL,
                name TEXT NOT NULL,
                type TEXT NOT NULL CHECK (type IN ('stock', 'etf', 'fund')),
                shares REAL NOT NULL DEFAULT 0.0,
                cost_basis REAL NOT NULL DEFAULT 0.0,
                currency TEXT NOT NULL CHECK (currency IN ('USD', 'TWD'))
            );",
    )
    .execute(pool)
    .await?;

    sqlx::query(
            "CREATE TABLE IF NOT EXISTS cash_flow (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL CHECK (category IN (
                    'Salary', 'Bonus', 'Interest Income', 'Investment Distributions', 'Cashback Rewards', 'Other Income',
                    'Loan Amortizations', 'Fixed Deductions', 'Food & Dining', 'Transportation', 'Entertainment',
                    'Baby Supplies', 'Pet Supplies', 'Daily Living', 'Subscriptions', 'Insurance', 'Utilities', 'Gifts', 'Taxation', 'Other Expense'
                )),
                amount REAL NOT NULL,
                currency TEXT NOT NULL CHECK (currency IN ('USD', 'TWD')),
                type TEXT NOT NULL CHECK (type IN ('inflow', 'outflow'))
            );",
        )
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS historical_assets (
                month TEXT PRIMARY KEY,
                bank_usd REAL NOT NULL DEFAULT 0.0,
                bank_twd REAL NOT NULL DEFAULT 0.0,
                market_value_usd REAL NOT NULL DEFAULT 0.0,
                market_value_twd REAL NOT NULL DEFAULT 0.0,
                cost_basis_usd REAL NOT NULL DEFAULT 0.0,
                cost_basis_twd REAL NOT NULL DEFAULT 0.0,
                usd_twd_rate REAL NOT NULL DEFAULT 32.5
            );",
    )
    .execute(pool)
    .await?;

    sqlx::query(
            "CREATE TABLE IF NOT EXISTS recurring_transactions (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                category TEXT NOT NULL CHECK (category IN (
                    'Salary', 'Bonus', 'Interest Income', 'Investment Distributions', 'Cashback Rewards', 'Other Income',
                    'Loan Amortizations', 'Fixed Deductions', 'Food & Dining', 'Transportation', 'Entertainment',
                    'Baby Supplies', 'Pet Supplies', 'Daily Living', 'Subscriptions', 'Insurance', 'Utilities', 'Gifts', 'Taxation', 'Other Expense'
                )),
                amount REAL NOT NULL,
                currency TEXT NOT NULL CHECK (currency IN ('USD', 'TWD')),
                type TEXT NOT NULL CHECK (type IN ('inflow', 'outflow')),
                next_run_date TEXT NOT NULL,
                anchor_day INTEGER
            );",
        )
        .execute(pool)
        .await?;

    // anchor_day: the day of month a recurring transaction repeats on, so
    // a rule on the 30th returns to the 30th after a short month.
    let has_anchor_day: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('recurring_transactions') WHERE name = 'anchor_day'",
        )
        .fetch_one(pool)
        .await?;
    if !has_anchor_day {
        sqlx::query("ALTER TABLE recurring_transactions ADD COLUMN anchor_day INTEGER")
            .execute(pool)
            .await?;
    }
    sqlx::query(
            "UPDATE recurring_transactions SET anchor_day = CAST(substr(next_run_date, 9, 2) AS INTEGER) WHERE anchor_day IS NULL",
        )
        .execute(pool)
        .await?;

    Ok(())
}
