<script lang="ts">
  import type { Session } from '$lib/auth-client';
  import ShieldAlert from '@lucide/svelte/icons/shield-alert';

  let { user }: { user: Session['user'] } = $props();
</script>

<div class="rise mx-auto flex max-w-sm flex-col items-center pt-10 text-center sm:pt-16">
  <span class="grid size-14 place-items-center rounded-2xl bg-muted text-muted-foreground">
    <ShieldAlert class="size-7" />
  </span>
  <h1 class="mt-6 text-xl font-semibold tracking-tight">Admins only</h1>
  <p class="mt-2 text-sm text-muted-foreground">
    You're signed in as <span class="font-medium text-foreground">{user.email}</span>, which doesn't
    have admin access. Ask an admin to grant it, or sign in with another account.
  </p>
</div>
