<script lang="ts">
  /**
   * Shadcn date picker bound to a `YYYY-MM-DD` string (what the API speaks).
   * Future dates are disabled — expenses record costs already charged.
   */
  import { Button } from '$lib/components/ui/button';
  import * as Calendar from '$lib/components/ui/calendar';
  import { Label } from '$lib/components/ui/label';
  import * as Popover from '$lib/components/ui/popover';
  import { cn } from '$lib/utils.js';
  import {
    DateFormatter,
    getLocalTimeZone,
    parseDate,
    today,
    type DateValue,
  } from '@internationalized/date';
  import CalendarIcon from '@lucide/svelte/icons/calendar';

  let { value = $bindable(''), id }: { value: string; id: string } = $props();

  const df = new DateFormatter('en-IN', { dateStyle: 'medium' });
  const maxDate = today(getLocalTimeZone());
  let open = $state(false);
  let picked = $state<DateValue | undefined>(undefined);
  // Last string pushed in either direction — distinguishes a parent reset
  // from the user picking a day (without this guard the sync below would
  // revert the user's pick before it propagates).
  let lastSynced = $state(value);

  function parseOrUndefined(raw: string): DateValue | undefined {
    try {
      return parseDate(raw);
    } catch {
      return undefined;
    }
  }

  // Sync the calendar selection when the parent value changes (e.g. dialog reset).
  $effect.pre(() => {
    if (value !== lastSynced) {
      lastSynced = value;
      picked = value ? parseOrUndefined(value) : undefined;
    }
  });

  // Push calendar picks back to the parent as `YYYY-MM-DD` and close.
  $effect(() => {
    if (picked && picked.toString() !== value) {
      lastSynced = picked.toString();
      value = lastSynced;
      open = false;
    }
  });

  const display = $derived.by(() => {
    const parsed = value ? parseOrUndefined(value) : undefined;
    return parsed ? df.format(parsed.toDate(getLocalTimeZone())) : 'Pick a date';
  });
</script>

<div class="space-y-1.5">
  <Label for={id}>Charge date</Label>
  <Popover.Root bind:open>
    <Popover.Trigger {id}>
      {#snippet child({ props })}
        <Button
          variant="outline"
          {...props}
          class={cn(
            'w-full justify-start rounded-xl px-3 text-left font-normal',
            !value && 'text-muted-foreground',
          )}
        >
          <CalendarIcon class="mr-2 size-4 shrink-0" />
          {display}
        </Button>
      {/snippet}
    </Popover.Trigger>
    <Popover.Content class="w-auto p-0" align="start">
      <Calendar.Calendar
        type="single"
        bind:value={picked}
        bind:placeholder={picked}
        maxValue={maxDate}
        captionLayout="dropdown"
      />
    </Popover.Content>
  </Popover.Root>
</div>
