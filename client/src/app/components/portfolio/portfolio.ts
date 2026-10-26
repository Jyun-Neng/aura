import { Component, OnInit, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { FinanceService, Investment } from '../../services/finance.service';
import { CurrencySymbolPipe } from '../../pipes/currency-symbol.pipe';

@Component({
  selector: 'app-portfolio',
  standalone: true,
  imports: [CommonModule, FormsModule, CurrencySymbolPipe],
  templateUrl: './portfolio.html',
  styleUrls: [],
})
export class PortfolioComponent implements OnInit {
  private financeService = inject(FinanceService);

  // Lists
  public investments = signal<Investment[]>([]);

  // Holdings tab, split by currency
  public investmentTab = signal<'USD' | 'TWD'>('USD');

  setInvestmentTab(currency: 'USD' | 'TWD'): void {
    this.investmentTab.set(currency);
  }

  public tabInvestments = computed(() =>
    this.investments().filter((investment) => investment.currency === this.investmentTab()),
  );

  // Investment Modal Form
  public isInvModalOpen = false;
  public invFormTitle = 'Add Investment holding';
  public invForm: Investment = {
    ticker: '',
    name: '',
    type: 'stock',
    shares: 0,
    cost_basis: 0,
    currency: 'USD',
  };

  ngOnInit(): void {
    this.loadData();
  }

  loadData(): void {
    this.financeService.getInvestments().subscribe({
      next: (res) => {
        this.investments.set(res);
      },
      error: (err) => {
        console.error('Error loading investments:', err);
      },
    });
  }

  // Investment Handlers
  openInvModal(inv?: Investment): void {
    if (inv) {
      this.invFormTitle = 'Edit Investment';
      this.invForm = { ...inv };
    } else {
      this.invFormTitle = 'Add Investment';
      this.invForm = {
        ticker: '',
        name: '',
        type: 'stock',
        shares: 0,
        cost_basis: 0,
        currency: 'USD',
      };
    }
    this.isInvModalOpen = true;
  }

  closeInvModal(): void {
    this.isInvModalOpen = false;
  }

  saveInvestment(): void {
    if (!this.invForm.ticker || !this.invForm.name) {
      alert('Please enter ticker symbol and asset name.');
      return;
    }
    this.invForm.ticker = this.invForm.ticker.toUpperCase();

    if (this.invForm.id) {
      // Update
      this.financeService.updateInvestment(this.invForm.id, this.invForm).subscribe({
        next: () => {
          this.closeInvModal();
          this.loadData();
        },
        error: (err) => alert('Failed to update investment: ' + err.error),
      });
    } else {
      // Create
      this.financeService.createInvestment(this.invForm).subscribe({
        next: () => {
          this.closeInvModal();
          this.loadData();
        },
        error: (err) => alert('Failed to create investment: ' + err.error),
      });
    }
  }

  deleteInvestment(id: string): void {
    if (confirm('Are you sure you want to delete this investment inventory?')) {
      this.financeService.deleteInvestment(id).subscribe({
        next: () => this.loadData(),
        error: (err) => alert('Failed to delete investment: ' + err.error),
      });
    }
  }
}
