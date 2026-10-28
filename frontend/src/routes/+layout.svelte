<script lang="ts">
  import './layout.css';
  import { Toaster } from '$lib/components/ui/sonner';
  import { TooltipProvider } from '$lib/components/ui/tooltip';
  import { queryClient } from '$lib/query-client';
  import { QueryClientProvider } from '@tanstack/svelte-query';
  import { ModeWatcher } from 'mode-watcher';

  let { children } = $props();
</script>

<svelte:head>
  <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
  <meta name="robots" content="noindex" />
  <title>Links</title>
</svelte:head>

<ModeWatcher />
<QueryClientProvider client={queryClient}>
  <TooltipProvider delayDuration={300}>
    {@render children()}
  </TooltipProvider>
</QueryClientProvider>
<Toaster richColors position="top-center" />
