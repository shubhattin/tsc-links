<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { authClient, isAdmin } from '$lib/auth-client';
  import AppHeader from '$lib/components/app-header.svelte';
  import NoAccess from '$lib/components/auth/no-access.svelte';
  import SignInCard from '$lib/components/auth/sign-in-card.svelte';
  import CreateExpenseDialog from '$lib/components/expenses/create-expense-dialog.svelte';
  import CreateSourceDialog from '$lib/components/expenses/create-source-dialog.svelte';
  import EditExpenseDialog from '$lib/components/expenses/edit-expense-dialog.svelte';
  import ExpenseSourceCard from '$lib/components/expenses/expense-source-card.svelte';
  import InvoiceViewerDialog from '$lib/components/expenses/invoice-viewer-dialog.svelte';
  import ManageSourcesDialog from '$lib/components/expenses/manage-sources-dialog.svelte';
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
    monthLabel,
    MONTH_PARAM,
    normalizeMonthParam,
    totalsBySource,
    type ExpenseDto,
    type SourceMonthTotal,
  } from '$lib/expenses-api';
  import { queryClient } from '$lib/query-client';
  import Archive from '@lucide/svelte/icons/archive';
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import Layers from '@lucide/svelte/icons/layers';
  import Plus from '@lucide/svelte/icons/plus';
  import ReceiptText from '@lucide/svelte/icons/receipt-text';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import Wallet from '@lucide/svelte/icons/wallet';
  import { createQuery } from '@tanstack/svelte-query';
  import { untrack } from 'svelte';

  // Better Auth session store. UX gate only — the Rust API re-checks the
  // session cookie on every request.
  const session = authClient.useSession();
  const user = $derived($session.data?.user ?? null);

  // Month filter synced with `?month=YYYY-MM`:
  // - no param (or an invalid one) → current month, canonical URL has no param
  // - `?month=<current month>` → stripped to the canonical param-less URL
  // - any other valid month → kept in the URL so it stays shareable
  // `month` is local state so stepping feels instant; `selectMonth` pushes the
  // matching URL, and the effect below only adopts *external* URL changes
  // (back/forward navigation, pasted links) — it deliberately does not track
  // `month`, otherwise it would revert a step before the `goto` lands.
  let month = $state(normalizeMonthParam(page.url.searchParams.get(MONTH_PARAM)) ?? currentMonth());

  function selectMonth(next: string) {
    if (next === month) return;
    month = next;
    const target = next === currentMonth() ? '/expenses' : `/expenses?${MONTH_PARAM}=${next}`;
    void goto(target, { replaceState: true, keepFocus: true, noScroll: true });
  }

  // URL → state: adopt external URL changes and canonicalize the address bar
  // (invalid params, redundant current-month params, `MM-YYYY` shape).
  $effect(() => {
    const raw = page.url.searchParams.get(MONTH_PARAM);
    const today = currentMonth();
    const normalized = normalizeMonthParam(raw);
    if ((normalized ?? today) !== untrack(() => month)) month = normalized ?? today;
    if (raw != null && raw !== '') {
      if (normalized == null || normalized === today) {
        void goto('/expenses', { replaceState: true, keepFocus: true, noScroll: true });
      } else if (raw !== normalized) {
        void goto(`/expenses?${MONTH_PARAM}=${normalized}`, {
          replaceState: true,
          keepFocus: true,
          noScroll: true,
        });
      }
    }
  });

  let editing = $state<ExpenseDto | null>(null);
  let viewing = $state<ExpenseDto | null>(null);
  let creating = $state(false);
  /** Source pre-selected in the Add bill dialog (from a source card). */
  let creatingSourceKey = $state('');
  let creatingSource = $state(false);
  let managing = $state(false);
  let showInactive = $state(false);
  /**
   * - `billed`: sources with a charge this month
   * - `unbilled`: every source that is *not* in `billed` (zero-charge entries, or nothing at all)
   * - `all`: every source
   */
  let billedFilter = $state<'all' | 'billed' | 'unbilled'>('billed');

  function addBill(sourceKey = '') {
    creatingSourceKey = sourceKey;
    creating = true;
  }

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
  const entries = $derived(expensesQuery.data ?? []);
  const billedEntries = $derived(entries.filter((e) => e.inr_paise !== 0));
  const billedKeys = $derived(new Set(billedEntries.map((e) => e.source_key)));
  /** Whether anything at all was billed this month (drives the "add a bill" prompts). */
  const monthHasBills = $derived(billedEntries.length > 0);

  /** Cards for the current tab. `inactive` is the accordion (never used on Billed). */
  const view = $derived.by((): { main: SourceMonthTotal[]; inactive: SourceMonthTotal[] } => {
    if (billedFilter === 'billed') {
      return {
        main: totalsBySource(sources, billedEntries).filter((t) => t.entries.length > 0),
        inactive: [],
      };
    }
    const totals =
      billedFilter === 'all'
        ? totalsBySource(sources, entries)
        : totalsBySource(
            sources.filter((s) => !billedKeys.has(s.key)),
            entries.filter((e) => !billedKeys.has(e.source_key)),
          );
    // Sources with entries first, the rest keep their alphabetical order.
    const main = totals
      .filter((t) => t.source.active)
      .sort((a, b) => Number(b.entries.length > 0) - Number(a.entries.length > 0));
    return { main, inactive: totals.filter((t) => !t.source.active) };
  });
  const shownSources = $derived(view.main.length + view.inactive.length);

  const monthTotalInr = $derived(
    [...view.main, ...view.inactive].reduce((sum, t) => sum + t.totalInr, 0),
  );
  const monthTotalUsd = $derived.by(() => {
    let sum = 0;
    let has = false;
    for (const t of [...view.main, ...view.inactive]) {
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
              {#if expensesQuery.isSuccess && sourcesQuery.isSuccess}
                <Badge variant="secondary">{shownSources}</Badge>
              {/if}
            </h2>
            <div class="flex flex-wrap items-center gap-2">
              <Button variant="outline" onclick={() => (managing = true)}>
                <Layers /> Manage sources
              </Button>
              <Button variant="outline" onclick={() => (creatingSource = true)}>
                <Plus /> Add source
              </Button>
              <Button class="h-10 gap-2.5 pr-4 pl-2" onclick={() => addBill()}>
                <span
                  class="grid size-6 place-items-center rounded-xl bg-primary-foreground/15"
                  aria-hidden="true"
                >
                  <ReceiptText class="size-3.5" />
                </span>
                <span class="flex flex-col items-start text-left leading-tight">
                  <span class="text-sm font-semibold">Add bill</span>
                  <span class="text-[11px] font-normal opacity-75">for {monthLabel(month)}</span>
                </span>
              </Button>
            </div>
          </div>

          <div class="flex flex-wrap items-center justify-between gap-3">
            <div
              role="group"
              aria-label="Filter by billed status"
              class="flex items-center rounded-full border bg-muted/50 p-0.5 text-xs font-medium"
            >
              {#each [['all', 'All'], ['billed', 'Billed'], ['unbilled', 'Unbilled']] as [value, label] (value)}
                <button
                  type="button"
                  aria-pressed={billedFilter === value}
                  class={[
                    'rounded-full px-3 py-1 transition-colors',
                    billedFilter === value
                      ? 'bg-background text-foreground shadow-sm'
                      : 'text-muted-foreground hover:text-foreground',
                  ]}
                  onclick={() => (billedFilter = value as typeof billedFilter)}
                >
                  {label}
                </button>
              {/each}
            </div>
            <div class="flex items-center gap-1.5">
              <MonthPicker {month} onChange={selectMonth} />
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
                <EmptyDescription>Add a source, then record its bills.</EmptyDescription>
              </EmptyHeader>
              <Button onclick={() => (creatingSource = true)}><Plus /> Add source</Button>
            </Empty>
          {:else if billedFilter === 'billed' && !monthHasBills}
            {@render noBills(false)}
          {:else}
            {#if billedFilter !== 'billed' && !monthHasBills}
              {@render noBills(true)}
            {/if}

            {#if view.main.length > 0}
              {@render cardGrid(view.main)}
            {:else if view.inactive.length === 0}
              <Empty class="border py-12">
                <EmptyHeader>
                  <EmptyMedia variant="icon"><ReceiptText /></EmptyMedia>
                  <EmptyTitle>Every source is billed</EmptyTitle>
                  <EmptyDescription>
                    All {sources.length}
                    {sources.length === 1 ? 'source has' : 'sources have'} a bill for {monthLabel(
                      month,
                    )}.
                  </EmptyDescription>
                </EmptyHeader>
              </Empty>
            {/if}

            {#if view.inactive.length > 0}
              <div class="rounded-2xl border bg-card/40">
                <button
                  type="button"
                  class="flex w-full items-center gap-2 rounded-2xl px-4 py-3 text-left text-sm font-medium outline-none focus-visible:ring-3 focus-visible:ring-ring/30"
                  aria-expanded={showInactive}
                  aria-controls="inactive-sources"
                  onclick={() => (showInactive = !showInactive)}
                >
                  <Archive class="size-4 text-muted-foreground" />
                  Inactive sources
                  <Badge variant="secondary">{view.inactive.length}</Badge>
                  <ChevronDown
                    class={[
                      'ml-auto size-4 text-muted-foreground transition-transform',
                      showInactive && 'rotate-180',
                    ]}
                  />
                </button>
                {#if showInactive}
                  <div id="inactive-sources" class="px-3 pb-3">
                    {@render cardGrid(view.inactive)}
                  </div>
                {/if}
              </div>
            {/if}
          {/if}
        </section>
      </div>
    {/if}
  </main>
</div>

{#snippet cardGrid(list: SourceMonthTotal[])}
  <div class="grid items-start gap-3 md:grid-cols-2">
    {#each list as total (total.source.key)}
      <ExpenseSourceCard
        {total}
        {month}
        onEdit={(entry) => (editing = entry)}
        onView={(entry) => (viewing = entry)}
        onAddBill={addBill}
      />
    {/each}
  </div>
{/snippet}

{#snippet noBills(compact: boolean)}
  {#if compact}
    <div
      class="flex flex-wrap items-center gap-3 rounded-2xl border border-dashed px-4 py-3 sm:px-5"
    >
      <span class="grid size-9 shrink-0 place-items-center rounded-xl bg-muted">
        <ReceiptText class="size-4.5 text-muted-foreground" />
      </span>
      <div class="min-w-0 flex-1">
        <p class="text-sm font-medium">No bills for {monthLabel(month)} yet</p>
        <p class="text-xs text-muted-foreground">Add a bill to start tracking this month.</p>
      </div>
      <Button size="lg" onclick={() => addBill()}><Plus /> Add bill</Button>
    </div>
  {:else}
    <Empty class="border border-dashed py-12">
      <EmptyHeader>
        <EmptyMedia variant="icon"><ReceiptText /></EmptyMedia>
        <EmptyTitle>No bills for {monthLabel(month)} yet</EmptyTitle>
        <EmptyDescription>Please add a bill for this month to start tracking it.</EmptyDescription>
      </EmptyHeader>
      <Button onclick={() => addBill()}><Plus /> Add bill</Button>
    </Empty>
  {/if}
{/snippet}

<CreateExpenseDialog
  {month}
  open={creating}
  initialSourceKey={creatingSourceKey}
  onClose={() => (creating = false)}
/>
<CreateSourceDialog open={creatingSource} onClose={() => (creatingSource = false)} />
<ManageSourcesDialog
  open={managing}
  onClose={() => (managing = false)}
  onAddSource={() => (creatingSource = true)}
/>
<EditExpenseDialog entry={editing} {month} onClose={() => (editing = null)} />
<InvoiceViewerDialog entry={viewing} onClose={() => (viewing = null)} />
