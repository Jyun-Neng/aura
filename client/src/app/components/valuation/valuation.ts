import { Component, OnInit, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { FinanceService, HistoricalAsset } from '../../services/finance.service';
import { forkJoin } from 'rxjs';

function emptyHistoricalAsset(month = ''): HistoricalAsset {
  return {
    month,
    bank_usd: 0,
    bank_twd: 0,
    market_value_usd: 0,
    market_value_twd: 0,
    cost_basis_usd: 0,
    cost_basis_twd: 0,
    usd_twd_rate: 0,
  };
}

@Component({
  selector: 'app-valuation',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './valuation.html',
  styleUrls: [],
})
export class ValuationComponent implements OnInit {
  private financeService = inject(FinanceService);

  // States
  public historicalAssets = signal<HistoricalAsset[]>([]);

  // Rows enriched with combined (TWD) totals, computed using each row's own
  // logged exchange rate rather than the live rate.
  public displayAssets = computed(() => {
    return this.historicalAssets().map((snapshot) => {
      const rate = snapshot.usd_twd_rate;
      const totalBalance = snapshot.bank_usd * rate + snapshot.bank_twd;
      const totalMarketValue = snapshot.market_value_usd * rate + snapshot.market_value_twd;
      const totalCostBasis = snapshot.cost_basis_usd * rate + snapshot.cost_basis_twd;
      return { ...snapshot, totalBalance, totalMarketValue, totalCostBasis };
    });
  });

  // Historical Assets form modal
  public isHistModalOpen = false;
  public histFormTitle = 'Manage Monthly Asset Snapshot';
  public histForm: HistoricalAsset = emptyHistoricalAsset();

  ngOnInit(): void {
    this.loadData();
  }

  loadData(): void {
    this.financeService.getHistoricalAssets().subscribe({
      next: (res) => {
        this.historicalAssets.set(res);
      },
      error: (err) => console.error('Error loading historical assets:', err),
    });
  }

  // Edit / Add Historical Assets manually
  openHistModal(snapshot?: HistoricalAsset): void {
    if (snapshot) {
      this.histFormTitle = 'Edit Monthly Asset Snapshot';
      this.histForm = { ...snapshot };
    } else {
      this.histFormTitle = 'Log Monthly Asset Snapshot';
      const now = new Date();
      const currentMonthStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
      this.histForm = emptyHistoricalAsset(currentMonthStr);
    }
    this.isHistModalOpen = true;
  }

  closeHistModal(): void {
    this.isHistModalOpen = false;
  }

  saveHistoricalAsset(): void {
    const monthRegex = /^\d{4}-\d{2}$/;
    if (!monthRegex.test(this.histForm.month)) {
      alert('Please enter month in YYYY-MM format.');
      return;
    }

    this.financeService.saveHistoricalAsset(this.histForm).subscribe({
      next: () => {
        this.closeHistModal();
        this.loadData();
      },
      error: (err) => alert('Failed to save snapshot: ' + err.error),
    });
  }

  deleteHistoricalAsset(month: string): void {
    if (confirm(`Are you sure you want to delete historical snapshot for ${month}?`)) {
      this.financeService.deleteHistoricalAsset(month).subscribe({
        next: () => this.loadData(),
        error: (err) => alert('Failed to delete snapshot: ' + err.error),
      });
    }
  }

  // Automatically snapshot current assets in database
  snapshotCurrentMonth(): void {
    const now = new Date();
    const currentMonth = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;

    // We fetch bank accounts, investments, rates, prices, then aggregate and submit
    forkJoin({
      accounts: this.financeService.getBankAccounts(),
      investments: this.financeService.getInvestments(),
      val: this.financeService.getValuation(),
    }).subscribe({
      next: (res) => {
        // Calculate Bank deposits by currency actually held
        const bankUsd = res.accounts
          .filter((account) => account.currency === 'USD')
          .reduce((sum, account) => sum + account.balance, 0);
        const bankTwd = res.accounts
          .filter((account) => account.currency === 'TWD')
          .reduce((sum, account) => sum + account.balance, 0);

        // Calculate market value (stock, ETF, and fund) by currency
        const marketValueUsd = res.investments
          .filter((investment) => investment.currency === 'USD')
          .reduce((sum, investment) => {
            const priceObj = res.val.prices.find((price) => price.ticker === investment.ticker);
            const currentPrice = priceObj
              ? priceObj.price
              : investment.cost_basis / investment.shares || 0;
            return sum + investment.shares * currentPrice;
          }, 0);

        const marketValueTwd = res.investments
          .filter((investment) => investment.currency === 'TWD')
          .reduce((sum, investment) => {
            const priceObj = res.val.prices.find((price) => price.ticker === investment.ticker);
            const currentPrice = priceObj
              ? priceObj.price
              : investment.cost_basis / investment.shares || 0;
            return sum + investment.shares * currentPrice;
          }, 0);

        // Calculate cost basis (stock, ETF, and fund) by currency
        const costBasisUsd = res.investments
          .filter((investment) => investment.currency === 'USD')
          .reduce((sum, investment) => sum + investment.cost_basis, 0);
        const costBasisTwd = res.investments
          .filter((investment) => investment.currency === 'TWD')
          .reduce((sum, investment) => sum + investment.cost_basis, 0);

        const usdTwd = res.val.rates.find((rate) => rate.pair === 'USD_TWD');

        if (!usdTwd) {
          alert('Error: USD_TWD exchange rate could not be found. Snapshot aborted.');
          return; // Stop the subscription logic here
        }

        const newSnapshot: HistoricalAsset = {
          month: currentMonth,
          bank_usd: bankUsd,
          bank_twd: bankTwd,
          market_value_usd: marketValueUsd,
          market_value_twd: marketValueTwd,
          cost_basis_usd: costBasisUsd,
          cost_basis_twd: costBasisTwd,
          usd_twd_rate: usdTwd.rate,
        };

        this.financeService.saveHistoricalAsset(newSnapshot).subscribe({
          next: () => {
            alert(`Aggregated current asset balances and created snapshot for ${currentMonth}!`);
            this.loadData();
          },
          error: (err) => alert('Failed to save snapshot: ' + err.error),
        });
      },
      error: (err) => console.error('Snapshot aggregate error:', err),
    });
  }
}
