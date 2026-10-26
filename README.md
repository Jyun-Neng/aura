# AURA - Personal finance dashboard, with Claude as assistant for logging and updating

Single-user personal finance dashboard for a Taiwan-based user with USD/TWD
dual-currency holdings. Rust/Axum backend, Angular frontend, SQLite database.
Claude is the intended assistant for writing data.

## Running the app

- Server config: copy `server/config.example.yaml` to `server/config.yaml`
  (gitignored) and set `database` to your SQLite file. Without `config.yaml`
  the server uses the example, which points at `server/example/sample.db`: a
  gitignored working copy of the committed sample `server/example/aura-example.db`,
  created on first use. A relative path resolves from `server/`; `~/` is expanded.
- Backend: `cd server && cargo run` — serves on `http://127.0.0.1:3000`,
  creates any missing tables in the configured database on startup. It
  refuses to start if the database file is missing (it creates only the sample
  copy). Browsers may call the API only from `http://localhost` /
  `http://127.0.0.1` pages (any port).
- Frontend: `cd client && npm start` (= `ng serve`) — serves on
  `http://localhost:4200`.

Checks:
- Backend: `cargo fmt && cargo clippy --all-targets && cargo test && cargo build`
- Frontend: `npx ng build`, `npx ng test --watch=false`, `npx prettier --write <file>`

## Features

### Cash Ledger

Manages bank account balances (USD/TWD) and logs cash flow transactions —
income and expenses, each tagged with a category and a direction (`inflow`/`outflow`).
Also supports recurring transactions: a rule with a `next_run_date` that auto-logs
a matching Cash Flow entry every month on the same day (clamped in shorter months),
with catch-up for months the server was offline, until deleted.

### Securities Ledger

Tracks investment holdings — stocks, ETFs, and funds — with shares/units and
cost basis, denominated in USD or TWD. Currency determines the trading market

### Asset Growth Ledger

Monthly snapshots of net worth composition: bank balance, market value, and
cost basis, each split USD/TWD, plus that month's USD→TWD exchange rate. Every
historical USD amount elsewhere in the app converts to TWD using *its own
month's* logged rate (not a single live rate), so charts stay accurate as the
exchange rate drifts over time. Snapshots can be logged manually or aggregated
automatically from current data ("Snapshot Current Month").

### Valuation

Live-fetched on every request, nothing cached in the database: the current
USD/TWD exchange rate and each held ticker's closing price. Powers the
Dashboard's real-time figures and the Securities Ledger's current-value column.

### Dashboard

At-a-glance view: net worth, bank deposits, securities portfolio, and
year-to-date passive income, plus an Asset Allocation section with four pie
charts (Balance vs. Investment, USD vs. TWD Investment, and a per-holding
breakdown for each currency), switchable between Cost Basis and Market Value.

### Analytics

Historical trend charts, aggregated server-side from the Asset Growth Ledger
and Cash Ledger:

- **Asset Growth Trend** — Total Net Worth, Total Bank Balance, or Total Market
  Value over time (one series shown at a time, since Bank Balance is on a much
  smaller scale than the others), always in TWD using each month's logged rate.
- **Cash Flow Timeline** — monthly inflows vs. outflows.
- **Passive Income Trend** — Interest Income + Investment Distributions,
  combined into a single monthly series.
- **Unrealized Gain/Loss** — Total Market Value minus Total Cost Basis, per
  month.
