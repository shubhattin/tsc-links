<script lang="ts">
  import ExpenseDateField from '$lib/components/expenses/expense-date-field.svelte';
  import SourceBrandIcon from '$lib/components/expenses/source-brand-icon.svelte';
  import { Button } from '$lib/components/ui/button';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import * as Select from '$lib/components/ui/select';
  import { Spinner } from '$lib/components/ui/spinner';
  import { Textarea } from '$lib/components/ui/textarea';
  import {
    createExpense,
    createSource,
    EXPENSE_SOURCES_QUERY_KEY,
    expensesQueryKey,
    listSources,
    parseInrToPaise,
    parseUsdToCents,
    todayIso,
    uploadInvoice,
    type CreateExpenseBody,
    type ExpenseSourceDto,
  } from '$lib/expenses-api';
  import { queryClient } from '$lib/query-client';
  import Plus from '@lucide/svelte/icons/plus';
  import Search from '@lucide/svelte/icons/search';
  import { createMutation, createQuery } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  let {
    month,
    open,
    onClose,
  }: {
    month: string;
    open: boolean;
    onClose: () => void;
  } = $props();

  const sourcesQuery = createQuery(
    () => ({
      queryKey: EXPENSE_SOURCES_QUERY_KEY,
      queryFn: listSources,
      staleTime: 5 * 60_000,
    }),
    () => queryClient,
  );

  const sources = $derived(sourcesQuery.data ?? []);

  let sourceFilter = $state('');
  const filteredSources = $derived.by(() => {
    const q = sourceFilter.trim().toLowerCase();
    if (!q) return sources;
    return sources.filter((s) =>
      `${s.name} ${s.key} ${s.description ?? ''}`.toLowerCase().includes(q),
    );
  });

  let sourceKey = $state('');
  let newKey = $state('');
  let newName = $state('');
  let newDescription = $state('');
  let date = $state(todayIso());
  let label = $state('');
  let note = $state('');
  let usd = $state('');
  let inr = $state('');
  let invoiceLink = $state('');
  let invoiceFile = $state<File | null>(null);
  let fileInput = $state<HTMLInputElement | null>(null);

  function reset() {
    sourceKey = '';
    sourceFilter = '';
    newKey = '';
    newName = '';
    newDescription = '';
    date = todayIso();
    label = '';
    note = '';
    usd = '';
    inr = '';
    invoiceLink = '';
    invoiceFile = null;
    if (fileInput) fileInput.value = '';
  }

  // Fresh form every time the dialog opens.
  $effect.pre(() => {
    if (open) reset();
  });

  const creatingNew = $derived(sourceKey === '__new__');
  const inrPaise = $derived(parseInrToPaise(inr));
  const usdCents = $derived(usd.trim() ? parseUsdToCents(usd) : undefined);

  const formError = $derived.by(() => {
    if (!sourceKey) return 'Pick a source';
    if (creatingNew) {
      if (!newKey.trim() || !/^[a-z0-9_-]{1,32}$/.test(newKey.trim().toLowerCase()))
        return 'New key must be 1–32 lowercase letters, digits, - or _';
      if (!newName.trim()) return 'New source needs a name';
    }
    if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) return 'Pick a valid date';
    if (inrPaise == null) return 'Enter the final ₹ amount';
    if (usd.trim() && usdCents == null) return 'Dollar amount looks off';
    if (note.trim().length > 2000) return 'Note must be at most 2000 characters';
    return null;
  });

  const mutation = createMutation(
    () => ({
      mutationFn: async () => {
        let key = sourceKey;
        if (creatingNew) {
          const created: ExpenseSourceDto = await createSource({
            key: newKey.trim().toLowerCase(),
            name: newName.trim(),
            description: newDescription.trim() || null,
          });
          await queryClient.invalidateQueries({ queryKey: EXPENSE_SOURCES_QUERY_KEY });
          key = created.key;
        }
        const body: CreateExpenseBody = {
          source_key: key,
          date,
          label: label.trim() || null,
          note: note.trim() || null,
          usd_cents: usdCents ?? null,
          inr_paise: inrPaise ?? 0,
          invoice_link: invoiceLink.trim() || null,
        };
        let entry = await createExpense(body);
        if (invoiceFile) entry = await uploadInvoice(entry.id, invoiceFile);
        return entry;
      },
      onSuccess: async () => {
        await queryClient.invalidateQueries({ queryKey: expensesQueryKey(month) });
        toast.success('Expense recorded');
        onClose();
      },
      onError: (e: Error) => toast.error('Could not record expense', { description: e.message }),
    }),
    () => queryClient,
  );

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (formError || mutation.isPending) return;
    mutation.mutate();
  }
</script>

<Dialog {open} onOpenChange={(isOpen) => !isOpen && onClose()}>
  <DialogContent class="max-h-[90svh] overflow-y-auto sm:max-w-lg">
    <DialogHeader>
      <DialogTitle>Add expense</DialogTitle>
      <DialogDescription>Charged costs land in the current month view.</DialogDescription>
    </DialogHeader>
    <form id="create-expense-form" onsubmit={submit} class="space-y-4">
      <div class="grid gap-4 sm:grid-cols-2">
        <div class="space-y-1.5">
          <Label for="expense-source">Source</Label>
          <Select.Root type="single" bind:value={sourceKey}>
            <Select.Trigger id="expense-source" class="w-full">
              {#if sourceKey && sourceKey !== '__new__'}
                <SourceBrandIcon {sourceKey} size="sm" />
              {/if}
              <Select.Value placeholder="Pick a source" />
            </Select.Trigger>
            <Select.Content class="max-h-80">
              <div class="sticky top-0 z-10 bg-popover pt-1 pb-1.5">
                <div class="relative">
                  <Search
                    class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
                  />
                  <Input
                    class="h-8 rounded-xl pr-2 pl-8 text-sm"
                    placeholder="Search sources…"
                    autocomplete="off"
                    bind:value={sourceFilter}
                    onkeydown={(e) => {
                      // Let the select handle Escape (close); keep typing,
                      // arrows and Enter local to the search box.
                      if (e.key !== 'Escape') e.stopPropagation();
                    }}
                  />
                </div>
              </div>
              <Select.Group>
                {#each filteredSources as source (source.key)}
                  <Select.Item value={source.key} label={source.name}>
                    <SourceBrandIcon sourceKey={source.key} size="sm" />
                    <span class="min-w-0">
                      <span class="block truncate">{source.name}</span>
                      {#if source.description}
                        <span class="block truncate text-xs text-muted-foreground">
                          {source.description}
                        </span>
                      {/if}
                    </span>
                  </Select.Item>
                {/each}
                {#if filteredSources.length === 0}
                  <p class="px-3 py-4 text-center text-xs text-muted-foreground">
                    No sources match “{sourceFilter.trim()}”.
                  </p>
                {/if}
              </Select.Group>
              <Select.Separator />
              <Select.Item value="__new__" label="New source…">
                <span
                  aria-hidden="true"
                  class="grid size-7 shrink-0 place-items-center rounded-xl border border-dashed text-lg leading-none text-primary"
                  >+</span
                >
                New source…
              </Select.Item>
            </Select.Content>
          </Select.Root>
        </div>
        <ExpenseDateField id="expense-date" bind:value={date} />
      </div>

      {#if creatingNew}
        <div class="grid gap-4 rounded-xl border border-dashed p-4 sm:grid-cols-2">
          <div class="space-y-1.5">
            <Label for="new-source-key">Source key</Label>
            <Input
              id="new-source-key"
              class="font-mono"
              maxlength={32}
              autocomplete="off"
              spellcheck={false}
              placeholder="my-vendor"
              bind:value={newKey}
            />
          </div>
          <div class="space-y-1.5">
            <Label for="new-source-name">Display name</Label>
            <Input
              id="new-source-name"
              maxlength={64}
              autocomplete="off"
              placeholder="My Vendor"
              bind:value={newName}
            />
          </div>
          <div class="space-y-1.5 sm:col-span-2">
            <Label for="new-source-desc"
              >Description <span class="font-normal text-muted-foreground">(optional)</span></Label
            >
            <Input
              id="new-source-desc"
              maxlength={2000}
              autocomplete="off"
              placeholder="What is this for?"
              bind:value={newDescription}
            />
          </div>
        </div>
      {/if}

      <div class="grid gap-4 sm:grid-cols-3">
        <div class="space-y-1.5">
          <Label for="expense-inr">Final charge (₹)</Label>
          <Input
            id="expense-inr"
            inputmode="decimal"
            required
            autocomplete="off"
            placeholder="1,750.00"
            bind:value={inr}
            aria-invalid={inr.trim() && inrPaise == null ? true : undefined}
          />
        </div>
        <div class="space-y-1.5">
          <Label for="expense-usd">
            Billed ($) <span class="font-normal text-muted-foreground">(optional)</span>
          </Label>
          <Input
            id="expense-usd"
            inputmode="decimal"
            autocomplete="off"
            placeholder="20.00"
            bind:value={usd}
            aria-invalid={usd.trim() && usdCents == null ? true : undefined}
          />
        </div>
        <div class="space-y-1.5">
          <Label for="expense-label">
            Label <span class="font-normal text-muted-foreground">(optional)</span>
          </Label>
          <Input
            id="expense-label"
            maxlength={120}
            autocomplete="off"
            placeholder="Pro plan"
            bind:value={label}
          />
        </div>
      </div>

      <div class="space-y-1.5">
        <Label for="expense-note">
          Note <span class="font-normal text-muted-foreground">(optional)</span>
        </Label>
        <Textarea
          id="expense-note"
          rows={2}
          maxlength={2000}
          autocomplete="off"
          placeholder="Why this charge, who it's for…"
          bind:value={note}
        />
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <div class="space-y-1.5">
          <Label for="expense-invoice-link">
            Invoice URL <span class="font-normal text-muted-foreground">(optional)</span>
          </Label>
          <Input
            id="expense-invoice-link"
            type="url"
            inputmode="url"
            autocomplete="off"
            spellcheck={false}
            placeholder="https://…/invoice.pdf"
            bind:value={invoiceLink}
          />
        </div>
        <div class="space-y-1.5">
          <Label for="expense-invoice-file">
            Invoice file <span class="font-normal text-muted-foreground">(image/PDF, ≤4 MiB)</span>
          </Label>
          <Input
            id="expense-invoice-file"
            type="file"
            accept="image/png,image/jpeg,image/webp,application/pdf"
            bind:ref={fileInput}
            onchange={(e) => {
              invoiceFile = e.currentTarget.files?.[0] ?? null;
            }}
          />
        </div>
      </div>

      {#if formError && (sourceKey || inr)}
        <p class="text-xs text-destructive">{formError}</p>
      {/if}
    </form>
    <DialogFooter>
      <Button variant="outline" onclick={onClose}>Cancel</Button>
      <Button type="submit" form="create-expense-form" disabled={!!formError || mutation.isPending}>
        {#if mutation.isPending}<Spinner />{/if}
        <Plus /> Record expense
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
