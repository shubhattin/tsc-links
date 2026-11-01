<script lang="ts">
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '$lib/components/ui/alert-dialog';
  import { buttonVariants } from '$lib/components/ui/button';
  import { Spinner } from '$lib/components/ui/spinner';
  import {
    deleteSource,
    EXPENSE_SOURCES_QUERY_KEY,
    type ExpenseSourceDto,
  } from '$lib/expenses-api';
  import { queryClient } from '$lib/query-client';
  import { createMutation } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  let { source, onClose }: { source: ExpenseSourceDto | null; onClose: () => void } = $props();

  const mutation = createMutation(
    () => ({
      mutationFn: (target: ExpenseSourceDto) => deleteSource(target.key),
      onSuccess: async (_: void, target: ExpenseSourceDto) => {
        await queryClient.invalidateQueries({ queryKey: EXPENSE_SOURCES_QUERY_KEY });
        toast.success(`Deleted ${target.name}`);
        onClose();
      },
      onError: (e: Error) => toast.error('Could not delete source', { description: e.message }),
    }),
    () => queryClient,
  );
</script>

<AlertDialog open={source !== null} onOpenChange={(open) => !open && onClose()}>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Delete this source?</AlertDialogTitle>
      <AlertDialogDescription>
        {#if source}
          <span class="font-medium break-words text-foreground">{source.name}</span> will be removed
          for good and its key
          <span class="font-mono text-foreground">{source.key}</span> becomes free to reuse. This can't
          be undone.
        {/if}
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel disabled={mutation.isPending}>Cancel</AlertDialogCancel>
      <AlertDialogAction
        class={buttonVariants({ variant: 'destructive' })}
        disabled={mutation.isPending}
        onclick={(e: MouseEvent) => {
          // Keep the dialog open until the request settles.
          e.preventDefault();
          if (source) mutation.mutate(source);
        }}
      >
        {#if mutation.isPending}<Spinner />{/if}
        Delete
      </AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
