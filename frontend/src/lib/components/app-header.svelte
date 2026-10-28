<script lang="ts">
  import { authClient, isAdmin, type Session } from '$lib/auth-client';
  import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
  import { Button } from '$lib/components/ui/button';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu';
  import { queryClient } from '$lib/query-client';
  import Link2 from '@lucide/svelte/icons/link-2';
  import LogOut from '@lucide/svelte/icons/log-out';
  import Monitor from '@lucide/svelte/icons/monitor';
  import Moon from '@lucide/svelte/icons/moon';
  import Sun from '@lucide/svelte/icons/sun';
  import { setMode, userPrefersMode } from 'mode-watcher';
  import { toast } from 'svelte-sonner';

  let { user }: { user: Session['user'] | null } = $props();

  let signingOut = $state(false);

  const initials = $derived(
    (user?.name ?? user?.email ?? '?')
      .split(/\s+/)
      .map((p) => p[0])
      .join('')
      .slice(0, 2)
      .toUpperCase(),
  );

  async function signOut() {
    signingOut = true;
    try {
      const { error } = await authClient.signOut();
      if (error) throw new Error(error.message);
      queryClient.clear();
    } catch (e) {
      toast.error('Sign-out failed', { description: e instanceof Error ? e.message : undefined });
    } finally {
      signingOut = false;
    }
  }
</script>

<header class="sticky top-0 z-30 border-b bg-background/80 backdrop-blur">
  <div class="mx-auto flex h-14 w-full max-w-3xl items-center justify-between px-4">
    <a href="/" class="flex items-center gap-2 font-semibold tracking-tight">
      <span
        class="grid size-7 place-items-center rounded-lg bg-primary text-primary-foreground shadow-sm"
      >
        <Link2 class="size-4" />
      </span>
      Links
    </a>
    <div class="flex items-center gap-1">
      <DropdownMenu>
        <DropdownMenuTrigger>
          {#snippet child({ props })}
            <Button {...props} variant="ghost" size="icon" aria-label="Theme">
              <Sun class="hidden dark:block" />
              <Moon class="dark:hidden" />
            </Button>
          {/snippet}
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem onSelect={() => setMode('system')}>
            <Monitor /> System{userPrefersMode.current === 'system' ? ' ✓' : ''}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => setMode('light')}>
            <Sun /> Light{userPrefersMode.current === 'light' ? ' ✓' : ''}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={() => setMode('dark')}>
            <Moon /> Dark{userPrefersMode.current === 'dark' ? ' ✓' : ''}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      {#if user}
        <DropdownMenu>
          <DropdownMenuTrigger>
            {#snippet child({ props })}
              <button {...props} class="ml-1 rounded-full" aria-label="Account">
                <Avatar class="size-8">
                  {#if user.image}
                    <AvatarImage src={user.image} alt={user.name} />
                  {/if}
                  <AvatarFallback class="text-xs">{initials}</AvatarFallback>
                </Avatar>
              </button>
            {/snippet}
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" class="w-60">
            <DropdownMenuLabel class="flex flex-col gap-0.5 font-normal">
              <span class="truncate text-sm font-medium text-foreground">{user.name}</span>
              <span class="truncate text-xs text-muted-foreground">{user.email}</span>
              {#if isAdmin(user)}
                <span class="mt-1 text-[11px] font-medium tracking-wide text-primary uppercase">
                  Admin
                </span>
              {/if}
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem disabled={signingOut} onSelect={signOut}>
              <LogOut /> Sign out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      {/if}
    </div>
  </div>
</header>
