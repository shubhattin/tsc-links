<script lang="ts">
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
  import { Spinner } from '$lib/components/ui/spinner';
  import {
    deleteInvoice,
    expensesQueryKey,
    parseInrToPaise,
    parseUsdToCents,
    updateExpense,
    uploadInvoice,
    type ExpenseDto,
    type UpdateExpenseBody,
  } from '$lib/expenses-api';
  import { queryClient } from '$lib/query-client';
  import { createMutation } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  let {
    entry,
    month,
    onClose,
  }: {
    entry: ExpenseDto | null;
    month: string;
    onClose: () => void;
  } = $props();

  let date = $state('');
  let label = $state('');
  let usd = $state('');
  let inr = $state('');
  let invoiceLink = $state('');
  let removeLink = $state(false);
  let removeBlob = $state(false);
  let invoiceFile = $state<File | null>(null);

  // Reset the form whenever a (different) entry is opened.
  $effect.pre(() => {
    if (!entry) return;
    date = entry.date;
    label = entry.label ?? '';
    usd = entry.usd_cents != null ? (entry.usd_cents / 100).toString() : '';
    inr = (entry.inr_paise / 100).toString();
    invoiceLink = entry.invoice_link ?? '';
    removeLink = false;
    removeBlob = false;
    invoiceFile = null;
  });

  const inrPaise = $derived(parseInrToPaise(inr));
  const usdCents = $derived(usd.trim() ? parseUsdToCents(usd) : undefined);

  /** Only send fields that changed (`null` clears optional ones). */
  const changes = $derived.by((): UpdateExpenseBody => {
    if (!entry) return {};
    const body: UpdateExpenseBody = {};
    if (date !== entry.date) body.date = date;
    if (label.trim() !== (entry.label ?? '')) body.label = label.trim();
    const wantUsd = usd.trim() ? (usdCents ?? entry.usd_cents) : null;
    if (wantUsd !== entry.usd_cents) body.usd_cents = wantUsd;
    if (inrPaise != null && inrPaise !== entry.inr_paise) body.inr_paise = inrPaise;
    if (removeLink) body.invoice_link = null;
    else if (invoiceLink.trim() !== (entry.invoice_link ?? ''))
      body.invoice_link = invoiceLink.trim();
    return body;
  });

  const formError = $derived.by(() => {
    if (!entry) return 'No expense selected';
    if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) return 'Pick a valid date';
    if (inrPaise == null) return 'Enter the final ₹ amount';
    if (usd.trim() && usdCents == null) return 'Dollar amount looks off';
    return null;
  });
  const dirty = $derived(Object.keys(changes).length > 0 || invoiceFile != null || removeBlob);

  const mutation = createMutation(
    () => ({
      mutationFn: async (id: number) => {
        let updated = entry;
        if (Object.keys(changes).length > 0) {
          updated = await updateExpense(id, { ...changes });
        }
        if (removeBlob && updated) {
          updated = await deleteInvoice(id);
        }
        if (invoiceFile && updated) {
          updated = await uploadInvoice(id, invoiceFile);
        }
        return updated;
      },
      onSuccess: async () => {
        await queryClient.invalidateQueries({ queryKey: expensesQueryKey(month) });
        toast.success('Expense updated');
        onClose();
      },
      onError: (e: Error) => toast.error('Could not update expense', { description: e.message }),
    }),
    () => queryClient,
  );

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!entry || formError || !dirty || mutation.isPending) return;
    mutation.mutate(entry.id);
  }
</script>

<Dialog open={entry !== null} onOpenChange={(open) => !open && onClose()}>
  <DialogContent class="sm:max-w-lg">
    <DialogHeader>
      <DialogTitle>Edit expense</DialogTitle>
      <DialogDescription>
        {#if entry}
          {entry.source_name} · {entry.date}
        {/if}
      </DialogDescription>
    </DialogHeader>
    <form id="edit-expense-form" onsubmit={submit} class="space-y-4">
      <div class="grid gap-4 sm:grid-cols-2">
        <div class="space-y-1.5">
          <Label for="edit-expense-date">Charge date</Label>
          <Input id="edit-expense-date" type="date" required bind:value={date} />
        </div>
        <div class="space-y-1.5">
          <Label for="edit-expense-label">
            Label <span class="font-normal text-muted-foreground">(optional)</span>
          </Label>
          <Input id="edit-expense-label" maxlength={120} autocomplete="off" bind:value={label} />
        </div>
      </div>
      <div class="grid gap-4 sm:grid-cols-2">
        <div class="space-y-1.5">
          <Label for="edit-expense-inr">Final charge (₹)</Label>
          <Input
            id="edit-expense-inr"
            inputmode="decimal"
            required
            autocomplete="off"
            bind:value={inr}
          />
        </div>
        <div class="space-y-1.5">
          <Label for="edit-expense-usd">
            Billed ($) <span class="font-normal text-muted-foreground">(optional)</span>
          </Label>
          <Input
            id="edit-expense-usd"
            inputmode="decimal"
            autocomplete="off"
            placeholder="Blank clears it"
            bind:value={usd}
          />
        </div>
      </div>
      <div class="space-y-1.5">
        <Label for="edit-expense-link">
          Invoice URL <span class="font-normal text-muted-foreground">(optional)</span>
        </Label>
        {#if removeLink}
          <p class="text-xs text-muted-foreground">
            The link will be removed on save.
            <button type="button" class="underline" onclick={() => (removeLink = false)}
              >Undo</button
            >
          </p>
        {:else}
          <div class="flex gap-2">
            <Input
              id="edit-expense-link"
              type="url"
              inputmode="url"
              autocomplete="off"
              spellcheck={false}
              class="flex-1"
              bind:value={invoiceLink}
            />
            {#if entry?.invoice_link}
              <Button type="button" variant="outline" size="sm" onclick={() => (removeLink = true)}>
                Remove
              </Button>
            {/if}
          </div>
        {/if}
      </div>
      <div class="space-y-1.5">
        <Label for="edit-expense-file">Invoice file</Label>
        {#if entry?.has_invoice && !invoiceFile && !removeBlob}
          <div class="flex items-center justify-between gap-2 rounded-lg border px-3 py-2 text-sm">
            <span class="truncate text-muted-foreground">{entry.invoice_name ?? 'Attached'}</span>
            <Button type="button" variant="outline" size="sm" onclick={() => (removeBlob = true)}>
              Remove
            </Button>
          </div>
        {:else if removeBlob && !invoiceFile}
          <p class="text-xs text-muted-foreground">
            The attached file will be removed on save — or pick a replacement below.
          </p>
        {/if}
        <Input
          id="edit-expense-file"
          type="file"
          accept="image/png,image/jpeg,image/webp,application/pdf"
          onchange={(e) => {
            invoiceFile = e.currentTarget.files?.[0] ?? null;
            if (invoiceFile) removeBlob = false;
          }}
        />
      </div>
      {#if formError && dirty}
        <p class="text-xs text-destructive">{formError}</p>
      {/if}
    </form>
    <DialogFooter>
      <Button variant="outline" onclick={onClose}>Cancel</Button>
      <Button
        type="submit"
        form="edit-expense-form"
        disabled={!dirty || !!formError || mutation.isPending}
      >
        {#if mutation.isPending}<Spinner />{/if}
        Save changes
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
