<script lang="ts">
  import { signInWith } from '$lib/auth-client';
  import BrandIcon from '$lib/components/auth/brand-icon.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Spinner } from '$lib/components/ui/spinner';
  import Link2 from '@lucide/svelte/icons/link-2';
  import { toast } from 'svelte-sonner';

  let pending = $state<'google' | 'github' | null>(null);

  async function signIn(provider: 'google' | 'github') {
    pending = provider;
    try {
      // Navigates away to the provider on success.
      await signInWith(provider);
    } catch (e) {
      pending = null;
      toast.error('Could not start sign-in', {
        description: e instanceof Error ? e.message : undefined,
      });
    }
  }
</script>

<div class="rise mx-auto flex max-w-sm flex-col items-center pt-10 text-center sm:pt-16">
  <span
    class="grid size-14 place-items-center rounded-2xl bg-primary text-primary-foreground shadow-md"
  >
    <Link2 class="size-7" />
  </span>
  <h1 class="mt-6 text-2xl font-semibold tracking-tight">Short links, managed</h1>
  <p class="mt-2 text-sm text-muted-foreground">
    Sign in with an admin account to create and edit short links.
  </p>
  <div class="mt-8 flex w-full flex-col gap-2.5">
    <Button
      variant="outline"
      size="lg"
      class="h-11 w-full"
      disabled={pending !== null}
      onclick={() => signIn('google')}
    >
      {#if pending === 'google'}<Spinner />{:else}<BrandIcon brand="google" class="size-4" />{/if}
      Continue with Google
    </Button>
    <Button
      variant="outline"
      size="lg"
      class="h-11 w-full"
      disabled={pending !== null}
      onclick={() => signIn('github')}
    >
      {#if pending === 'github'}<Spinner />{:else}<BrandIcon brand="github" class="size-4" />{/if}
      Continue with GitHub
    </Button>
  </div>
</div>
