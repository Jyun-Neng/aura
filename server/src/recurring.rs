use chrono::{Datelike, NaiveDate};
use tokio::sync::Mutex;
use uuid::Uuid;

// Serializes catch-up runs so two concurrent requests can't log the same
// occurrence twice.
static PROCESS_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(sqlx::FromRow)]
struct DueTransaction {
    id: String,
    description: String,
    category: String,
    amount: f64,
    currency: String,
    r#type: String,
    next_run_date: String,
    anchor_day: Option<i64>,
}

/// The day of month in a `YYYY-MM-DD` date, used as a rule's anchor day.
pub fn day_of_month(date_str: &str) -> u32 {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map(|d| d.day())
        .unwrap_or(1)
}

// Returns the date one calendar month after `date_str`, on `anchor_day`,
// clamped to the last day of that month: with anchor 31, Jan 31 -> Feb 28 ->
// Mar 31. None if `date_str` isn't a valid YYYY-MM-DD date.
pub fn next_occurrence(date_str: &str, anchor_day: u32) -> Option<String> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    let first_of_following = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }?;
    let last_day = first_of_following.pred_opt()?.day();

    NaiveDate::from_ymd_opt(year, month, anchor_day.clamp(1, last_day))
        .map(|d| d.format("%Y-%m-%d").to_string())
}

// Logs a new cash_flow entry for every recurring transaction whose next run
// date has arrived, advancing each one month at a time so the server catches
// up correctly even if it wasn't running for a while. All-or-nothing: a
// failure partway leaves nothing logged and no date advanced.
pub async fn process_due_recurring_transactions(
    pool: &sqlx::SqlitePool,
) -> Result<(), sqlx::Error> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    process_due_as_of(pool, &today).await
}

async fn process_due_as_of(pool: &sqlx::SqlitePool, today: &str) -> Result<(), sqlx::Error> {
    let _guard = PROCESS_LOCK.lock().await;
    let mut tx = pool.begin().await?;

    let due_transactions = sqlx::query_as::<_, DueTransaction>(
        "SELECT id, description, category, amount, currency, type, next_run_date, anchor_day FROM recurring_transactions WHERE next_run_date <= ?",
    )
    .bind(today)
    .fetch_all(&mut *tx)
    .await?;

    for due in due_transactions {
        let anchor_day = due
            .anchor_day
            .map(|d| d as u32)
            .unwrap_or_else(|| day_of_month(&due.next_run_date));
        let mut next_date = due.next_run_date.clone();
        while next_date.as_str() <= today {
            sqlx::query(
                "INSERT INTO cash_flow (id, date, description, category, amount, currency, type) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(&next_date)
            .bind(&due.description)
            .bind(&due.category)
            .bind(due.amount)
            .bind(&due.currency)
            .bind(&due.r#type)
            .execute(&mut *tx)
            .await?;

            next_date = match next_occurrence(&next_date, anchor_day) {
                Some(d) => d,
                None => {
                    eprintln!(
                        "Recurring transaction {} has an invalid next_run_date {:?}; skipped.",
                        due.id, next_date
                    );
                    break;
                }
            };
        }

        sqlx::query("UPDATE recurring_transactions SET next_run_date = ? WHERE id = ?")
            .bind(&next_date)
            .bind(&due.id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_anchor_day_after_short_months() {
        let mut date = "2027-01-30".to_string();
        let mut seen = vec![];
        for _ in 0..3 {
            date = next_occurrence(&date, 30).unwrap();
            seen.push(date.clone());
        }
        assert_eq!(seen, ["2027-02-28", "2027-03-30", "2027-04-30"]);
    }

    #[test]
    fn clamps_to_the_last_day_and_crosses_years() {
        assert_eq!(next_occurrence("2027-12-31", 31).unwrap(), "2028-01-31");
        assert_eq!(next_occurrence("2028-01-31", 31).unwrap(), "2028-02-29");
        assert_eq!(next_occurrence("2026-08-31", 31).unwrap(), "2026-09-30");
        assert_eq!(next_occurrence("2026-11-15", 15).unwrap(), "2026-12-15");
        assert!(next_occurrence("2026/11/15", 15).is_none());
    }

    async fn test_pool() -> sqlx::SqlitePool {
        // One connection, so every query sees the same in-memory database.
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::db::create_schema(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn catches_up_once_and_is_idempotent() {
        let pool = test_pool().await;
        sqlx::query(
            "INSERT INTO recurring_transactions (id, description, category, amount, currency, type, next_run_date, anchor_day)
             VALUES ('r1', 'Rent', 'Daily Living', 100.0, 'TWD', 'outflow', '2027-01-30', 30)",
        )
        .execute(&pool)
        .await
        .unwrap();

        process_due_as_of(&pool, "2027-04-01").await.unwrap();
        process_due_as_of(&pool, "2027-04-01").await.unwrap();

        let dates: Vec<String> = sqlx::query_scalar("SELECT date FROM cash_flow ORDER BY date")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(dates, ["2027-01-30", "2027-02-28", "2027-03-30"]);

        let next: String =
            sqlx::query_scalar("SELECT next_run_date FROM recurring_transactions WHERE id = 'r1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(next, "2027-04-30");
    }
}
