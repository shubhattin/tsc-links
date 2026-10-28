import { QueryClient } from '@tanstack/svelte-query';

export const DEFAULT_STALE_TIME_MS = 15 * 60 * 1000;

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: DEFAULT_STALE_TIME_MS,
    },
  },
});
