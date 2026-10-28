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
  import { deleteLink, shortUrlDisplay, type LinkDto } from '$lib/links-api';
  import { removeLinkFromCache } from '$lib/links-cache';
  import { queryClient } from '$lib/query-client';
  import { createMutation } from '@tanstack/svelte-query';
  import { toast } from 'svelte-sonner';

  let { link, onClose }: { link: LinkDto | null; onClose: () => void } = $props();

  const mutation = createMutation(
    () => ({
      mutationFn: (target: LinkDto) => deleteLink(target.id),
      onSuccess: (_: void, target: LinkDto) => {
        removeLinkFromCache(target.id);
        toast.success(`Deleted /${target.slug}`);
        onClose();
      },
      onError: (e: Error) => toast.error('Could not delete link', { description: e.message }),
    }),
    () => queryClient,
  );
</script>

<AlertDialog open={link !== null} onOpenChange={(open) => !open && onClose()}>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Delete this link?</AlertDialogTitle>
      <AlertDialogDescription>
        {#if link}
          <span class="font-mono text-foreground">{shortUrlDisplay(link.slug)}</span> will stop working
          immediately and the slug becomes free to reuse. This can't be undone — disable the link instead
          if you might need it again.
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
          if (link) mutation.mutate(link);
        }}
      >
        {#if mutation.isPending}<Spinner />{/if}
        Delete
      </AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
