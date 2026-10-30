<script lang="ts">
  import DeleteLinkDialog from '$lib/components/links/delete-link-dialog.svelte';
  import EditLinkDialog from '$lib/components/links/edit-link-dialog.svelte';
  import LinkRow from '$lib/components/links/link-row.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import {
    Empty,
    EmptyDescription,
    EmptyHeader,
    EmptyMedia,
    EmptyTitle,
  } from '$lib/components/ui/empty';
  import { InputGroup, InputGroupAddon, InputGroupInput } from '$lib/components/ui/input-group';
  import { Kbd } from '$lib/components/ui/kbd';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { LINKS_QUERY_KEY, listLinks, updateLink, type LinkDto } from '$lib/links-api';
  import { replaceLinkInCache } from '$lib/links-cache';
  import { queryClient } from '$lib/query-client';
  import Link2Off from '@lucide/svelte/icons/link-2-off';
  import Plus from '@lucide/svelte/icons/plus';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import Search from '@lucide/svelte/icons/search';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import { createMutation, createQuery } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  const query = createQuery(
    () => ({ queryKey: LINKS_QUERY_KEY, queryFn: listLinks, staleTime: 30_000 }),
    () => queryClient,
  );

  let { onAdd }: { onAdd: () => void } = $props();

  let search = $state('');
  let searchInput = $state<HTMLInputElement | null>(null);
  let editing = $state<LinkDto | null>(null);
  let deleting = $state<LinkDto | null>(null);
  let togglingId = $state<string | null>(null);

  const links = $derived(query.data ?? []);
  const filtered = $derived.by(() => {
    const q = search.trim().toLowerCase();
    if (!q) return links;
    return links.filter((l) =>
      [l.slug, l.url, l.title ?? ''].some((field) => field.toLowerCase().includes(q)),
    );
  });

  const toggleMutation = createMutation(
    () => ({
      mutationFn: (input: { link: LinkDto; enabled: boolean }) =>
        updateLink(input.link.id, { enabled: input.enabled }),
      onMutate: (input: { link: LinkDto; enabled: boolean }) => {
        togglingId = input.link.id;
        // Optimistic flip; rolled back on error.
        replaceLinkInCache({ ...input.link, enabled: input.enabled });
      },
      onSuccess: (updated: LinkDto) => {
        replaceLinkInCache(updated);
        toast.success(updated.enabled ? `/${updated.slug} enabled` : `/${updated.slug} disabled`);
      },
      onError: (e: Error, input: { link: LinkDto; enabled: boolean }) => {
        replaceLinkInCache(input.link);
        toast.error('Could not update link', { description: e.message });
      },
      onSettled: () => {
        togglingId = null;
      },
    }),
    () => queryClient,
  );

  /** `/` focuses search (unless typing somewhere already). */
  function onKeydown(event: KeyboardEvent) {
    if (event.key !== '/' || event.metaKey || event.ctrlKey || event.altKey) return;
    const target = event.target as HTMLElement | null;
    if (target?.closest('input, textarea, [contenteditable="true"], [role="dialog"]')) return;
    event.preventDefault();
    searchInput?.focus();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<section class="rise space-y-3" style="animation-delay: 60ms">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <h2 class="flex items-center gap-2 text-lg font-semibold tracking-tight">
      Links
      {#if query.isSuccess}
        <Badge variant="secondary">{links.length}</Badge>
      {/if}
    </h2>
    <div class="flex items-center gap-1.5">
      <InputGroup class="w-56 sm:w-64">
        <InputGroupAddon>
          <Search />
        </InputGroupAddon>
        <InputGroupInput
          bind:ref={searchInput}
          type="search"
          placeholder="Search links"
          aria-label="Search links"
          bind:value={search}
        />
        {#if !search}
          <InputGroupAddon align="inline-end" class="max-sm:hidden">
            <Kbd>/</Kbd>
          </InputGroupAddon>
        {/if}
      </InputGroup>
      <Button
        variant="ghost"
        size="icon"
        aria-label="Refresh"
        disabled={query.isFetching}
        onclick={() => query.refetch()}
      >
        <RefreshCw class={query.isFetching ? 'animate-spin motion-reduce:animate-none' : ''} />
      </Button>
      <Button onclick={onAdd}><Plus /> Shorten</Button>
    </div>
  </div>

  <div class="overflow-hidden rounded-2xl border bg-card">
    {#if query.isPending}
      <ul class="divide-y" aria-busy="true">
        {#each [0, 1, 2] as i (i)}
          <li class="flex items-center gap-3 px-5 py-4">
            <div class="flex-1 space-y-2">
              <Skeleton class="h-4 w-40" />
              <Skeleton class="h-3 w-72 max-w-full" />
            </div>
            <Skeleton class="h-5 w-9 rounded-full" />
          </li>
        {/each}
      </ul>
    {:else if query.isError}
      <Empty class="py-12">
        <EmptyHeader>
          <EmptyMedia variant="icon"><TriangleAlert /></EmptyMedia>
          <EmptyTitle>Couldn't load links</EmptyTitle>
          <EmptyDescription>{query.error.message}</EmptyDescription>
        </EmptyHeader>
        <Button variant="outline" onclick={() => query.refetch()}>Try again</Button>
      </Empty>
    {:else if links.length === 0}
      <Empty class="py-12">
        <EmptyHeader>
          <EmptyMedia variant="icon"><Link2Off /></EmptyMedia>
          <EmptyTitle>No links yet</EmptyTitle>
          <EmptyDescription>Shorten your first URL to get started.</EmptyDescription>
        </EmptyHeader>
        <Button onclick={onAdd}><Plus /> Shorten a link</Button>
      </Empty>
    {:else if filtered.length === 0}
      <Empty class="py-12">
        <EmptyHeader>
          <EmptyMedia variant="icon"><Search /></EmptyMedia>
          <EmptyTitle>No matches</EmptyTitle>
          <EmptyDescription>Nothing matches “{search.trim()}”.</EmptyDescription>
        </EmptyHeader>
        <Button variant="outline" onclick={() => (search = '')}>Clear search</Button>
      </Empty>
    {:else}
      <ul class="divide-y">
        {#each filtered as link (link.id)}
          <LinkRow
            {link}
            toggling={togglingId === link.id}
            onToggle={(enabled) => toggleMutation.mutate({ link, enabled })}
            onEdit={() => (editing = link)}
            onDelete={() => (deleting = link)}
          />
        {/each}
      </ul>
    {/if}
  </div>
</section>

<EditLinkDialog link={editing} onClose={() => (editing = null)} />
<DeleteLinkDialog link={deleting} onClose={() => (deleting = null)} />
