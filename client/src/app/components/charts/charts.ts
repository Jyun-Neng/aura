import { Component, OnInit, OnDestroy, ElementRef, ViewChild, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FinanceService, AggregatedChartData } from '../../services/finance.service';
import { Chart, ChartOptions } from 'chart.js/auto';

// Catppuccin Mocha Color Palette Constants
const MAUVE = '#cba6f7';
const GREEN = '#a6e3a1';
const TEAL = '#94e2d5';
const PEACH = '#fab387';
const RED = '#f38ba8';
const TEXT = '#cdd6f4';
const SURFACE2 = '#585b70';
const GRID_COLOR = 'rgba(88, 91, 112, 0.2)'; // Catppuccin Surface2 with transparency

type AssetGrowthSeries = 'total' | 'bank_total' | 'market_value_total';

const ASSET_GROWTH_SERIES_CONFIG: Record<
  AssetGrowthSeries,
  { label: string; color: string; fillColor: string }
> = {
  total: { label: 'Total Net Worth', color: MAUVE, fillColor: 'rgba(203, 166, 247, 0.12)' },
  bank_total: { label: 'Total Bank Balance', color: GREEN, fillColor: 'rgba(166, 227, 161, 0.12)' },
  market_value_total: {
    label: 'Total Market Value',
    color: PEACH,
    fillColor: 'rgba(250, 179, 135, 0.12)',
  },
};

@Component({
  selector: 'app-charts',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './charts.html',
  styleUrls: [],
})
export class ChartsComponent implements OnInit, OnDestroy {
  private financeService = inject(FinanceService);

  // View Children
  @ViewChild('assetGrowthCanvas') assetGrowthCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('cashFlowCanvas') cashFlowCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('passiveIncomeCanvas') passiveIncomeCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('gainLossCanvas') gainLossCanvas!: ElementRef<HTMLCanvasElement>;

  // Chart instances
  private assetGrowthChart: Chart | null = null;
  private cashFlowChart: Chart | null = null;
  private passiveIncomeChart: Chart | null = null;
  private gainLossChart: Chart | null = null;

  // Data State
  public apiData = signal<AggregatedChartData | null>(null);

  // Which single Asset Growth series is currently displayed (each has a very
  // different scale, so showing them together flattens the smaller ones).
  public assetGrowthSeries = signal<AssetGrowthSeries>('total');
  public assetGrowthSeriesOptions: { value: AssetGrowthSeries; label: string }[] = [
    { value: 'total', label: 'Total Net Worth' },
    { value: 'bank_total', label: 'Total Bank Balance' },
    { value: 'market_value_total', label: 'Total Market Value' },
  ];

  setAssetGrowthSeries(series: AssetGrowthSeries): void {
    this.assetGrowthSeries.set(series);
    this.renderAssetGrowthChart();
  }

  ngOnInit(): void {
    this.loadChartData();
  }

  ngOnDestroy(): void {
    this.destroyCharts();
  }

  loadChartData(): void {
    this.financeService.getAggregatedCharts().subscribe({
      next: (data) => {
        this.apiData.set(data);
        // Wait for next tick so canvas references are active
        setTimeout(() => this.renderCharts(), 0);
      },
      error: (err) => console.error('Error loading aggregated charts data:', err),
    });
  }

  destroyCharts(): void {
    this.assetGrowthChart?.destroy();
    this.assetGrowthChart = null;
    this.cashFlowChart?.destroy();
    this.cashFlowChart = null;
    this.passiveIncomeChart?.destroy();
    this.passiveIncomeChart = null;
    this.gainLossChart?.destroy();
    this.gainLossChart = null;
  }

  private commonOptions(): ChartOptions {
    return {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: {
          labels: {
            color: TEXT,
            font: { family: 'Outfit', size: 12 },
          },
        },
        tooltip: {
          backgroundColor: '#1e1e2e',
          titleColor: MAUVE,
          bodyColor: TEXT,
          borderColor: SURFACE2,
          borderWidth: 1,
          titleFont: { family: 'Outfit', weight: 'bold' as const },
          bodyFont: { family: 'Outfit' },
        },
      },
      scales: {
        x: {
          grid: { color: GRID_COLOR },
          ticks: { color: TEXT, font: { family: 'Outfit' } },
        },
        y: {
          grid: { color: GRID_COLOR },
          ticks: { color: TEXT, font: { family: 'Outfit' } },
        },
      },
    };
  }

  renderCharts(): void {
    if (!this.apiData()) return;

    this.destroyCharts();
    this.renderAssetGrowthChart();
    this.renderCashFlowChart();
    this.renderPassiveIncomeChart();
    this.renderGainLossChart();
  }

  // 1. Asset Growth Trend — one series at a time (Total Net Worth, Total Bank
  // Balance, or Total Market Value), each in TWD. Shown one at a time since
  // Total Bank Balance is dwarfed by the other two on a shared axis.
  private renderAssetGrowthChart(): void {
    const data = this.apiData();
    if (!data || !this.assetGrowthCanvas) return;

    this.assetGrowthChart?.destroy();

    const series = this.assetGrowthSeries();
    const config = ASSET_GROWTH_SERIES_CONFIG[series];
    const labels = data.asset_growth.map((point) => point.month);

    this.assetGrowthChart = new Chart(this.assetGrowthCanvas.nativeElement, {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: config.label,
            data: data.asset_growth.map((point) => point[series]),
            borderColor: config.color,
            backgroundColor: config.fillColor,
            fill: true,
            tension: 0.2,
            borderWidth: 3,
            pointHoverRadius: 6,
          },
        ],
      },
      options: this.commonOptions(),
    });
  }

  // 2. Cash Flow Timeline
  private renderCashFlowChart(): void {
    const data = this.apiData();
    if (!data || !this.cashFlowCanvas) return;

    const labels = data.income_expense.map((point) => point.month);

    this.cashFlowChart = new Chart(this.cashFlowCanvas.nativeElement, {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: 'Income',
            data: data.income_expense.map((point) => point.inflow),
            borderColor: GREEN,
            backgroundColor: 'rgba(166, 227, 161, 0.15)',
            tension: 0.2,
            fill: true,
            borderWidth: 2.5,
          },
          {
            label: 'Expenses',
            data: data.income_expense.map((point) => point.outflow),
            borderColor: PEACH,
            backgroundColor: 'rgba(250, 179, 135, 0.15)',
            tension: 0.2,
            fill: true,
            borderWidth: 2.5,
          },
        ],
      },
      options: this.commonOptions(),
    });
  }

  // 3. Passive Income Trend (Bank Interest + Investment Distributions, combined)
  private renderPassiveIncomeChart(): void {
    const data = this.apiData();
    if (!data || !this.passiveIncomeCanvas) return;

    const labels = data.passive_income.map((point) => point.month);

    this.passiveIncomeChart = new Chart(this.passiveIncomeCanvas.nativeElement, {
      type: 'bar',
      data: {
        labels,
        datasets: [
          {
            label: 'Passive Income',
            data: data.passive_income.map((point) => point.total),
            backgroundColor: TEAL,
            borderRadius: 4,
          },
        ],
      },
      options: this.commonOptions(),
    });
  }

  // 4. Unrealized Gain/Loss — Total Market Value minus Total Cost Basis
  private renderGainLossChart(): void {
    const data = this.apiData();
    if (!data || !this.gainLossCanvas) return;

    const labels = data.asset_growth.map((point) => point.month);
    const gainLoss = data.asset_growth.map(
      (point) => point.market_value_total - point.cost_basis_total,
    );

    this.gainLossChart = new Chart(this.gainLossCanvas.nativeElement, {
      type: 'bar',
      data: {
        labels,
        datasets: [
          {
            label: 'Unrealized Gain/Loss',
            data: gainLoss,
            backgroundColor: gainLoss.map((value) => (value >= 0 ? GREEN : RED)),
            borderRadius: 4,
          },
        ],
      },
      options: this.commonOptions(),
    });
  }
}
