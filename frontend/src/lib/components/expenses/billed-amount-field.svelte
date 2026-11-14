<script lang="ts">
  import * as InputGroup from '$lib/components/ui/input-group';
  import { Label } from '$lib/components/ui/label';
  import {
    BILLED_CURRENCIES,
    BILLED_CURRENCY_SYMBOL,
    type BilledCurrency,
  } from '$lib/expenses-api';
  import { cn } from '$lib/utils';

  let {
    id,
    value = $bindable(''),
    currency = $bindable<BilledCurrency>('USD'),
    invalid = false,
    placeholder = '20.00',
  }: {
    id: string;
    value?: string;
    currency?: BilledCurrency;
    invalid?: boolean;
    placeholder?: string;
  } = $props();
</script>

<div class="space-y-1.5">
  <Label for={id}>
    Billed ({BILLED_CURRENCY_SYMBOL[currency]})
    <span class="font-normal text-muted-foreground">(optional)</span>
  </Label>
  <InputGroup.Root>
    <InputGroup.Input
      {id}
      inputmode="decimal"
      autocomplete="off"
      {placeholder}
      bind:value
      aria-invalid={invalid ? true : undefined}
    />
    <InputGroup.Addon align="inline-end" role="radiogroup" aria-label="Billed currency">
      {#each BILLED_CURRENCIES as code (code)}
        <InputGroup.Button
          size="icon-xs"
          role="radio"
          aria-checked={currency === code}
          aria-label={code}
          title={code}
          variant={currency === code ? 'secondary' : 'ghost'}
          class={cn(currency !== code && 'text-muted-foreground')}
          onclick={() => (currency = code)}
        >
          {BILLED_CURRENCY_SYMBOL[code]}
        </InputGroup.Button>
      {/each}
    </InputGroup.Addon>
  </InputGroup.Root>
</div>
