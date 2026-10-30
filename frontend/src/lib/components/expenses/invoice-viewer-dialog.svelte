<script lang="ts">
  /**
   * In-dialog invoice preview. Images render directly; PDFs use the browser's
   * built-in viewer (no extra packages). The blob URL is revoked on close.
   */
  import { Button } from '$lib/components/ui/button';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { invoiceObjectUrl, type ExpenseDto } from '$lib/expenses-api';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import { untrack } from 'svelte';

  let { entry, onClose }: { entry: ExpenseDto | null; onClose: () => void } = $props();

  let objectUrl = $state<string | null>(null);
  let isPdf = $state(false);
  let loading = $state(false);
  let error = $state<string | null>(null);

  function revoke() {
    if (objectUrl) {
      URL.revokeObjectURL(objectUrl);
      objectUrl = null;
    }
  }

  // Fetch the blob whenever a (different) invoice is opened. `revoke()`
  // reads `objectUrl`, so it must run untracked — otherwise the effect
  // subscribes to the URL and refetches in a loop.
  $effect.pre(() => {
    if (!entry?.has_invoice) {
      loading = false;
      error = null;
      return;
    }
    const id = entry.id;
    isPdf = (entry.invoice_name ?? '').toLowerCase().endsWith('.pdf');
    loading = true;
    error = null;
    untrack(revoke);
    invoiceObjectUrl(id)
      .then(({ url }) => {
        if (entry?.id !== id) {
          URL.revokeObjectURL(url);
          return;
        }
        objectUrl = url;
      })
      .catch((e: unknown) => {
        error = e instanceof Error ? e.message : 'Could not load invoice';
      })
      .finally(() => {
        if (entry?.id === id) loading = false;
      });
    return () => {
      untrack(revoke);
    };
  });
</script>

<Dialog open={entry !== null} onOpenChange={(open) => !open && onClose()}>
  <DialogContent class="sm:max-w-3xl">
    <DialogHeader>
      <DialogTitle class="truncate">
        {entry?.invoice_name ?? 'Invoice'}
      </DialogTitle>
      <DialogDescription>
        {#if entry}
          {entry.source_name} · {entry.date}
        {/if}
      </DialogDescription>
    </DialogHeader>
    {#if loading}
      <Skeleton class="h-[60svh] w-full rounded-xl" />
    {:else if error || !objectUrl}
      <div class="flex flex-col items-center gap-2 py-12 text-center">
        <TriangleAlert class="size-6 text-destructive" />
        <p class="text-sm text-muted-foreground">{error ?? 'No invoice attached'}</p>
        <Button variant="outline" size="sm" onclick={onClose}>Close</Button>
      </div>
    {:else if isPdf}
      <iframe
        title={entry?.invoice_name ?? 'Invoice PDF'}
        src={objectUrl}
        class="h-[70svh] w-full rounded-xl border"
      ></iframe>
    {:else}
      <div
        class="grid max-h-[70svh] place-items-center overflow-auto rounded-xl border bg-muted/30 p-2"
      >
        <img
          src={objectUrl}
          alt={entry?.invoice_name ?? 'Invoice image'}
          class="max-h-[68svh] w-auto rounded-lg object-contain"
        />
      </div>
    {/if}
  </DialogContent>
</Dialog>
