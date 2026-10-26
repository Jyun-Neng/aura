// Format checks for date fields the schema stores as plain TEXT. Dates must
// be zero-padded so they sort and group correctly as strings.

use chrono::NaiveDate;

/// `YYYY-MM-DD`, a real calendar date.
pub fn is_date(s: &str) -> bool {
    s.len() == 10 && NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

/// `YYYY-MM`, a real month.
pub fn is_month(s: &str) -> bool {
    s.len() == 7 && is_date(&format!("{}-01", s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert!(is_date("2026-07-06"));
        assert!(is_date("2028-02-29"));
        assert!(!is_date("2026-02-30"));
        assert!(!is_date("2026/07/06"));
        assert!(!is_date("2026-7-6"));
        assert!(!is_date("2026-07-06T00:00"));
        assert!(!is_date(""));
    }

    #[test]
    fn months() {
        assert!(is_month("2026-07"));
        assert!(!is_month("2026-13"));
        assert!(!is_month("2026-7"));
        assert!(!is_month("July"));
        assert!(!is_month("2026-07-01"));
    }
}
