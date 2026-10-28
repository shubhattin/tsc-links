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
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupInput,
    InputGroupText,
  } from '$lib/components/ui/input-group';
  import { Label } from '$lib/components/ui/label';
  import { Spinner } from '$lib/components/ui/spinner';
  import { Switch } from '$lib/components/ui/switch';
  import { slugError, updateLink, type LinkDto, type UpdateLinkBody } from '$lib/links-api';
  import { replaceLinkInCache } from '$lib/links-cache';
  import { queryClient } from '$lib/query-client';
  import { createMutation } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  let { link, onClose }: { link: LinkDto | null; onClose: () => void } = $props();

  let slug = $state('');
  let url = $state('');
  let title = $state('');
  let enabled = $state(true);

  // Reset the form whenever a (different) link is opened.
  $effect.pre(() => {
    if (!link) return;
    slug = link.slug;
    url = link.url;
    title = link.title ?? '';
    enabled = link.enabled;
  });

  const host = $derived(typeof window === 'undefined' ? '' : window.location.host);
  const slugErr = $derived(slug.trim() ? slugError(slug.trim()) : 'Slug is required');

  /** Only send fields that changed. */
  const changes = $derived.by((): UpdateLinkBody => {
    if (!link) return {};
    const body: UpdateLinkBody = {};
    if (slug.trim() !== link.slug) body.slug = slug.trim();
    if (url.trim() !== link.url) body.url = url.trim();
    if (title.trim() !== (link.title ?? '')) body.title = title.trim();
    if (enabled !== link.enabled) body.enabled = enabled;
    return body;
  });
  const dirty = $derived(Object.keys(changes).length > 0);

  const mutation = createMutation(
    () => ({
      mutationFn: (input: { id: string; body: UpdateLinkBody }) => updateLink(input.id, input.body),
      onSuccess: (updated: LinkDto) => {
        replaceLinkInCache(updated);
        toast.success('Link updated');
        onClose();
      },
      onError: (e: Error) => toast.error('Could not update link', { description: e.message }),
    }),
    () => queryClient,
  );

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!link || slugErr || !dirty || mutation.isPending) return;
    mutation.mutate({ id: link.id, body: changes });
  }
</script>

<Dialog open={link !== null} onOpenChange={(open) => !open && onClose()}>
  <DialogContent class="sm:max-w-lg">
    <DialogHeader>
      <DialogTitle>Edit link</DialogTitle>
      <DialogDescription>Changes apply to the very next visit of the short link.</DialogDescription>
    </DialogHeader>
    <form id="edit-link-form" onsubmit={submit} class="space-y-4">
      <div class="space-y-1.5">
        <Label for="edit-slug">Slug</Label>
        <InputGroup>
          <InputGroupAddon class="pr-0">
            <InputGroupText class="max-w-48 truncate font-mono text-xs">{host}/</InputGroupText>
          </InputGroupAddon>
          <InputGroupInput
            id="edit-slug"
            class="pl-0.5 font-mono"
            autocomplete="off"
            spellcheck={false}
            maxlength={64}
            required
            aria-invalid={slugErr ? true : undefined}
            bind:value={slug}
          />
        </InputGroup>
        {#if slugErr}
          <p class="text-xs text-destructive">{slugErr}</p>
        {:else if link && slug.trim() !== link.slug}
          <p class="text-xs text-muted-foreground">
            The old slug <span class="font-mono">/{link.slug}</span> will stop working.
          </p>
        {/if}
      </div>
      <div class="space-y-1.5">
        <Label for="edit-url">Destination URL</Label>
        <Input
          id="edit-url"
          type="url"
          inputmode="url"
          required
          autocomplete="off"
          spellcheck={false}
          bind:value={url}
        />
      </div>
      <div class="space-y-1.5">
        <Label for="edit-title">
          Title <span class="font-normal text-muted-foreground">(optional)</span>
        </Label>
        <Input id="edit-title" maxlength={255} autocomplete="off" bind:value={title} />
      </div>
      <label
        class="flex cursor-pointer items-center justify-between gap-4 rounded-xl border px-4 py-3"
      >
        <span class="space-y-0.5">
          <span class="block text-sm font-medium">Enabled</span>
          <span class="block text-xs text-muted-foreground">
            Disabled links show a 404 page instead of redirecting.
          </span>
        </span>
        <Switch bind:checked={enabled} />
      </label>
    </form>
    <DialogFooter>
      <Button variant="outline" onclick={onClose}>Cancel</Button>
      <Button
        type="submit"
        form="edit-link-form"
        disabled={!dirty || !!slugErr || !url.trim() || mutation.isPending}
      >
        {#if mutation.isPending}<Spinner />{/if}
        Save changes
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
