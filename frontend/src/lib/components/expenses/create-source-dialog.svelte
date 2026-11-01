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
  import { createSource, EXPENSE_SOURCES_QUERY_KEY } from '$lib/expenses-api';
  import { queryClient } from '$lib/query-client';
  import Plus from '@lucide/svelte/icons/plus';
  import { createMutation } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  let { open, onClose }: { open: boolean; onClose: () => void } = $props();

  let key = $state('');
  let name = $state('');
  let description = $state('');

  function reset() {
    key = '';
    name = '';
    description = '';
  }

  // Fresh form every time the dialog opens.
  $effect.pre(() => {
    if (open) reset();
  });

  const formError = $derived.by(() => {
    if (!key.trim() || !/^[a-z0-9_-]{1,32}$/.test(key.trim().toLowerCase()))
      return 'Key must be 1–32 lowercase letters, digits, - or _';
    if (!name.trim()) return 'Give the source a name';
    if (description.trim().length > 2000) return 'Description must be at most 2000 characters';
    return null;
  });

  const mutation = createMutation(
    () => ({
      mutationFn: () =>
        createSource({
          key: key.trim().toLowerCase(),
          name: name.trim(),
          description: description.trim() || null,
        }),
      onSuccess: async () => {
        await queryClient.invalidateQueries({ queryKey: EXPENSE_SOURCES_QUERY_KEY });
        toast.success('Source added');
        onClose();
      },
      onError: (e: Error) => toast.error('Could not add source', { description: e.message }),
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
  <DialogContent class="max-h-[90svh] overflow-y-auto sm:max-w-md">
    <DialogHeader>
      <DialogTitle>Add source</DialogTitle>
      <DialogDescription>Track a new vendor or cost bucket for expenses.</DialogDescription>
    </DialogHeader>
    <form id="create-source-form" onsubmit={submit} class="space-y-4">
      <div class="space-y-1.5">
        <Label for="new-source-key">Source key</Label>
        <Input
          id="new-source-key"
          class="font-mono"
          maxlength={32}
          autocomplete="off"
          spellcheck={false}
          placeholder="my-vendor"
          bind:value={key}
        />
        <p class="text-xs text-muted-foreground">
          Lowercase letters, digits, - or _. Used in URLs and API calls; the display name is shown
          in the UI.
        </p>
      </div>
      <div class="space-y-1.5">
        <Label for="new-source-name">Display name</Label>
        <Input
          id="new-source-name"
          maxlength={64}
          autocomplete="off"
          placeholder="My Vendor"
          bind:value={name}
        />
      </div>
      <div class="space-y-1.5">
        <Label for="new-source-desc">
          Description <span class="font-normal text-muted-foreground">(optional)</span>
        </Label>
        <Input
          id="new-source-desc"
          maxlength={2000}
          autocomplete="off"
          placeholder="What is this for?"
          bind:value={description}
        />
      </div>
      {#if formError && (key || name)}
        <p class="text-xs text-destructive">{formError}</p>
      {/if}
    </form>
    <DialogFooter>
      <Button variant="outline" onclick={onClose}>Cancel</Button>
      <Button type="submit" form="create-source-form" disabled={!!formError || mutation.isPending}>
        {#if mutation.isPending}<Spinner />{/if}
        <Plus /> Add source
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
