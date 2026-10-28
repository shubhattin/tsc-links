<script lang="ts">
  import CopyButton from '$lib/components/links/copy-button.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Switch } from '$lib/components/ui/switch';
  import { Tooltip, TooltipContent, TooltipTrigger } from '$lib/components/ui/tooltip';
  import { shortUrl, shortUrlDisplay, timeAgo, type LinkDto } from '$lib/links-api';
  import CornerDownRight from '@lucide/svelte/icons/corner-down-right';
  import Pencil from '@lucide/svelte/icons/pencil';
  import Trash2 from '@lucide/svelte/icons/trash-2';

  let {
    link,
    toggling = false,
    onToggle,
    onEdit,
    onDelete,
  }: {
    link: LinkDto;
    toggling?: boolean;
    onToggle: (enabled: boolean) => void;
    onEdit: () => void;
    onDelete: () => void;
  } = $props();
</script>

<li class="group flex items-start gap-3 px-4 py-3.5 transition-colors hover:bg-muted/40 sm:px-5">
  <div class={['min-w-0 flex-1 space-y-1', !link.enabled && 'opacity-55']}>
    <div class="flex min-w-0 items-center gap-1.5">
      <a
        href={shortUrl(link.slug)}
        target="_blank"
        rel="noopener noreferrer"
        class="truncate font-mono text-sm font-medium text-primary hover:underline"
        title={shortUrl(link.slug)}
      >
        {shortUrlDisplay(link.slug)}
      </a>
      <CopyButton text={shortUrl(link.slug)} size="icon-xs" />
      {#if !link.enabled}
        <Badge variant="outline" class="shrink-0">Disabled</Badge>
      {/if}
    </div>
    {#if link.title}
      <p class="truncate text-sm">{link.title}</p>
    {/if}
    <div class="flex min-w-0 items-center gap-1 text-xs text-muted-foreground">
      <CornerDownRight class="size-3 shrink-0" />
      <a
        href={link.url}
        target="_blank"
        rel="noopener noreferrer"
        class="truncate hover:text-foreground hover:underline"
        title={link.url}>{link.url}</a
      >
      <span class="shrink-0 before:mx-1 before:content-['·']" title={link.created_at}>
        {timeAgo(link.created_at)}
      </span>
    </div>
  </div>

  <div class="flex shrink-0 items-center gap-0.5 pt-0.5">
    <Tooltip>
      <TooltipTrigger>
        {#snippet child({ props })}
          <span {...props} class="mr-1.5 inline-flex">
            <Switch
              checked={link.enabled}
              disabled={toggling}
              aria-label={link.enabled ? 'Disable link' : 'Enable link'}
              onCheckedChange={onToggle}
            />
          </span>
        {/snippet}
      </TooltipTrigger>
      <TooltipContent
        >{link.enabled ? 'Enabled — redirecting' : 'Disabled — shows 404'}</TooltipContent
      >
    </Tooltip>
    <Button variant="ghost" size="icon-sm" aria-label="Edit link" onclick={onEdit}>
      <Pencil />
    </Button>
    <Button
      variant="ghost"
      size="icon-sm"
      class="hover:bg-destructive/10 hover:text-destructive"
      aria-label="Delete link"
      onclick={onDelete}
    >
      <Trash2 />
    </Button>
  </div>
</li>
