<script lang="ts">
  import DeleteSourceDialog from '$lib/components/expenses/delete-source-dialog.svelte';
  import EditSourceDialog from '$lib/components/expenses/edit-source-dialog.svelte';
  import SourceBrandIcon from '$lib/components/expenses/source-brand-icon.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu';
  import { Input } from '$lib/components/ui/input';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import {
    EXPENSE_SOURCES_QUERY_KEY,
    listSources,
    updateSource,
    type ExpenseSourceDto,
  } from '$lib/expenses-api';
  import { queryClient } from '$lib/query-client';
  import EllipsisVertical from '@lucide/svelte/icons/ellipsis-vertical';
  import Eye from '@lucide/svelte/icons/eye';
  import EyeOff from '@lucide/svelte/icons/eye-off';
  import Pencil from '@lucide/svelte/icons/pencil';
  import Plus from '@lucide/svelte/icons/plus';
  import Receipt from '@lucide/svelte/icons/receipt';
  import Search from '@lucide/svelte/icons/search';
  import Trash2 from '@lucide/svelte/icons/trash-2';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import { createMutation, createQuery } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  let {
    open,
    onClose,
    onAddSource,
  }: { open: boolean; onClose: () => void; onAddSource: () => void } = $props();

  const sourcesQuery = createQuery(
    () => ({
      queryKey: EXPENSE_SOURCES_QUERY_KEY,
      queryFn: listSources,
      staleTime: 5 * 60_000,
    }),
    () => queryClient,
  );

  const sources = $derived(sourcesQuery.data ?? []);
  const inactiveCount = $derived(sources.filter((s) => !s.active).length);

  let search = $state('');
  let editing = $state<ExpenseSourceDto | null>(null);
  let deleting = $state<ExpenseSourceDto | null>(null);

  // Fresh search every time the dialog opens.
  $effect.pre(() => {
    if (open) search = '';
  });

  const filtered = $derived.by(() => {
    const q = search.trim().toLowerCase();
    if (!q) return sources;
    return sources.filter((s) =>
      `${s.name} ${s.key} ${s.description ?? ''}`.toLowerCase().includes(q),
    );
  });

  const toggleActive = createMutation(
    () => ({
      mutationFn: (source: ExpenseSourceDto) =>
        updateSource(source.key, { active: !source.active }),
      onSuccess: async (updated: ExpenseSourceDto) => {
        await queryClient.invalidateQueries({ queryKey: EXPENSE_SOURCES_QUERY_KEY });
        toast.success(updated.active ? `${updated.name} is active` : `${updated.name} is inactive`);
      },
      onError: (e: Error) => toast.error('Could not update source', { description: e.message }),
    }),
    () => queryClient,
  );
</script>

<Dialog {open} onOpenChange={(isOpen) => !isOpen && onClose()}>
  <DialogContent
    class="flex h-[min(46rem,90svh)] flex-col gap-4 sm:max-w-3xl lg:max-w-5xl xl:max-w-6xl"
  >
    <DialogHeader class="pr-10">
      <DialogTitle>Manage sources</DialogTitle>
      <DialogDescription>
        {#if sourcesQuery.isSuccess}
          {sources.length}
          {sources.length === 1 ? 'source' : 'sources'}{inactiveCount > 0
            ? ` · ${inactiveCount} inactive`
            : ''}. A source with bills can't be deleted or re-keyed — mark it inactive instead.
        {:else}
          Edit, retire or delete the vendors you track bills against.
        {/if}
      </DialogDescription>
    </DialogHeader>

    <div class="flex items-center gap-2">
      <div class="relative min-w-0 flex-1">
        <Search
          class="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground"
        />
        <Input
          class="pl-9"
          placeholder="Search by name, key or description…"
          autocomplete="off"
          aria-label="Search sources"
          bind:value={search}
        />
      </div>
      <Button variant="outline" class="shrink-0" onclick={onAddSource}>
        <Plus /> Add source
      </Button>
    </div>

    <div class="-mx-1 min-h-0 flex-1 overflow-y-auto px-1 pb-1">
      {#if sourcesQuery.isPending}
        <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4" aria-busy="true">
          {#each [0, 1, 2, 3, 4, 5, 6, 7] as i (i)}
            <Skeleton class="h-28 w-full rounded-2xl" />
          {/each}
        </div>
      {:else if sourcesQuery.error}
        <div
          class="flex h-full flex-col items-center justify-center gap-3 text-center text-sm text-muted-foreground"
        >
          <TriangleAlert class="size-6" />
          <p>{sourcesQuery.error.message}</p>
          <Button variant="outline" onclick={() => sourcesQuery.refetch()}>Try again</Button>
        </div>
      {:else if filtered.length === 0}
        <div
          class="flex h-full flex-col items-center justify-center gap-1 text-center text-sm text-muted-foreground"
        >
          <Search class="mb-1 size-6" />
          {#if sources.length === 0}
            <p class="font-medium text-foreground">No sources yet</p>
            <p>Add a source to start tracking bills.</p>
          {:else}
            <p class="font-medium text-foreground">No matching sources</p>
            <p class="max-w-xs break-words">Nothing matches “{search.trim()}”.</p>
          {/if}
        </div>
      {:else}
        <ul class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {#each filtered as source (source.key)}
            {@const inUse = source.expense_count > 0}
            <li
              class={[
                'flex items-start gap-3 rounded-2xl border bg-card p-3.5',
                !source.active && 'opacity-70',
              ]}
            >
              <SourceBrandIcon sourceKey={source.key}>
                <Receipt class="size-5 text-muted-foreground" />
              </SourceBrandIcon>
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-1.5">
                  <p class="min-w-0 truncate text-sm font-semibold">{source.name}</p>
                  {#if !source.active}
                    <Badge variant="outline" class="shrink-0">Inactive</Badge>
                  {/if}
                </div>
                <p class="truncate font-mono text-[11px] text-muted-foreground">{source.key}</p>
                <p class="mt-1.5 line-clamp-2 text-xs break-words text-muted-foreground">
                  {source.description || 'No description'}
                </p>
                <p class="mt-2 text-[11px] text-muted-foreground tabular-nums">
                  {inUse
                    ? `${source.expense_count} ${source.expense_count === 1 ? 'bill' : 'bills'}`
                    : 'No bills'}
                </p>
              </div>
              <DropdownMenu>
                <DropdownMenuTrigger>
                  {#snippet child({ props })}
                    <Button
                      {...props}
                      variant="ghost"
                      size="icon-sm"
                      class="-mt-1 -mr-1.5 shrink-0"
                      aria-label={`Actions for ${source.name}`}
                    >
                      <EllipsisVertical />
                    </Button>
                  {/snippet}
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" class="w-52">
                  <DropdownMenuItem onclick={() => (editing = source)}>
                    <Pencil /> Edit
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    disabled={toggleActive.isPending}
                    onclick={() => toggleActive.mutate(source)}
                  >
                    {#if source.active}
                      <EyeOff /> Mark inactive
                    {:else}
                      <Eye /> Mark active
                    {/if}
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    variant="destructive"
                    disabled={inUse}
                    onclick={() => (deleting = source)}
                  >
                    <Trash2 /> Delete
                    {#if inUse}
                      <span class="ml-auto text-xs text-muted-foreground">has bills</span>
                    {/if}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </DialogContent>
</Dialog>

<EditSourceDialog source={editing} onClose={() => (editing = null)} />
<DeleteSourceDialog source={deleting} onClose={() => (deleting = null)} />
