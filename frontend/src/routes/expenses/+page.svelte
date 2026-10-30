<script lang="ts">
  import { authClient, isAdmin } from '$lib/auth-client';
  import AppHeader from '$lib/components/app-header.svelte';
  import NoAccess from '$lib/components/auth/no-access.svelte';
  import SignInCard from '$lib/components/auth/sign-in-card.svelte';
  import CreateExpenseDialog from '$lib/components/expenses/create-expense-dialog.svelte';
  import EditExpenseDialog from '$lib/components/expenses/edit-expense-dialog.svelte';
  import ExpenseSourceCard from '$lib/components/expenses/expense-source-card.svelte';
  import MonthPicker from '$lib/components/expenses/month-picker.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import {
    Empty,
    EmptyDescription,
    EmptyHeader,
    EmptyMedia,
    EmptyTitle,
  } from '$lib/components/ui/empty';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import {
    currentMonth,
    EXPENSE_SOURCES_QUERY_KEY,
    expensesQueryKey,
    formatInr,
    formatUsd,
    listExpenses,
    listSources,
    totalsBySource,
    type ExpenseDto,
  } from '$lib/expenses-api';
  import { queryClient } from '$lib/query-client';
  import Plus from '@lucide/svelte/icons/plus';
  import ReceiptText from '@lucide/svelte/icons/receipt-text';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import Wallet from '@lucide/svelte/icons/wallet';
  import { createQuery } from '@tanstack/svelte-query';

  // Better Auth session store. UX gate only — the Rust API re-checks the
  // session cookie on every request.
  const session = authClient.useSession();
  const user = $derived($session.data?.user ?? null);

  let month = $state(currentMonth());
  let editing = $state<ExpenseDto | null>(null);
  let creating = $state(false);

  const sourcesQuery = createQuery(
    () => ({
      queryKey: EXPENSE_SOURCES_QUERY_KEY,
      queryFn: listSources,
      staleTime: 5 * 60_000,
    }),
    () => queryClient,
  );

  const expensesQuery = createQuery(
    () => ({
      queryKey: expensesQueryKey(month),
      queryFn: () => listExpenses(month),
      staleTime: 30_000,
    }),
    () => queryClient,
  );

  const sources = $derived(sourcesQuery.data ?? []);
  const totals = $derived(totalsBySource(sources, expensesQuery.data ?? []));
  const monthTotalInr = $derived(totals.reduce((sum, t) => sum + t.totalInr, 0));
  const monthTotalUsd = $derived.by(() => {
    let sum = 0;
    let has = false;
    for (const t of totals) {
      if (t.totalUsd != null) {
        sum += t.totalUsd;
        has = true;
      }
    }
    return has ? sum : null;
  });
  const loading = $derived(sourcesQuery.isPending || expensesQuery.isPending);
  const loadError = $derived(sourcesQuery.error ?? expensesQuery.error);

  function refresh() {
    void sourcesQuery.refetch();
    void expensesQuery.refetch();
  }
</script>

<div class="flex min-h-svh flex-col">
  <AppHeader {user} />
  <main class="mx-auto w-full max-w-6xl flex-1 px-4 py-8 sm:px-6 sm:py-12 lg:px-8">
    {#if $session.isPending && !$session.data}
      <div class="grid gap-6 md:grid-cols-2" aria-busy="true">
        <Skeleton class="h-44 w-full rounded-2xl" />
        <Skeleton class="h-44 w-full rounded-2xl" />
        <Skeleton class="h-72 w-full rounded-2xl" />
        <Skeleton class="h-72 w-full rounded-2xl" />
      </div>
    {:else if !user}
      <div class="mx-auto max-w-md"><SignInCard /></div>
    {:else if !isAdmin(user)}
      <div class="mx-auto max-w-md"><NoAccess {user} /></div>
    {:else}
      <div class="space-y-6">
        <section class="rise space-y-4">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <h2 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
              Expenses
              {#if expensesQuery.isSuccess}
                <Badge variant="secondary">{expensesQuery.data.length}</Badge>
              {/if}
            </h2>
            <div class="flex flex-wrap items-center gap-1.5">
              <MonthPicker {month} onChange={(m) => (month = m)} />
              <Button
                variant="ghost"
                size="icon"
                aria-label="Refresh"
                disabled={expensesQuery.isFetching}
                onclick={refresh}
              >
                <RefreshCw
                  class={expensesQuery.isFetching ? 'animate-spin motion-reduce:animate-none' : ''}
                />
              </Button>
              <Button onclick={() => (creating = true)}><Plus /> Add expense</Button>
            </div>
          </div>

          {#if monthTotalInr > 0 || (monthTotalUsd ?? 0) > 0}
            <div class="flex items-center gap-3 rounded-2xl border bg-card px-4 py-3 sm:px-5">
              <span class="grid size-9 shrink-0 place-items-center rounded-xl bg-primary/10">
                <Wallet class="size-4.5 text-primary" />
              </span>
              <div class="min-w-0 flex-1">
                <p class="text-xs text-muted-foreground">Month total</p>
                <p class="truncate text-xl font-bold tabular-nums sm:text-2xl">
                  {formatInr(monthTotalInr)}
                </p>
              </div>
              {#if monthTotalUsd != null}
                <p class="shrink-0 text-sm text-muted-foreground tabular-nums">
                  {formatUsd(monthTotalUsd)}
                </p>
              {/if}
            </div>
          {/if}

          {#if loading}
            <div class="grid gap-3 md:grid-cols-2" aria-busy="true">
              {#each [0, 1, 2, 3] as i (i)}
                <Skeleton class="h-20 w-full rounded-2xl" />
              {/each}
            </div>
          {:else if loadError}
            <Empty class="border py-12">
              <EmptyHeader>
                <EmptyMedia variant="icon"><TriangleAlert /></EmptyMedia>
                <EmptyTitle>Couldn't load expenses</EmptyTitle>
                <EmptyDescription>{loadError.message}</EmptyDescription>
              </EmptyHeader>
              <Button variant="outline" onclick={refresh}>Try again</Button>
            </Empty>
          {:else if sources.length === 0}
            <Empty class="border py-12">
              <EmptyHeader>
                <EmptyMedia variant="icon"><ReceiptText /></EmptyMedia>
                <EmptyTitle>No sources yet</EmptyTitle>
                <EmptyDescription>Add your first expense to get started.</EmptyDescription>
              </EmptyHeader>
              <Button onclick={() => (creating = true)}><Plus /> Add expense</Button>
            </Empty>
          {:else}
            <div class="grid items-start gap-3 md:grid-cols-2">
              {#each totals as total (total.source.key)}
                <ExpenseSourceCard {total} {month} onEdit={(entry) => (editing = entry)} />
              {/each}
            </div>
          {/if}
        </section>
      </div>
    {/if}
  </main>
</div>

<CreateExpenseDialog {month} open={creating} onClose={() => (creating = false)} />
<EditExpenseDialog entry={editing} {month} onClose={() => (editing = null)} />
