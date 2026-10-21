<script>
  import Panel from '$lib/components/Panel.svelte';
  import IconButton from '$lib/components/IconButton.svelte';
  import Facts from './Facts.svelte';
  import { formatDateTime, formatRelative } from '$lib/format.js';
  import { failureReasonLabels, recoveryActionLabel } from '$lib/health.js';

  let { sim, refreshing = false, onRefresh } = $props();

  const reasons = $derived(failureReasonLabels(sim?.failure_reasons));
  const facts = $derived([
    { label: 'Failures', value: String(sim?.consecutive_failures ?? 0) },
    {
      label: 'Last probe',
      value: formatRelative(sim?.last_probe_at),
      title: formatDateTime(sim?.last_probe_at)
    },
    {
      label: 'Last OK',
      value: formatRelative(sim?.last_ok_at),
      title: formatDateTime(sim?.last_ok_at)
    },
    {
      label: 'Last recovery',
      value: recoveryActionLabel(sim?.last_recovery_action),
      title: formatDateTime(sim?.last_recovery_at)
    }
  ]);

  const note = $derived(!sim?.health_status ? 'No probe recorded.' : '');
</script>

{#snippet actions()}
  <IconButton
    name="refresh"
    label="Re-check health"
    size={30}
    disabled={refreshing}
    onclick={() => onRefresh?.()}
  />
{/snippet}

<Panel title="Health" {actions}>
  <div class="health">
    {#if reasons.length}
      <ul class="health__reasons">
        {#each reasons as reason (reason)}
          <li class="health__reason">{reason}</li>
        {/each}
      </ul>
    {:else if note}
      <p class="health__note">{note}</p>
    {/if}
    <Facts items={facts} />
  </div>
</Panel>

<style>
  .health {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-width: 0;
  }

  .health__reasons {
    display: flex;
    flex-direction: column;
    gap: 4px;
    list-style: none;
  }

  .health__reason {
    font-size: var(--text-sm);
    color: var(--ink-2);
  }

  .health__note {
    font-size: var(--text-sm);
    color: var(--ink-3);
  }
</style>
