import { Pipe, PipeTransform } from '@angular/core';

// USD amounts are prefixed with '$', everything else (TWD) with 'NT$'.
@Pipe({
  name: 'currencySymbol',
  standalone: true,
})
export class CurrencySymbolPipe implements PipeTransform {
  transform(currency: string): string {
    return currency === 'USD' ? '$' : 'NT$';
  }
}
