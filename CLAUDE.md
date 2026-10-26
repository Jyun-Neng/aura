## Core Rules

- The backend must be running for any API call to work.
- **Use the API Reference below when building a request body — don't guess shapes or the category enum from memory.**
- **Mutate data through the API, never edit the database file directly.** Every write (`POST`/`PUT`/`DELETE`) is checked — category, currency and type against the schema's allowed values, and dates (`YYYY-MM-DD`) and months (`YYYY-MM`) by format — and new records get their UUID server-side. Editing the SQLite file directly bypasses that and risks a CHECK constraint mismatch between rows. Use `curl` against the endpoints below for every data change, whether it's a one-off task or part of a larger change.

## Running the App

- Server config: copy `server/config.example.yaml` to `server/config.yaml`
  (gitignored) and set `database` to your SQLite file. Without `config.yaml`
  the server uses the example, which points at `server/example/sample.db`: a
  gitignored working copy of the committed sample `server/example/aura-example.db`,
  created on first use. A relative path resolves from `server/`; `~/` is expanded.
- Backend: `cd server && cargo run` — serves on `http://127.0.0.1:3000`,
  creates any missing tables in the configured database on startup. It
  refuses to start if the database file is missing (it creates only the sample
  copy). Browsers may call the API only from `http://localhost` /
  `http://127.0.0.1` pages (any port); `curl` is unaffected.
- Frontend: `cd client && npm start` (= `ng serve`) — serves on
  `http://localhost:4200`.

Checks:
- Backend: `cargo fmt && cargo clippy --all-targets && cargo test && cargo build`
- Frontend: `npx ng build`, `npx ng test --watch=false`, `npx prettier --write <file>`

## Database

All personal financial data is stored in a SQLite database at the path set by `database` in `server/config.yaml` (or `server/config.example.yaml` if that's missing). Read it for the location; never commit `config.yaml`. Don't add database files to this repo other than the sample `server/example/aura-example.db`. Tables: `cash_flow`, `bank_accounts`, `investments`, `historical_assets`, `recurring_transactions`.

`category` classifies each `cash_flow` row by type of income/expense, so spending and income can be broken down and tracked by category (e.g. "how much did I spend on X this year"). It is constrained by the schema (`CHECK` constraint) to a fixed list — always pick from it, never invent a new category. `type` must match the category's direction below (Salary is always `inflow`, Food & Dining is always `outflow`, etc.) — infer it from the category, don't guess.

- Inflow: `Salary`, `Bonus`, `Interest Income`, `Investment Distributions`, `Cashback Rewards`, `Other Income`
- Outflow: `Loan Amortizations`, `Fixed Deductions`, `Food & Dining`, `Transportation`, `Entertainment`, `Baby Supplies`, `Pet Supplies`, `Daily Living`, `Subscriptions`, `Insurance`, `Utilities`, `Gifts`, `Taxation`, `Other Expense`

### Category Conventions

- **Utilities**: recurring telecom/utility bills (mobile, home internet, electricity, water, gas).
- **Gifts**: gift-giving expenses (e.g. Mother's Day, Red Envelope, duty-free purchases bought as a gift), not `Other Expense`.
- **Grocery purchases are not a category** — there's no `Groceries` entry in the enum. Route each one to either `Food & Dining` or `Daily Living` depending on what the trip was mostly for (food vs. household/personal-care items) — there's no single fixed rule here, and it's fine if it varies transaction to transaction.
- **Baby Supplies vs. Daily Living (medical)**: the distinction is *whose body the treatment is for*, not whether it's baby-related in a loose sense. Medical care for a parent is `Daily Living`, even when it's baby-related. Only expenses actually for the baby itself (supplies, the baby's own medical care/vaccines, education/investment fund contributions for the child) are `Baby Supplies`. There's no dedicated Medical category — a vaccine, for example, is `Baby Supplies` if given to the child and `Daily Living` if given to a parent; check which one per transaction when it's not obvious.
- **Pet Supplies**: same pattern as Baby Supplies — pet-related medical/vet costs go here, not `Daily Living`.
- **Fixed Deductions vs. Insurance**: anything deducted via payroll withholding (tax, insurance premiums, pension, etc.) is `Fixed Deductions`. Insurance paid directly by you (not via payroll) is `Insurance`.

## Credit-Card Transaction Logging

**Trigger:** run this workflow when I say any of the following (or a close paraphrase), which all mean "log my card statement expenses into the DB":
- "Help me track my expenses this month"
- "Track / log my expenses for [month]"
- "Log my transactions" / "log my credit-card transactions [for month]"
- "Update the database with this/last month's expenses"

When no month is specified, default to the current month (Taipei time).

**Key convention — everything is keyed off the statement's payment due date (繳款截止日), not the actual purchase date.** A statement due 2026/07/06 belongs to July 2026, and every row logged from it gets `date` = that due date, even though the underlying purchases happened on different days in June. This is intentional: it tracks "which month did I pay this," not a purchase-date timeline.

**If logging the current month**, always download statements first. If the download produces no statements at all, stop the task immediately and notify me — that means the download itself is broken.

**If logging a month other than the current one**, skip the download — it only ever fetches the newest statement, so it can't retrieve an older month's file anyway. Instead, first ask me to confirm whether that month's statement file already exists. If it doesn't exist, stop — there's nothing to log yet. If it does, proceed using that file.

If you don't know where to download statements or where the statements are, ask me to confirm.

Then, for each statement:

1. **Month selection by due date:** include a statement only if its due date falls in the month I asked to log (see convention above).
2. **One row per purchase, fees merged:** merge each foreign-transaction fee into the purchase that caused it and log a single combined item. Never log fees as separate rows.
3. **Transaction date = due date:** stamp every logged row's `date` with the statement's due date (see convention above), so all rows land in the logging month.
4. **Description format — clean, not raw statement text.** Write each `description` as a short, human-readable merchant name matching the style of existing rows (e.g. `Netflix`, `Home Internet`, `Car Maintenance`). Specifically:
   - Use a normalized English merchant name. Translate/normalize Chinese names when a common English name exists (`好味道小吃店` → `Good Taste Eatery`).
   - Add a location qualifier in parentheses when useful for disambiguation — city, state, or airport code (`Target (Springfield IL)`, `Corner Bookstore (Portland)`, `Duty-Free Shop (JFK)`).
   - Strip country suffixes (`US`, `SG`), store/terminal numbers (`T-2725`, `#452`), and payment-channel noise.
   - Do NOT annotate the merged foreign-transaction fee in the description; the fee is folded silently into `amount`.
5. **Skip transactions already recorded.** Before inserting each statement line, check for a match in two places:
   - `recurring_transactions` (and recurring items already written into `cash_flow`, e.g. Netflix, Home Internet) — match by merchant/description only, not amount, since the recurring entry and the statement line can differ (e.g. a recurring Netflix entry of NT$300 vs. a statement line of NT$320 + fee); it's still the same subscription and must be skipped.
   - `cash_flow` rows already logged in the same month (match by month only, not the exact date) — match by description and amount; skip if a match already exists, since it likely means this month has already been logged.
   If a row is similar but not clearly identical to an existing one (e.g. close but not exact description or amount) and you aren't sure whether it's the same transaction, don't decide either way — ask me to confirm before logging.

   Note: `GET /api/cash-flow` may itself insert newly-due recurring-transaction rows as a side effect (see Recurring Transactions API below) — that's intentional server behavior, not a bug. Any such rows are legitimate and should be treated as already-logged, same as any other match above.
6. **Log without per-item approval, then report:** writing these rows to the database does not require approval. Afterward, always summarize what was logged. Highlight any item whose category I guessed/was unsure about.

Exclude non-expense lines (bill autopay/payment of the prior balance, NT$0 reward/points entries). Record purchases as `type='outflow'`.

## API Reference

All bodies are JSON. `id` is generated server-side on create — never send one.
`PUT` and `DELETE` return `404` if no record has that id/month; invalid dates or
months return `400`.

### Bank Accounts — `/api/bank-accounts`
`GET` list · `POST` create · `PUT /:id` update · `DELETE /:id`
```json
{ "name": "Mega Bank Checking", "currency": "USD" | "TWD", "balance": 12345.67 }
```

### Investments — `/api/investments`
`GET` list · `POST` create · `PUT /:id` update · `DELETE /:id`
```json
{
  "ticker": "AAPL",
  "name": "Apple Inc.",
  "type": "stock" | "etf" | "fund",
  "shares": 10.5,
  "cost_basis": 4000.00,
  "currency": "USD" | "TWD"
}
```
`currency` determines the trading market, not `ticker` alone — a TWD ticker without a
`.` in it (e.g. `0050`) gets a `.TW` suffix automatically when the server fetches its
live price from Yahoo Finance. Don't add `.TW` yourself.

### Cash Flow — `/api/cash-flow`
`GET` list · `POST` create · `PUT /:id` update · `DELETE /:id`
```json
{
  "date": "2026-07-19",
  "description": "Dinner with friends",
  "category": "Food & Dining",
  "amount": 850.0,
  "currency": "USD" | "TWD",
  "type": "inflow" | "outflow"
}
```
`category` must be exactly one of the values listed under **Database** above
(case-sensitive, or the insert fails a CHECK constraint); `type` must match that
category's listed direction.

### Recurring Transactions — `/api/recurring-transactions`
`GET` list · `POST` create · `PUT /:id` update · `DELETE /:id`
Same shape as Cash Flow, minus `date`, plus `next_run_date` (`YYYY-MM-DD`). The server
auto-logs a matching Cash Flow entry (and advances `next_run_date` by one month) every time `GET /api/cash-flow` is called and
the rule is due — including catch-up for months the server was offline. Deleting the
rule is how the user "cancels" a recurring transaction; there's no separate pause state.
Each rule repeats on the day of month of the `next_run_date` it was created with (or
last changed to), clamped in shorter months: a rule on the 30th runs Jan 30, Feb 28,
Mar 30. Editing a rule without changing `next_run_date` keeps its day.

### Historical Assets (Asset Growth Ledger Snapshots) — `/api/historical-assets`
`GET` list · `POST` create-or-replace (same `month` overwrites) · `DELETE /:month`
```json
{
  "month": "2026-07",
  "bank_usd": 5000.0, "bank_twd": 800000.0,
  "market_value_usd": 50000.0, "market_value_twd": 600000.0,
  "cost_basis_usd": 40000.0, "cost_basis_twd": 500000.0,
  "usd_twd_rate": 32.13
}
```
Every USD amount elsewhere in the app that needs to be shown or summed in TWD is
converted using **that transaction's own month's** `usd_twd_rate` from this table (found
by matching `date[0:7]` to `month`), not a single live rate — this is what makes
historical charts accurate as the exchange rate drifts over time. Get this value right.

### Valuation (read-only) — `GET /api/valuation`
Live-fetched, not cached: `{ "rates": [{ "pair": "USD_TWD", "rate": 32.13 }], "prices": [{ "ticker": "AAPL", "price": 213.5, "currency": "USD" }] }`

### Aggregated Charts (read-only) — `GET /api/charts/aggregated`
Powers the Analytics page; not generally needed for data-entry tasks.

## Snapshotting Monthly Wealth

There's no dedicated endpoint for this — in the browser it's client-side
aggregation logic (`valuation.ts`'s "Snapshot Current Month" button), not a backend
feature. Replicate it exactly:

1. `GET /api/bank-accounts` → sum `balance` separately for USD and TWD accounts →
   `bank_usd`, `bank_twd`.
2. `GET /api/investments` → sum `cost_basis` separately for USD and TWD holdings →
   `cost_basis_usd`, `cost_basis_twd`.
3. `GET /api/valuation` → get `prices` (by ticker) and the `USD_TWD` rate.
4. For each investment, current value = `shares * price` (fall back to
   `cost_basis / shares` if that ticker has no live price). Sum by currency →
   `market_value_usd`, `market_value_twd`.
5. `POST /api/historical-assets` with `month` = current `YYYY-MM`, the six sums above,
   and `usd_twd_rate` from step 3.

## Keeping This File Accurate

This file owns everything above — Core Rules, run/check commands, the database schema
and category conventions, the credit-card logging workflow, the API reference, and the
snapshot recipe. If a change touches any of it — a new/renamed endpoint, a changed
field, an added/removed category, a changed run/check command, or a change to the
snapshot procedure — update this file in the same change. Run/check commands and the
feature set are also documented in `README.md` for human readers; this file never
reads from it, but if your change affects what's written there, update `README.md`
too, in the same change. A stale doc is worse than no doc, since it actively misleads
whoever (or whatever) reads it next instead of just leaving them to check the source.
