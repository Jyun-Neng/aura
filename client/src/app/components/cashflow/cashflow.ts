import { Component, OnInit, inject, signal, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import {
  FinanceService,
  CashFlow,
  BankAccount,
  RecurringTransaction,
} from '../../services/finance.service';
import { CurrencySymbolPipe } from '../../pipes/currency-symbol.pipe';

// Adds one calendar month to a YYYY-MM-DD date, clamping the day to the last
// day of the target month (e.g. Jan 31 -> Feb 28/29). Mirrors the server's
// add_one_month() so the first scheduled run lines up with later catch-ups.
function addOneMonthClamped(dateStr: string): string {
  const [y, m, d] = dateStr.split('-').map(Number);
  let year = y;
  let month = m + 1;
  if (month > 12) {
    month = 1;
    year++;
  }
  const lastDayOfMonth = new Date(year, month, 0).getDate();
  const day = Math.min(d, lastDayOfMonth);
  return `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
}

@Component({
  selector: 'app-cashflow',
  standalone: true,
  imports: [CommonModule, FormsModule, CurrencySymbolPipe],
  templateUrl: './cashflow.html',
  styleUrls: [],
})
export class CashflowComponent implements OnInit {
  private financeService = inject(FinanceService);

  // Lists
  public bankAccounts = signal<BankAccount[]>([]);
  public cashFlows = signal<CashFlow[]>([]);
  public recurringTransactions = signal<RecurringTransaction[]>([]);

  // Collapsed by default; the user expands it to check or manage automations
  public showRecurringPanel = false;

  // Bank Accounts tab (split by currency actually held)
  public accountTab = signal<'USD' | 'TWD'>('USD');

  setAccountTab(currency: 'USD' | 'TWD'): void {
    this.accountTab.set(currency);
  }

  public tabAccounts = computed(() =>
    this.bankAccounts().filter((account) => account.currency === this.accountTab()),
  );

  // Bank Account Modal Form
  public isAccModalOpen = false;
  public accFormTitle = 'Add Bank Account';
  public accForm: BankAccount = {
    name: '',
    currency: 'USD',
    balance: 0,
  };

  // Modal Form State
  public isModalOpen = false;
  public formTitle = 'Log Transaction';

  public categories = [
    // Inflows
    'Salary',
    'Bonus',
    'Interest Income',
    'Investment Distributions',
    'Cashback Rewards',
    'Other Income',
    // Outflows
    'Loan Amortizations',
    'Fixed Deductions',
    'Food & Dining',
    'Transportation',
    'Entertainment',
    'Baby Supplies',
    'Pet Supplies',
    'Daily Living',
    'Subscriptions',
    'Insurance',
    'Utilities',
    'Gifts',
    'Taxation',
    'Other Expense',
  ];

  public flowForm: CashFlow = {
    date: new Date().toISOString().substring(0, 10),
    description: '',
    category: 'Daily Living',
    amount: 0,
    currency: 'USD',
    type: 'outflow',
  };

  // Only offered when logging a brand-new transaction (not when editing one)
  public flowIsRecurring = false;

  // Recurring Transaction Modal Form
  public isRecurringModalOpen = false;
  public recurringFormTitle = 'Edit Automatic Transaction';
  public recurringForm: RecurringTransaction = {
    description: '',
    category: 'Daily Living',
    amount: 0,
    currency: 'USD',
    type: 'outflow',
    next_run_date: '',
  };

  ngOnInit(): void {
    this.loadAccounts();
    this.loadData();
  }

  loadAccounts(): void {
    this.financeService.getBankAccounts().subscribe({
      next: (res) => this.bankAccounts.set(res),
      error: (err) => console.error('Error loading bank accounts:', err),
    });
  }

  loadRecurringTransactions(): void {
    this.financeService.getRecurringTransactions().subscribe({
      next: (res) => this.recurringTransactions.set(res),
      error: (err) => console.error('Error loading recurring transactions:', err),
    });
  }

  loadData(): void {
    // Fetching the registry also triggers the server's recurring-transaction
    // catch-up, so refresh the recurring list afterwards to pick up any
    // advanced next_run_date.
    this.financeService.getCashFlows().subscribe({
      next: (data) => {
        this.cashFlows.set(data);
        this.loadRecurringTransactions();
      },
      error: (err) => console.error('Error fetching cash flow registry:', err),
    });
  }

  // Bank Account Handlers
  openAccModal(acc?: BankAccount): void {
    if (acc) {
      this.accFormTitle = 'Edit Bank Account';
      this.accForm = { ...acc };
    } else {
      this.accFormTitle = 'Add Bank Account';
      this.accForm = {
        name: '',
        currency: 'USD',
        balance: 0,
      };
    }
    this.isAccModalOpen = true;
  }

  closeAccModal(): void {
    this.isAccModalOpen = false;
  }

  saveAccount(): void {
    if (!this.accForm.name) {
      alert('Please enter a bank name.');
      return;
    }

    if (this.accForm.id) {
      this.financeService.updateBankAccount(this.accForm.id, this.accForm).subscribe({
        next: () => {
          this.closeAccModal();
          this.loadAccounts();
        },
        error: (err) => alert('Failed to update bank account: ' + err.error),
      });
    } else {
      this.financeService.createBankAccount(this.accForm).subscribe({
        next: () => {
          this.closeAccModal();
          this.loadAccounts();
        },
        error: (err) => alert('Failed to create bank account: ' + err.error),
      });
    }
  }

  deleteAccount(id: string): void {
    if (confirm('Are you sure you want to delete this bank account?')) {
      this.financeService.deleteBankAccount(id).subscribe({
        next: () => this.loadAccounts(),
        error: (err) => alert('Failed to delete bank account: ' + err.error),
      });
    }
  }

  onCategoryChange(): void {
    const inflowCats = [
      'Salary',
      'Bonus',
      'Interest Income',
      'Investment Distributions',
      'Cashback Rewards',
      'Other Income',
    ];
    if (inflowCats.includes(this.flowForm.category)) {
      this.flowForm.type = 'inflow';
    } else {
      this.flowForm.type = 'outflow';
    }
  }

  openModal(flow?: CashFlow): void {
    this.flowIsRecurring = false;
    if (flow) {
      this.formTitle = 'Edit Transaction';
      this.flowForm = { ...flow };
    } else {
      this.formTitle = 'Log Transaction';
      this.flowForm = {
        date: new Date().toISOString().substring(0, 10),
        description: '',
        category: 'Daily Living',
        amount: 0,
        currency: 'USD',
        type: 'outflow',
      };
    }
    this.isModalOpen = true;
  }

  closeModal(): void {
    this.isModalOpen = false;
  }

  saveFlow(): void {
    if (!this.flowForm.description || this.flowForm.amount <= 0 || !this.flowForm.date) {
      alert('Please fill in all required fields and ensure amount is greater than 0.');
      return;
    }

    // Double check type
    this.onCategoryChange();

    if (this.flowForm.id) {
      // Update
      this.financeService.updateCashFlow(this.flowForm.id, this.flowForm).subscribe({
        next: () => {
          this.closeModal();
          this.loadData();
        },
        error: (err) => alert('Failed to update transaction: ' + err.error),
      });
    } else {
      // Create
      this.financeService.createCashFlow(this.flowForm).subscribe({
        next: () => {
          if (this.flowIsRecurring) {
            this.financeService
              .createRecurringTransaction({
                description: this.flowForm.description,
                category: this.flowForm.category,
                amount: this.flowForm.amount,
                currency: this.flowForm.currency,
                type: this.flowForm.type,
                next_run_date: addOneMonthClamped(this.flowForm.date),
              })
              .subscribe({
                next: () => {
                  this.closeModal();
                  this.loadData();
                },
                error: (err) =>
                  alert(
                    'Transaction logged, but failed to schedule the automatic repeat: ' + err.error,
                  ),
              });
          } else {
            this.closeModal();
            this.loadData();
          }
        },
        error: (err) => alert('Failed to create transaction: ' + err.error),
      });
    }
  }

  deleteFlow(id: string): void {
    if (confirm('Are you sure you want to delete this cash flow record?')) {
      this.financeService.deleteCashFlow(id).subscribe({
        next: () => this.loadData(),
        error: (err) => alert('Failed to delete transaction: ' + err.error),
      });
    }
  }

  // Recurring Transaction Handlers
  openRecurringModal(rt: RecurringTransaction): void {
    this.recurringFormTitle = 'Edit Automatic Transaction';
    this.recurringForm = { ...rt };
    this.isRecurringModalOpen = true;
  }

  closeRecurringModal(): void {
    this.isRecurringModalOpen = false;
  }

  saveRecurringTransaction(): void {
    if (!this.recurringForm.id) {
      return;
    }
    if (
      !this.recurringForm.description ||
      this.recurringForm.amount <= 0 ||
      !this.recurringForm.next_run_date
    ) {
      alert('Please fill in all required fields and ensure amount is greater than 0.');
      return;
    }

    const inflowCats = [
      'Salary',
      'Bonus',
      'Interest Income',
      'Investment Distributions',
      'Cashback Rewards',
      'Other Income',
    ];
    this.recurringForm.type = inflowCats.includes(this.recurringForm.category)
      ? 'inflow'
      : 'outflow';

    this.financeService
      .updateRecurringTransaction(this.recurringForm.id, this.recurringForm)
      .subscribe({
        next: () => {
          this.closeRecurringModal();
          this.loadRecurringTransactions();
        },
        error: (err) => alert('Failed to update automatic transaction: ' + err.error),
      });
  }

  cancelRecurringTransaction(id: string): void {
    if (confirm('Cancel this automatic transaction? It will no longer be logged going forward.')) {
      this.financeService.deleteRecurringTransaction(id).subscribe({
        next: () => this.loadRecurringTransactions(),
        error: (err) => alert('Failed to cancel automatic transaction: ' + err.error),
      });
    }
  }
}
