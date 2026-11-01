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
  import { Switch } from '$lib/components/ui/switch';
  import { Textarea } from '$lib/components/ui/textarea';
  import {
    EXPENSE_SOURCES_QUERY_KEY,
    updateSource,
    type ExpenseSourceDto,
    type UpdateSourceBody,
  } from '$lib/expenses-api';
  import { queryClient } from '$lib/query-client';
  import Check from '@lucide/svelte/icons/check';
  import Lock from '@lucide/svelte/icons/lock';
  import { createMutation } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  let { source, onClose }: { source: ExpenseSourceDto | null; onClose: () => void } = $props();

  let key = $state('');
  let name = $state('');
  let description = $state('');
  let active = $state(true);

  // Reset the form whenever a (different) source is opened.
  $effect.pre(() => {
    if (!source) return;
    key = source.key;
    name = source.name;
    description = source.description ?? '';
    active = source.active;
  });

  /** The key is the primary key — it can only change while no bill points at it. */
  const keyLocked = $derived((source?.expense_count ?? 0) > 0);

  /** Only send fields that changed. */
  const changes = $derived.by((): UpdateSourceBody => {
    if (!source) return {};
    const body: UpdateSourceBody = {};
    if (!keyLocked && key.trim().toLowerCase() !== source.key) body.key = key.trim().toLowerCase();
    if (name.trim() !== source.name) body.name = name.trim();
    if (description.trim() !== (source.description ?? '')) body.description = description.trim();
    if (active !== source.active) body.active = active;
    return body;
  });
  const dirty = $derived(Object.keys(changes).length > 0);

  const formError = $derived.by(() => {
    if (!source) return 'No source selected';
    if (!keyLocked && !/^[a-z0-9_-]{1,32}$/.test(key.trim().toLowerCase()))
      return 'Key must be 1–32 lowercase letters, digits, - or _';
    if (!name.trim()) return 'Give the source a name';
    if (description.trim().length > 2000) return 'Description must be at most 2000 characters';
    return null;
  });

  const mutation = createMutation(
    () => ({
      mutationFn: (target: ExpenseSourceDto) => updateSource(target.key, { ...changes }),
      onSuccess: async () => {
        await Promise.all([
          queryClient.invalidateQueries({ queryKey: EXPENSE_SOURCES_QUERY_KEY }),
          // Bills carry the source name.
          queryClient.invalidateQueries({ queryKey: ['expenses'] }),
        ]);
        toast.success('Source updated');
        onClose();
      },
      onError: (e: Error) => toast.error('Could not update source', { description: e.message }),
    }),
    () => queryClient,
  );

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!source || formError || !dirty || mutation.isPending) return;
    mutation.mutate(source);
  }
</script>

<Dialog open={source !== null} onOpenChange={(open) => !open && onClose()}>
  <DialogContent class="max-h-[90svh] overflow-y-auto sm:max-w-md">
    <DialogHeader>
      <DialogTitle>Edit source</DialogTitle>
      <DialogDescription class="line-clamp-2 break-words">
        {#if source}
          {source.name}
        {/if}
      </DialogDescription>
    </DialogHeader>
    <form id="edit-source-form" onsubmit={submit} class="space-y-4">
      <div class="space-y-1.5">
        <Label for="edit-source-key" class="gap-1.5">
          Source key
          {#if keyLocked}<Lock class="size-3 text-muted-foreground" />{/if}
        </Label>
        <Input
          id="edit-source-key"
          class="font-mono"
          maxlength={32}
          autocomplete="off"
          spellcheck={false}
          disabled={keyLocked}
          bind:value={key}
        />
        <p class="text-xs text-muted-foreground">
          {#if keyLocked}
            Locked — {source?.expense_count}
            {source?.expense_count === 1 ? 'bill uses' : 'bills use'} this source.
          {:else}
            No bills yet, so the key can still be changed.
          {/if}
        </p>
      </div>
      <div class="space-y-1.5">
        <Label for="edit-source-name">Display name</Label>
        <Input id="edit-source-name" maxlength={64} autocomplete="off" bind:value={name} />
      </div>
      <div class="space-y-1.5">
        <Label for="edit-source-desc">
          Description <span class="font-normal text-muted-foreground">(optional)</span>
        </Label>
        <Textarea
          id="edit-source-desc"
          rows={3}
          maxlength={2000}
          autocomplete="off"
          placeholder="What is this for?"
          bind:value={description}
        />
      </div>
      <div class="flex items-center justify-between gap-4 rounded-2xl border px-3.5 py-3">
        <div class="min-w-0">
          <Label for="edit-source-active" class="text-sm">Active</Label>
          <p class="text-xs text-muted-foreground">
            Inactive sources are tucked away in All and Unbilled, and can't take new bills.
          </p>
        </div>
        <Switch id="edit-source-active" bind:checked={active} />
      </div>
      {#if formError}
        <p class="text-xs text-destructive">{formError}</p>
      {/if}
    </form>
    <DialogFooter>
      <Button variant="outline" onclick={onClose}>Cancel</Button>
      <Button
        type="submit"
        form="edit-source-form"
        disabled={!!formError || !dirty || mutation.isPending}
      >
        {#if mutation.isPending}<Spinner />{:else}<Check />{/if}
        Save changes
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
