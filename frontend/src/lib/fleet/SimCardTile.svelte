<script>
  import Badge from '$lib/components/Badge.svelte';
  import SignalBars from '$lib/components/SignalBars.svelte';
  import { simDisplayName, signalLabel } from '$lib/format.js';
  import { healthState } from '$lib/health.js';

  let { sim, card = null, index = 1 } = $props();

  const health = $derived(healthState(sim?.health_status));
  const displayName = $derived(simDisplayName(card, sim?.sim_id ?? ''));
  const rssi = $derived(sim?.signal_quality?.rssi ?? null);
  const operatorName = $derived(
    sim?.operator_info?.operator_name || sim?.operator_info?.operator_id || ''
  );
  const iccid = $derived(card?.id ?? sim?.sim_id ?? '');
</script>

<a
  class="tile"
  href={`/fleet/${encodeURIComponent(sim?.sim_id ?? '')}`}
  title={`${displayName} — ${signalLabel(rssi)}`}
>
  <header class="tile__head">
    <span class="tile__index mono">{String(index).padStart(2, '0')}</span>
    <span class="tile__name truncate">{displayName}</span>
    <Badge tone={health.tone} dot>{health.label}</Badge>
  </header>

  <div class="tile__signal">
    <SignalBars {rssi} size="lg" />
    <span class="tile__rssi mono">{rssi == null ? '—' : `${rssi}/31`}</span>
    {#if operatorName}
      <span class="tile__divider" aria-hidden="true">·</span>
      <span class="tile__operator truncate">{operatorName}</span>
    {/if}
  </div>

  <span class="tile__id mono truncate" title={iccid}>{iccid}</span>
</a>

<style>
  .tile {
    display: flex;
    flex-direction: column;
    gap: 9px;
    min-width: 0;
    min-height: 128px;
    padding: 16px 18px;
    border: 0;
    border-radius: var(--radius);
    background: var(--raised);
    box-shadow: var(--shadow-card);
    transition:
      box-shadow var(--dur) ease,
      transform var(--dur) ease;
  }

  .tile:hover {
    box-shadow: var(--shadow-card-hover);
    transform: translateY(-1px);
  }

  .tile__head {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
  }

  .tile__index {
    font-size: var(--text-2xs);
    letter-spacing: 0.08em;
    color: var(--ink-3);
  }

  .tile__name {
    font-family: var(--font-sans);
    font-size: 15px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--ink);
  }

  .tile__signal {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    font-size: var(--text-sm);
    color: var(--ink-2);
  }

  .tile__rssi {
    flex: none;
    font-size: var(--text-xs);
    color: var(--ink-3);
  }

  .tile__divider {
    flex: none;
    color: var(--line-strong);
  }

  .tile__operator {
    min-width: 0;
  }

  .tile__id {
    margin-top: auto;
    font-size: var(--text-xs);
    color: var(--ink-3);
  }
</style>
