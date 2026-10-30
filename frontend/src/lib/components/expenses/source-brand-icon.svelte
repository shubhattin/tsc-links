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
  import type { Snippet } from 'svelte';

  let {
    sourceKey,
    size = 'md',
    children,
  }: { sourceKey: string; size?: 'md' | 'lg'; children?: Snippet } = $props();

  const brand = $derived(SOURCE_BRANDS[sourceKey]);
</script>

{#if brand}
  <span
    aria-hidden="true"
    class={[
      'grid shrink-0 place-items-center overflow-hidden rounded-xl border',
      size === 'lg' ? 'size-11' : 'size-10',
    ]}
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
      class={size === 'lg' ? 'size-5.5' : 'size-5'}
      fill="currentColor"
    >
      <path d={brand.path} />
    </svg>
  </span>
{:else}
  <span
    aria-hidden="true"
    class={[
      'grid shrink-0 place-items-center overflow-hidden rounded-xl border border-border bg-muted',
      size === 'lg' ? 'size-11' : 'size-10',
    ]}
  >
    {@render children?.()}
  </span>
{/if}
