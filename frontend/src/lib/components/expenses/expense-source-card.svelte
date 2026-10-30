<script lang="ts">
  import SourceBrandIcon from '$lib/components/expenses/source-brand-icon.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent } from '$lib/components/ui/card';
  import {
    deleteExpense,
    deleteInvoice,
    expensesQueryKey,
    formatDay,
    formatInr,
    formatUsd,
    invoiceObjectUrl,
    type ExpenseDto,
    type SourceMonthTotal,
  } from '$lib/expenses-api';
  import { queryClient } from '$lib/query-client';
  import EllipsisVertical from '@lucide/svelte/icons/ellipsis-vertical';
  import ExternalLink from '@lucide/svelte/icons/external-link';
  import Eye from '@lucide/svelte/icons/eye';
  import FileDown from '@lucide/svelte/icons/file-down';
  import FileText from '@lucide/svelte/icons/file-text';
  import Pencil from '@lucide/svelte/icons/pencil';
  import Receipt from '@lucide/svelte/icons/receipt';
  import Trash2 from '@lucide/svelte/icons/trash-2';
  import { createMutation } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  let {
    total,
    month,
    onEdit,
    onView,
  }: {
    total: SourceMonthTotal;
    month: string;
    onEdit: (entry: ExpenseDto) => void;
    onView: (entry: ExpenseDto) => void;
  } = $props();

  let expanded = $state(false);
  let confirmingId = $state<number | null>(null);
  let downloadingId = $state<number | null>(null);

  const mutation = createMutation(
    () => ({
      mutationFn: async (input: { id: number; action: 'delete' | 'unlink' }) => {
        if (input.action === 'delete') await deleteExpense(input.id);
        else await deleteInvoice(input.id);
      },
      onSuccess: async (_, input) => {
        await queryClient.invalidateQueries({ queryKey: expensesQueryKey(month) });
        confirmingId = null;
        toast.success(input.action === 'delete' ? 'Expense deleted' : 'Invoice removed');
      },
      onError: (e: Error) => toast.error('Action failed', { description: e.message }),
    }),
    () => queryClient,
  );

  async function download(entry: ExpenseDto) {
    if (downloadingId) return;
    downloadingId = entry.id;
    try {
      const { url, filename } = await invoiceObjectUrl(entry.id);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      a.remove();
      setTimeout(() => URL.revokeObjectURL(url), 30_000);
    } catch (e) {
      toast.error('Could not download invoice', {
        description: e instanceof Error ? e.message : undefined,
      });
    } finally {
      downloadingId = null;
    }
  }
</script>

<Card>
  <CardContent class="px-4 py-4 sm:px-5">
    <div class="flex items-center gap-3">
      <SourceBrandIcon sourceKey={total.source.key}>
        <Receipt class="size-5 text-muted-foreground" />
      </SourceBrandIcon>
      <div class="min-w-0 flex-1">
        <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
          <h3 class="truncate text-sm font-semibold sm:text-base">{total.source.name}</h3>
          {#each total.labels as label (label)}
            <Badge variant="secondary" class="shrink-0">{label}</Badge>
          {/each}
        </div>
        {#if total.source.description}
          <p class="truncate text-xs text-muted-foreground">{total.source.description}</p>
        {/if}
      </div>
      <div class="shrink-0 text-right">
        <p class="text-sm font-bold tabular-nums sm:text-lg">{formatInr(total.totalInr)}</p>
        {#if total.totalUsd != null}
          <p class="text-xs text-muted-foreground tabular-nums">{formatUsd(total.totalUsd)}</p>
        {/if}
      </div>
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label={expanded ? 'Hide details' : 'Show details'}
        aria-expanded={expanded}
        onclick={() => (expanded = !expanded)}
      >
        <EllipsisVertical />
      </Button>
    </div>

    {#if expanded}
      {#if total.entries.length === 0}
        <p class="mt-3 rounded-lg bg-muted/50 px-3 py-2 text-center text-xs text-muted-foreground">
          No charges this month.
        </p>
      {:else}
        <ul class="mt-3 divide-y rounded-xl border">
          {#each total.entries as entry (entry.id)}
            <li class="flex items-center gap-2 px-3 py-2.5 text-sm">
              <div class="min-w-0 flex-1">
                <div class="flex flex-wrap items-center gap-x-2">
                  <span class="font-mono text-xs text-muted-foreground"
                    >{formatDay(entry.date)}</span
                  >
                  {#if entry.label}
                    <Badge variant="outline" class="text-[11px]">{entry.label}</Badge>
                  {/if}
                </div>
                <p class="mt-0.5 tabular-nums">
                  <span class="font-semibold">{formatInr(entry.inr_paise)}</span>
                  {#if entry.usd_cents != null}
                    <span class="ml-1.5 text-xs text-muted-foreground">
                      {formatUsd(entry.usd_cents)}
                    </span>
                  {/if}
                </p>
                {#if entry.note}
                  <p class="mt-0.5 line-clamp-2 text-xs text-muted-foreground">{entry.note}</p>
                {/if}
              </div>
              <div class="flex shrink-0 items-center gap-0.5">
                {#if entry.invoice_link}
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    aria-label="Open invoice link"
                    href={entry.invoice_link}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <ExternalLink />
                  </Button>
                {/if}
                {#if entry.has_invoice}
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    aria-label={entry.invoice_name ? `View ${entry.invoice_name}` : 'View invoice'}
                    title="View invoice"
                    onclick={() => onView(entry)}
                  >
                    <Eye />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    aria-label={entry.invoice_name ?? 'Download invoice'}
                    title={entry.invoice_name ?? 'Download invoice'}
                    disabled={downloadingId === entry.id}
                    onclick={() => download(entry)}
                  >
                    {#if entry.invoice_name?.toLowerCase().endsWith('.pdf')}
                      <FileText />
                    {:else}
                      <FileDown />
                    {/if}
                  </Button>
                {/if}
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label="Edit expense"
                  onclick={() => onEdit(entry)}
                >
                  <Pencil />
                </Button>
                {#if confirmingId === entry.id}
                  <Button
                    variant="destructive"
                    size="sm"
                    class="h-7 px-2 text-xs"
                    disabled={mutation.isPending}
                    onclick={() => mutation.mutate({ id: entry.id, action: 'delete' })}
                  >
                    Confirm?
                  </Button>
                {:else}
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    class="hover:bg-destructive/10 hover:text-destructive"
                    aria-label="Delete expense"
                    onclick={() => {
                      confirmingId = entry.id;
                      setTimeout(() => {
                        if (confirmingId === entry.id) confirmingId = null;
                      }, 4000);
                    }}
                  >
                    <Trash2 />
                  </Button>
                {/if}
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </CardContent>
</Card>
