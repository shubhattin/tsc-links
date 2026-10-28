<script lang="ts">
  import { Button, type ButtonSize, type ButtonVariant } from '$lib/components/ui/button';
  import { Tooltip, TooltipContent, TooltipTrigger } from '$lib/components/ui/tooltip';
  import { UseClipboard } from '$lib/hooks/use-clipboard.svelte';
  import Check from '@lucide/svelte/icons/check';
  import Copy from '@lucide/svelte/icons/copy';

  let {
    text,
    label = 'Copy link',
    variant = 'ghost',
    size = 'icon-sm',
  }: { text: string; label?: string; variant?: ButtonVariant; size?: ButtonSize } = $props();

  const clipboard = new UseClipboard({ delay: 1500 });
</script>

<Tooltip>
  <TooltipTrigger>
    {#snippet child({ props })}
      <Button {...props} {variant} {size} aria-label={label} onclick={() => clipboard.copy(text)}>
        {#if clipboard.copied}
          <Check class="text-emerald-600 dark:text-emerald-400" />
        {:else}
          <Copy />
        {/if}
      </Button>
    {/snippet}
  </TooltipTrigger>
  <TooltipContent>{clipboard.copied ? 'Copied!' : label}</TooltipContent>
</Tooltip>
