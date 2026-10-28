<script lang="ts">
  import CopyButton from '$lib/components/links/copy-button.svelte';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupInput,
    InputGroupText,
  } from '$lib/components/ui/input-group';
  import { Label } from '$lib/components/ui/label';
  import { Spinner } from '$lib/components/ui/spinner';
  import {
    createLink,
    shortUrl,
    shortUrlDisplay,
    slugError,
    type CreateLinkBody,
    type LinkDto,
  } from '$lib/links-api';
  import { addLinkToCache } from '$lib/links-cache';
  import { queryClient } from '$lib/query-client';
  import ExternalLink from '@lucide/svelte/icons/external-link';
  import LinkIcon from '@lucide/svelte/icons/link';
  import X from '@lucide/svelte/icons/x';
  import { createMutation } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  let url = $state('');
  let slug = $state('');
  let title = $state('');
  let created = $state<LinkDto | null>(null);
  let urlInput = $state<HTMLInputElement | null>(null);

  const slugErr = $derived(slugError(slug.trim()));
  const host = $derived(typeof window === 'undefined' ? '' : window.location.host);

  const mutation = createMutation(
    () => ({
      mutationFn: (body: CreateLinkBody) => createLink(body),
      onSuccess: async (link: LinkDto) => {
        addLinkToCache(link);
        created = link;
        url = '';
        slug = '';
        title = '';
        try {
          await navigator.clipboard.writeText(shortUrl(link.slug));
          toast.success('Short link created and copied');
        } catch {
          toast.success('Short link created');
        }
      },
      onError: (e: Error) => toast.error('Could not create link', { description: e.message }),
    }),
    () => queryClient,
  );

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (slugErr || mutation.isPending) return;
    mutation.mutate({
      url: url.trim(),
      slug: slug.trim() || null,
      title: title.trim() || null,
    });
  }
</script>

<Card class="rise">
  <CardHeader>
    <CardTitle class="text-lg">Shorten a link</CardTitle>
    <CardDescription>
      Paste a long URL. Leave the slug empty for a random one — you can edit it any time.
    </CardDescription>
  </CardHeader>
  <CardContent>
    <form onsubmit={submit} class="space-y-4">
      <div class="flex flex-col gap-2 sm:flex-row">
        <InputGroup class="h-10 flex-1">
          <InputGroupAddon>
            <LinkIcon />
          </InputGroupAddon>
          <InputGroupInput
            bind:ref={urlInput}
            type="url"
            inputmode="url"
            required
            autocomplete="off"
            spellcheck={false}
            placeholder="https://example.com/a/very/long/url"
            aria-label="Destination URL"
            bind:value={url}
          />
        </InputGroup>
        <Button
          type="submit"
          size="lg"
          class="h-10 sm:w-32"
          disabled={mutation.isPending || !url.trim() || !!slugErr}
        >
          {#if mutation.isPending}<Spinner />{/if}
          Shorten
        </Button>
      </div>

      <div class="grid gap-4 sm:grid-cols-2">
        <div class="space-y-1.5">
          <Label for="create-slug">
            Custom slug <span class="font-normal text-muted-foreground">(optional)</span>
          </Label>
          <InputGroup>
            <InputGroupAddon class="pr-0">
              <InputGroupText class="max-w-40 truncate font-mono text-xs">{host}/</InputGroupText>
            </InputGroupAddon>
            <InputGroupInput
              id="create-slug"
              class="pl-0.5 font-mono"
              autocomplete="off"
              spellcheck={false}
              maxlength={64}
              placeholder="random"
              aria-invalid={slugErr ? true : undefined}
              bind:value={slug}
            />
          </InputGroup>
          {#if slugErr}
            <p class="text-xs text-destructive">{slugErr}</p>
          {/if}
        </div>
        <div class="space-y-1.5">
          <Label for="create-title">
            Title <span class="font-normal text-muted-foreground">(optional)</span>
          </Label>
          <Input
            id="create-title"
            maxlength={255}
            autocomplete="off"
            placeholder="What is this link for?"
            bind:value={title}
          />
        </div>
      </div>
    </form>

    {#if created}
      <div
        class="rise mt-5 flex items-center gap-2 rounded-xl border border-primary/25 bg-primary/5 py-2 pr-2 pl-4"
      >
        <div class="min-w-0 flex-1">
          <p class="text-xs text-muted-foreground">Your short link</p>
          <a
            href={shortUrl(created.slug)}
            target="_blank"
            rel="noopener noreferrer"
            class="block truncate font-mono text-sm font-medium text-primary hover:underline"
          >
            {shortUrlDisplay(created.slug)}
          </a>
        </div>
        <CopyButton text={shortUrl(created.slug)} variant="outline" size="icon" />
        <Button
          variant="outline"
          size="icon"
          href={shortUrl(created.slug)}
          target="_blank"
          rel="noopener noreferrer"
          aria-label="Open short link"
        >
          <ExternalLink />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Dismiss"
          onclick={() => {
            created = null;
            urlInput?.focus();
          }}
        >
          <X />
        </Button>
      </div>
    {/if}
  </CardContent>
</Card>
