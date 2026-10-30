<script lang="ts">
  /**
   * Brand glyph for an expense source.
   *
   * Simple-Icons paths are monochrome, so color comes from each brand's
   * official color on a soft tinted tile — much more recognizable than a
   * black/white glyph. Vercel's mark is pure black/white by design, and
   * unknown/custom sources fall back to `children` (usually a Receipt icon).
   */
  import { SOURCE_BRANDS } from '$lib/expenses-api';
  import IndianRupee from '@lucide/svelte/icons/indian-rupee';
  import type { Snippet } from 'svelte';

  let {
    sourceKey,
    size = 'md',
    children,
  }: { sourceKey: string; size?: 'sm' | 'md' | 'lg'; children?: Snippet } = $props();

  const brand = $derived(SOURCE_BRANDS[sourceKey]);

  const tileSize = $derived(size === 'lg' ? 'size-11' : size === 'sm' ? 'size-7' : 'size-10');
  const glyphSize = $derived(size === 'lg' ? 'size-5.5' : size === 'sm' ? 'size-4' : 'size-5');
</script>

{#if brand}
  <span
    aria-hidden="true"
    class={['grid shrink-0 place-items-center overflow-hidden rounded-xl border', tileSize]}
    style={brand.color
      ? `color: ${brand.color}; border-color: ${brand.color}33; background-color: ${brand.color}1a;`
      : undefined}
    class:bg-muted={brand.color == null}
    class:border-border={brand.color == null}
  >
    <svg
      role="img"
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
      class={glyphSize}
      fill="currentColor"
    >
      <path d={brand.path} />
    </svg>
  </span>
{:else if sourceKey === 'misc'}
  <!-- Catch-all bucket: rupee glyph on an emerald tile. -->
  <span
    aria-hidden="true"
    class={[
      'grid shrink-0 place-items-center overflow-hidden rounded-xl border border-emerald-500/20 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400',
      tileSize,
    ]}
  >
    <IndianRupee class={glyphSize} />
  </span>
{:else}
  <span
    aria-hidden="true"
    class={[
      'grid shrink-0 place-items-center overflow-hidden rounded-xl border border-border bg-muted',
      tileSize,
    ]}
  >
    {@render children?.()}
  </span>
{/if}
