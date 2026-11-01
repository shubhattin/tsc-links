<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { monthLabel, shiftMonth } from '$lib/expenses-api';
  import Calendar from '@lucide/svelte/icons/calendar';
  import ChevronLeft from '@lucide/svelte/icons/chevron-left';
  import ChevronRight from '@lucide/svelte/icons/chevron-right';

  let { month, onChange }: { month: string; onChange: (month: string) => void } = $props();

  const today = new Date();
  const thisMonth = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}`;
</script>

<div class="flex items-center gap-1">
  <Button
    variant="ghost"
    size="icon"
    aria-label="Previous month"
    onclick={() => onChange(shiftMonth(month, -1))}
  >
    <ChevronLeft />
  </Button>
  <span
    class="flex items-center gap-2 rounded-lg px-1 py-1 font-medium tabular-nums"
    aria-live="polite"
  >
    <Calendar class="size-4 text-muted-foreground" />
    <span class="min-w-32 text-center text-sm sm:text-base">{monthLabel(month)}</span>
  </span>
  <Button
    variant="ghost"
    size="icon"
    aria-label="Next month"
    disabled={month >= thisMonth}
    onclick={() => onChange(shiftMonth(month, 1))}
  >
    <ChevronRight />
  </Button>
</div>
