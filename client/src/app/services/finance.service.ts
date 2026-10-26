import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

export interface BankAccount {
  id?: string;
  name: string;
  currency: 'USD' | 'TWD';
  balance: number;
}

export interface Investment {
  id?: string;
  ticker: string;
  name: string;
  type: 'stock' | 'etf' | 'fund';
  shares: number;
  cost_basis: number;
  currency: 'USD' | 'TWD';
}

export interface CashFlow {
  id?: string;
  date: string;
  description: string;
  category: string;
  amount: number;
  currency: string;
  type: 'inflow' | 'outflow';
}

export interface RecurringTransaction {
  id?: string;
  description: string;
  category: string;
  amount: number;
  currency: string;
  type: 'inflow' | 'outflow';
  next_run_date: string;
}

export interface HistoricalAsset {
  month: string;
  bank_usd: number;
  bank_twd: number;
  market_value_usd: number;
  market_value_twd: number;
  cost_basis_usd: number;
  cost_basis_twd: number;
  usd_twd_rate: number;
}

export interface FxRate {
  pair: string;
  rate: number;
  updated_at?: string;
}

export interface TickerPrice {
  ticker: string;
  price: number;
  currency: string;
  updated_at?: string;
}

export interface ValuationData {
  rates: FxRate[];
  prices: TickerPrice[];
}

export interface AggregatedChartData {
  asset_growth: {
    month: string;
    total: number;
    bank_total: number;
    market_value_total: number;
    cost_basis_total: number;
  }[];
  income_expense: {
    month: string;
    inflow: number;
    outflow: number;
  }[];
  passive_income: {
    month: string;
    total: number;
  }[];
}

@Injectable({
  providedIn: 'root',
})
export class FinanceService {
  private http = inject(HttpClient);
  private apiUrl = 'http://127.0.0.1:3000/api';

  // Bank Accounts CRUD
  getBankAccounts(): Observable<BankAccount[]> {
    return this.http.get<BankAccount[]>(`${this.apiUrl}/bank-accounts`);
  }

  createBankAccount(account: BankAccount): Observable<BankAccount> {
    return this.http.post<BankAccount>(`${this.apiUrl}/bank-accounts`, account);
  }

  updateBankAccount(id: string, account: BankAccount): Observable<void> {
    return this.http.put<void>(`${this.apiUrl}/bank-accounts/${id}`, account);
  }

  deleteBankAccount(id: string): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/bank-accounts/${id}`);
  }

  // Investments CRUD
  getInvestments(): Observable<Investment[]> {
    return this.http.get<Investment[]>(`${this.apiUrl}/investments`);
  }

  createInvestment(investment: Investment): Observable<Investment> {
    return this.http.post<Investment>(`${this.apiUrl}/investments`, investment);
  }

  updateInvestment(id: string, investment: Investment): Observable<void> {
    return this.http.put<void>(`${this.apiUrl}/investments/${id}`, investment);
  }

  deleteInvestment(id: string): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/investments/${id}`);
  }

  // Cash Flow CRUD
  getCashFlows(): Observable<CashFlow[]> {
    return this.http.get<CashFlow[]>(`${this.apiUrl}/cash-flow`);
  }

  createCashFlow(flow: CashFlow): Observable<CashFlow> {
    return this.http.post<CashFlow>(`${this.apiUrl}/cash-flow`, flow);
  }

  updateCashFlow(id: string, flow: CashFlow): Observable<void> {
    return this.http.put<void>(`${this.apiUrl}/cash-flow/${id}`, flow);
  }

  deleteCashFlow(id: string): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/cash-flow/${id}`);
  }

  // Recurring Transactions CRUD
  getRecurringTransactions(): Observable<RecurringTransaction[]> {
    return this.http.get<RecurringTransaction[]>(`${this.apiUrl}/recurring-transactions`);
  }

  createRecurringTransaction(transaction: RecurringTransaction): Observable<RecurringTransaction> {
    return this.http.post<RecurringTransaction>(
      `${this.apiUrl}/recurring-transactions`,
      transaction,
    );
  }

  updateRecurringTransaction(id: string, transaction: RecurringTransaction): Observable<void> {
    return this.http.put<void>(`${this.apiUrl}/recurring-transactions/${id}`, transaction);
  }

  deleteRecurringTransaction(id: string): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/recurring-transactions/${id}`);
  }

  // Historical Assets CRUD
  getHistoricalAssets(): Observable<HistoricalAsset[]> {
    return this.http.get<HistoricalAsset[]>(`${this.apiUrl}/historical-assets`);
  }

  saveHistoricalAsset(asset: HistoricalAsset): Observable<HistoricalAsset> {
    return this.http.post<HistoricalAsset>(`${this.apiUrl}/historical-assets`, asset);
  }

  deleteHistoricalAsset(month: string): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/historical-assets/${month}`);
  }

  // Valuation
  getValuation(): Observable<ValuationData> {
    return this.http.get<ValuationData>(`${this.apiUrl}/valuation`);
  }

  // Chart aggregation pipeline
  getAggregatedCharts(): Observable<AggregatedChartData> {
    return this.http.get<AggregatedChartData>(`${this.apiUrl}/charts/aggregated`);
  }
}
