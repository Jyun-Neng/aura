import { Routes } from '@angular/router';
import { DashboardComponent } from './components/dashboard/dashboard';
import { PortfolioComponent } from './components/portfolio/portfolio';
import { CashflowComponent } from './components/cashflow/cashflow';
import { ValuationComponent } from './components/valuation/valuation';
import { ChartsComponent } from './components/charts/charts';

export const routes: Routes = [
  { path: '', redirectTo: 'dashboard', pathMatch: 'full' },
  { path: 'dashboard', component: DashboardComponent },
  { path: 'portfolio', component: PortfolioComponent },
  { path: 'cashflow', component: CashflowComponent },
  { path: 'valuation', component: ValuationComponent },
  { path: 'charts', component: ChartsComponent },
  { path: '**', redirectTo: 'dashboard' },
];
