# AURA — Client

Angular frontend for AURA, a personal finance dashboard for a Taiwan-based user with
USD/TWD dual-currency holdings. Talks to the Rust/Axum backend in `../server` over a
REST API (see `../CLAUDE.md` for the full API reference and data conventions).

## Pages

- **Dashboard** — net worth, holdings, recent transactions, and the Asset Allocation
  pie charts.
- **Cash Ledger** (`/cashflow`) — bank accounts, cash flow transactions, and recurring
  (auto-logged) transactions.
- **Securities Ledger** (`/portfolio`) — stock/ETF/fund holdings.
- **Asset Growth Ledger** (`/valuation`) — monthly net-worth snapshots.
- **Analytics** (`/charts`) — asset growth, cash flow, passive income, and
  unrealized gain/loss trend charts.

## Development server

```bash
npm start   # = ng serve, http://localhost:4200
```

The backend (`cd ../server && cargo run`) must be running on `http://127.0.0.1:3000`
for any page that fetches data.

## Building

```bash
npx ng build
```

Compiles the project into `dist/`.

## Running unit tests

Uses the [Vitest](https://vitest.dev/) test runner:

```bash
npx ng test --watch=false
```

## Formatting

```bash
npx prettier --write <file>
```

Note: Prettier's HTML formatter expands inline SVG icons (used throughout the
templates) onto many lines. Run it on `.ts` files freely; be prepared to manually
re-collapse SVGs if you run it on `.html` files.

## Additional Resources

This project was scaffolded with [Angular CLI](https://github.com/angular/angular-cli).
For CLI command references, see the
[Angular CLI Overview and Command Reference](https://angular.dev/tools/cli).
