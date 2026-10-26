import {
  Component,
  OnInit,
  OnDestroy,
  ElementRef,
  ViewChild,
  inject,
  signal,
  computed,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import {
  FinanceService,
  BankAccount,
  Investment,
  CashFlow,
  FxRate,
  TickerPrice,
} from '../../services/finance.service';
import { CurrencySymbolPipe } from '../../pipes/currency-symbol.pipe';
import { forkJoin } from 'rxjs';
import { Chart } from 'chart.js/auto';

// Catppuccin Mocha Color Palette Constants
const MAUVE = '#cba6f7';
const GREEN = '#a6e3a1';
const PEACH = '#fab387';
const BLUE = '#89b4fa';
const YELLOW = '#f9e2af';
const TEAL = '#94e2d5';
const RED = '#f38ba8';
const PINK = '#f5c2e7';
const SAPPHIRE = '#74c7ec';
const LAVENDER = '#b4befe';
const MAROON = '#eba0ac';
const SKY = '#89dceb';
const TEXT = '#cdd6f4';
const SURFACE2 = '#585b70';

// Cycled per slice for charts with a variable, per-holding number of slices.
const HOLDING_PALETTE = [
  MAUVE,
  GREEN,
  PEACH,
  BLUE,
  YELLOW,
  TEAL,
  RED,
  PINK,
  SAPPHIRE,
  LAVENDER,
  MAROON,
  SKY,
];

// A single pie slice, shared between the Chart.js dataset and the side list
// that shows each slice's percentage (in-slice labels get unreadable once a
// slice is small, e.g. a minor holding in a large portfolio).
interface PieSlice {
  label: string;
  color: string;
  value: number;
  displayAmount: string;
  pct: number;
}

function toSlices(
  entries: { label: string; value: number }[],
  colors: string[],
  symbol: string,
): PieSlice[] {
  const total = entries.reduce((sum, entry) => sum + entry.value, 0);
  return entries.map((entry, index) => ({
    label: entry.label,
    color: colors[index % colors.length],
    value: entry.value,
    displayAmount: `${symbol} ${entry.value.toLocaleString(undefined, { maximumFractionDigits: 0 })}`,
    pct: total > 0 ? (entry.value / total) * 100 : 0,
  }));
}

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule, RouterLink, CurrencySymbolPipe],
  templateUrl: './dashboard.html',
  styleUrls: [],
})
export class DashboardComponent implements OnInit, OnDestroy {
  private financeService = inject(FinanceService);

  // Raw API Data
  public bankAccounts = signal<BankAccount[]>([]);
  public investments = signal<Investment[]>([]);
  public cashFlows = signal<CashFlow[]>([]);
  public fxRates = signal<FxRate[]>([]);
  public tickerPrices = signal<TickerPrice[]>([]);

  // Investment Holdings tab, split by currency (native amounts, no conversion)
  public investmentTab = signal<'USD' | 'TWD'>('USD');

  setInvestmentTab(currency: 'USD' | 'TWD'): void {
    this.investmentTab.set(currency);
  }

  // Computed Exchange Rate
  public usdTwdRate = computed(() => {
    const fxRate = this.fxRates().find((rate) => rate.pair === 'USD_TWD');
    return fxRate ? fxRate.rate : 32.5;
  });

  private convertToTwd = (amountUsd: number, rate: number): number => amountUsd * rate;

  // Bank deposits, bucketed by the currency actually held.
  public totalBankUsd = computed(() =>
    this.bankAccounts()
      .filter((account) => account.currency === 'USD')
      .reduce((sum, account) => sum + account.balance, 0),
  );

  public totalBankTwd = computed(() =>
    this.bankAccounts()
      .filter((account) => account.currency === 'TWD')
      .reduce((sum, account) => sum + account.balance, 0),
  );

  public totalBank = computed(
    () => this.convertToTwd(this.totalBankUsd(), this.usdTwdRate()) + this.totalBankTwd(),
  );

  // Stock/fund/ETF holdings, bucketed by currency.
  public totalStockUsd = computed(() => {
    const prices = this.tickerPrices();
    return this.investments()
      .filter((investment) => investment.currency === 'USD')
      .reduce((sum, investment) => {
        const priceObj = prices.find((price) => price.ticker === investment.ticker);
        const currentPrice = priceObj
          ? priceObj.price
          : investment.cost_basis / investment.shares || 0;
        return sum + investment.shares * currentPrice;
      }, 0);
  });

  public totalStockTwd = computed(() => {
    const prices = this.tickerPrices();
    return this.investments()
      .filter((investment) => investment.currency === 'TWD')
      .reduce((sum, investment) => {
        const priceObj = prices.find((price) => price.ticker === investment.ticker);
        const currentPrice = priceObj
          ? priceObj.price
          : investment.cost_basis / investment.shares || 0;
        return sum + investment.shares * currentPrice;
      }, 0);
  });

  public totalStock = computed(
    () => this.convertToTwd(this.totalStockUsd(), this.usdTwdRate()) + this.totalStockTwd(),
  );

  public netWorth = computed(() => this.totalBank() + this.totalStock());

  // Cost basis (what was originally paid), bucketed by currency.
  public totalCostBasisUsd = computed(() =>
    this.investments()
      .filter((investment) => investment.currency === 'USD')
      .reduce((sum, investment) => sum + investment.cost_basis, 0),
  );

  public totalCostBasisTwd = computed(() =>
    this.investments()
      .filter((investment) => investment.currency === 'TWD')
      .reduce((sum, investment) => sum + investment.cost_basis, 0),
  );

  public totalCostBasis = computed(
    () => this.convertToTwd(this.totalCostBasisUsd(), this.usdTwdRate()) + this.totalCostBasisTwd(),
  );

  // Passive income (Interest Income + Investment Distributions) from Jan 1 of
  // the current year through today, converted to TWD at the live rate.
  public passiveIncomeYtd = computed(() => {
    const rate = this.usdTwdRate();
    const now = new Date();
    const yearStart = `${now.getFullYear()}-01-01`;
    const todayStr = now.toISOString().substring(0, 10);
    const passiveCategories = ['Interest Income', 'Investment Distributions'];

    return this.cashFlows()
      .filter(
        (transaction) =>
          passiveCategories.includes(transaction.category) &&
          transaction.date >= yearStart &&
          transaction.date <= todayStr,
      )
      .reduce(
        (sum, transaction) =>
          sum + (transaction.currency === 'USD' ? transaction.amount * rate : transaction.amount),
        0,
      );
  });

  // Recent Cash Flows
  public recentFlows = computed(() => {
    return this.cashFlows().slice(0, 5);
  });

  // Investment Holdings table data, shown in each holding's native currency
  public dashboardInvestments = computed(() => {
    const prices = this.tickerPrices();

    return this.investments().map((investment) => {
      const priceObj = prices.find((price) => price.ticker === investment.ticker);
      const currentPrice = priceObj
        ? priceObj.price
        : investment.cost_basis / investment.shares || 0;
      const marketValue = investment.shares * currentPrice;
      const profit = marketValue - investment.cost_basis;
      const profitPct = investment.cost_basis > 0 ? (profit / investment.cost_basis) * 100 : 0;

      return {
        ...investment,
        currentPrice,
        marketValue,
        profit,
        profitPct,
      };
    });
  });

  public tabInvestments = computed(() =>
    this.dashboardInvestments().filter(
      (investment) => investment.currency === this.investmentTab(),
    ),
  );

  // Totals for the currently selected tab, in that tab's native currency
  public tabTotals = computed(() => {
    const holdings = this.tabInvestments();
    const marketValue = holdings.reduce((sum, holding) => sum + holding.marketValue, 0);
    const costBasis = holdings.reduce((sum, holding) => sum + holding.cost_basis, 0);
    const profit = marketValue - costBasis;
    const profitPct = costBasis > 0 ? (profit / costBasis) * 100 : 0;
    return { marketValue, profit, profitPct };
  });

  // Asset Allocation — 4 pie charts, all shown together, all driven by a
  // single Cost Basis / Market Value tab:
  //   1. Bank Balance vs Investment
  //   2. USD vs TWD (within Investment)
  //   3. USD holdings breakdown
  //   4. TWD holdings breakdown
  public allocationMetric = signal<'cost_basis' | 'marketValue'>('cost_basis');

  setAllocationMetric(metric: 'cost_basis' | 'marketValue'): void {
    this.allocationMetric.set(metric);
    this.renderAllocationCharts();
  }

  private investmentTotal(metric: 'cost_basis' | 'marketValue'): number {
    return metric === 'marketValue' ? this.totalStock() : this.totalCostBasis();
  }

  public balanceSlices = computed<PieSlice[]>(() =>
    toSlices(
      [
        { label: 'Bank Balance', value: this.totalBank() },
        { label: 'Investment', value: this.investmentTotal(this.allocationMetric()) },
      ],
      [GREEN, PEACH],
      'NT$',
    ),
  );

  public currencySlices = computed<PieSlice[]>(() => {
    const metric = this.allocationMetric();
    const usd = metric === 'marketValue' ? this.totalStockUsd() : this.totalCostBasisUsd();
    const twd = metric === 'marketValue' ? this.totalStockTwd() : this.totalCostBasisTwd();
    return toSlices(
      [
        { label: 'USD', value: this.convertToTwd(usd, this.usdTwdRate()) },
        { label: 'TWD', value: twd },
      ],
      [BLUE, YELLOW],
      'NT$',
    );
  });

  private holdingSlices(currency: 'USD' | 'TWD'): PieSlice[] {
    const metric = this.allocationMetric();
    const entries = this.dashboardInvestments()
      .filter((investment) => investment.currency === currency)
      .map((investment) => ({
        label: investment.name,
        value: metric === 'marketValue' ? investment.marketValue : investment.cost_basis,
      }))
      .filter((entry) => entry.value > 0)
      .sort((a, b) => b.value - a.value);

    return toSlices(entries, HOLDING_PALETTE, currency === 'USD' ? '$' : 'NT$');
  }

  public usdHoldingsSlices = computed<PieSlice[]>(() => this.holdingSlices('USD'));
  public twdHoldingsSlices = computed<PieSlice[]>(() => this.holdingSlices('TWD'));

  @ViewChild('balanceCanvas') balanceCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('currencyCanvas') currencyCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('usdHoldingsCanvas') usdHoldingsCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('twdHoldingsCanvas') twdHoldingsCanvas!: ElementRef<HTMLCanvasElement>;
  private balanceChart: Chart | null = null;
  private currencyChart: Chart | null = null;
  private usdHoldingsChart: Chart | null = null;
  private twdHoldingsChart: Chart | null = null;

  ngOnInit(): void {
    this.refreshData();
  }

  ngOnDestroy(): void {
    this.balanceChart?.destroy();
    this.currencyChart?.destroy();
    this.usdHoldingsChart?.destroy();
    this.twdHoldingsChart?.destroy();
  }

  refreshData(): void {
    forkJoin({
      accounts: this.financeService.getBankAccounts(),
      investments: this.financeService.getInvestments(),
      cashFlows: this.financeService.getCashFlows(),
      valuation: this.financeService.getValuation(),
    }).subscribe({
      next: (res) => {
        this.bankAccounts.set(res.accounts);
        this.investments.set(res.investments);
        this.cashFlows.set(res.cashFlows);
        this.fxRates.set(res.valuation.rates);
        this.tickerPrices.set(res.valuation.prices);
        // Wait for next tick so the canvas references are active
        setTimeout(() => this.renderAllocationCharts(), 0);
      },
      error: (err) => {
        console.error('Error fetching dashboard data:', err);
      },
    });
  }

  private renderAllocationCharts(): void {
    this.balanceChart = this.renderPieChart(
      this.balanceCanvas,
      this.balanceChart,
      this.balanceSlices(),
    );
    this.currencyChart = this.renderPieChart(
      this.currencyCanvas,
      this.currencyChart,
      this.currencySlices(),
    );
    this.usdHoldingsChart = this.renderPieChart(
      this.usdHoldingsCanvas,
      this.usdHoldingsChart,
      this.usdHoldingsSlices(),
    );
    this.twdHoldingsChart = this.renderPieChart(
      this.twdHoldingsCanvas,
      this.twdHoldingsChart,
      this.twdHoldingsSlices(),
    );
  }

  // Renders straight off a PieSlice[] computed signal, so each chart and its
  // side list (which carries the percentage — in-slice labels become
  // unreadable once a slice is a few percent or smaller) always agree.
  private renderPieChart(
    canvas: ElementRef<HTMLCanvasElement> | undefined,
    existing: Chart | null,
    slices: PieSlice[],
  ): Chart | null {
    existing?.destroy();
    if (!canvas || slices.length === 0) return null;

    return new Chart(canvas.nativeElement, {
      type: 'pie',
      data: {
        labels: slices.map((slice) => slice.label),
        datasets: [
          {
            data: slices.map((slice) => slice.value),
            backgroundColor: slices.map((slice) => slice.color),
            borderColor: '#1e1e2e',
            borderWidth: 2,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: { display: false },
          tooltip: {
            backgroundColor: '#1e1e2e',
            titleColor: MAUVE,
            bodyColor: TEXT,
            borderColor: SURFACE2,
            borderWidth: 1,
            titleFont: { family: 'Outfit', weight: 'bold' as const },
            bodyFont: { family: 'Outfit' },
            callbacks: {
              label: (context) => {
                const slice = slices[context.dataIndex];
                return `${slice.label}: ${slice.displayAmount} (${slice.pct.toFixed(1)}%)`;
              },
            },
          },
        },
      },
    });
  }
}
