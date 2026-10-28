<script lang="ts">
  import { authClient, isAdmin } from '$lib/auth-client';
  import AppHeader from '$lib/components/app-header.svelte';
  import NoAccess from '$lib/components/auth/no-access.svelte';
  import SignInCard from '$lib/components/auth/sign-in-card.svelte';
  import CreateLinkCard from '$lib/components/links/create-link-card.svelte';
  import LinkList from '$lib/components/links/link-list.svelte';
  import { Skeleton } from '$lib/components/ui/skeleton';

  // Better Auth session store (deduped fetch, refetches on focus / sign-in / sign-out).
  // UX gate only — the Rust API re-checks the session cookie on every request.
  const session = authClient.useSession();
  const user = $derived($session.data?.user ?? null);
</script>

<div class="flex min-h-svh flex-col">
  <AppHeader {user} />
  <main class="mx-auto w-full max-w-3xl flex-1 px-4 py-8 sm:py-12">
    {#if $session.isPending && !$session.data}
      <div class="space-y-6" aria-busy="true">
        <Skeleton class="h-44 w-full rounded-2xl" />
        <Skeleton class="h-72 w-full rounded-2xl" />
      </div>
    {:else if !user}
      <SignInCard />
    {:else if !isAdmin(user)}
      <NoAccess {user} />
    {:else}
      <div class="space-y-8">
        <CreateLinkCard />
        <LinkList />
      </div>
    {/if}
  </main>
</div>
