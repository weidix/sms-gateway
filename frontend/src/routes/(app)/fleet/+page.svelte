<script>
  import { onMount } from 'svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import MetricStrip from '$lib/components/MetricStrip.svelte';
  import Button from '$lib/components/Button.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import SimCardTile from '$lib/fleet/SimCardTile.svelte';
  import { api } from '$lib/api.js';
  import { toastError } from '$lib/stores/toast.svelte.js';

  const REFRESH_MS = 20000;
  const TABS = [
    { value: 'all', label: 'All' },
    { value: 'healthy', label: 'Healthy' },
    { value: 'attention', label: 'Attention' }
  ];

  let sims = $state([]);
  let cards = $state([]);
  let conversations = $state(null);
  let loading = $state(true);
  let refreshing = $state(false);
  let error = $state(null);
  let statusFilter = $state('all');
  let inFlight = false;

  const cardsById = $derived(new Map(cards.map((card) => [card.id, card])));
  const rows = $derived(
    sims.map((sim, index) => ({ sim, card: cardsById.get(sim.sim_id) ?? null, index: index + 1 }))
  );
  const healthyCount = $derived(sims.filter((sim) => sim.health_status === 'healthy').length);
  const attentionCount = $derived(sims.length - healthyCount);
  const unreadCount = $derived(
    conversations
      ? conversations.filter((conversation) => conversation?.sms_preview?.status === 0).length
      : null
  );
  const visibleRows = $derived(
    rows.filter((row) => {
      if (statusFilter === 'healthy') return row.sim.health_status === 'healthy';
      if (statusFilter === 'attention') return row.sim.health_status !== 'healthy';
      return true;
    })
  );

  const metrics = $derived([
    { label: 'Lines', value: loading ? '—' : String(sims.length) },
    { label: 'Healthy', value: loading ? '—' : String(healthyCount) },
    { label: 'Attention', value: loading ? '—' : String(attentionCount) },
    { label: 'Unread', value: unreadCount == null ? '—' : String(unreadCount) }
  ]);

  async function load({ quiet = false, notify = false } = {}) {
    if (inFlight) return;
    inFlight = true;
    if (!quiet) loading = true;

    const hadData = sims.length > 0;
    const [simsResult, cardsResult, conversationsResult] = await Promise.allSettled([
      api.simsInfo(),
      api.simCards(),
      api.conversations()
    ]);

    if (simsResult.status === 'fulfilled') {
      sims = Array.isArray(simsResult.value) ? simsResult.value : [];
      error = null;
    } else {
      if (!hadData) error = simsResult.reason;
      if (notify) toastError(simsResult.reason?.message ?? 'Could not refresh the fleet.');
    }

    if (cardsResult.status === 'fulfilled') {
      cards = Array.isArray(cardsResult.value) ? cardsResult.value : [];
    }
    if (conversationsResult.status === 'fulfilled') {
      conversations = Array.isArray(conversationsResult.value) ? conversationsResult.value : [];
    }

    loading = false;
    inFlight = false;
  }

  async function refresh() {
    if (refreshing) return;
    refreshing = true;
    await load({ quiet: true, notify: true });
    refreshing = false;
  }

  onMount(() => {
    load();

    const timer = setInterval(() => {
      if (document.visibilityState === 'visible') load({ quiet: true });
    }, REFRESH_MS);

    const onVisibilityChange = () => {
      if (document.visibilityState === 'visible') load({ quiet: true });
    };
    document.addEventListener('visibilitychange', onVisibilityChange);

    return () => {
      clearInterval(timer);
      document.removeEventListener('visibilitychange', onVisibilityChange);
    };
  });
</script>

<svelte:head>
  <title>Fleet — SMS Gateway</title>
</svelte:head>

{#snippet headerActions()}
  <Button loading={refreshing} disabled={loading} onclick={refresh}>Refresh</Button>
{/snippet}

<div class="page page--scroll fleet-page">
  <PageHeader title="Fleet" actions={headerActions} />

  <div class="fleet__metrics fleet__metrics--wide">
    <MetricStrip items={metrics} cols={4} />
  </div>
  <div class="fleet__metrics fleet__metrics--narrow">
    <MetricStrip items={metrics.slice(0, 3)} cols={3} />
  </div>

  {#if !loading && rows.length}
    <div class="fleet__tabs" role="group" aria-label="Filter lines by health">
      {#each TABS as tab (tab.value)}
        <button
          type="button"
          class="fleet__tab"
          class:fleet__tab--active={statusFilter === tab.value}
          aria-pressed={statusFilter === tab.value}
          onclick={() => (statusFilter = tab.value)}
        >
          {tab.label}
        </button>
      {/each}
    </div>
  {/if}

  {#if loading}
    <div class="fleet__grid" aria-busy="true" aria-label="Loading lines">
      {#each [1, 2, 3, 4, 5, 6] as skeleton (skeleton)}
        <div class="fleet__skeleton">
          <span class="fleet__skeleton-line fleet__skeleton-line--title"></span>
          <span class="fleet__skeleton-line"></span>
          <span class="fleet__skeleton-line fleet__skeleton-line--id"></span>
        </div>
      {/each}
    </div>
  {:else if error && !sims.length}
    <div class="fleet__state">
      <EmptyState icon="alert" title="Fleet unavailable" description={error.message} />
    </div>
  {:else if !rows.length}
    <div class="fleet__state">
      <EmptyState icon="radio" title="No lines attached" description="Connect a modem, then refresh." />
    </div>
  {:else if !visibleRows.length}
    <div class="fleet__state">
      <EmptyState icon="radio" title="No lines match this filter">
        <Button onclick={() => (statusFilter = 'all')}>Show all</Button>
      </EmptyState>
    </div>
  {:else}
    <div class="fleet__grid">
      {#each visibleRows as row (row.sim.sim_id)}
        <SimCardTile sim={row.sim} card={row.card} index={row.index} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .fleet-page {
    width: 100%;
    max-width: 1500px;
    margin-inline: auto;
  }

  /* The page is a flex column; keep regions at natural height and let the
     scroll container scroll instead of shrinking the header on tall content. */
  .fleet-page > :global(*) {
    flex: none;
  }

  .fleet__metrics--narrow {
    display: none;
  }

  .fleet__tabs {
    display: flex;
    align-items: flex-end;
    justify-content: flex-end;
    gap: 18px;
    border-bottom: 1px solid var(--line);
  }

  .fleet__tab {
    margin-bottom: -1px;
    padding: 8px 10px 10px;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--ink-3);
    font-size: var(--text-sm);
    line-height: 1.2;
    transition:
      color var(--dur) ease,
      border-color var(--dur) ease;
  }

  .fleet__tab:hover {
    color: var(--ink);
  }

  .fleet__tab--active {
    color: var(--ink);
    border-bottom-color: var(--ink);
  }

  .fleet__state {
    display: flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    min-height: 320px;
  }

  .fleet__grid {
    display: grid;
    /* auto-fit keeps a short fleet filling the row, while the raised track
       minimum caps wide screens at three columns. */
    grid-template-columns: repeat(
      auto-fit,
      minmax(min(100%, max(300px, calc((100% - 2 * var(--space-4)) / 3))), 1fr)
    );
    gap: var(--space-4);
    align-items: stretch;
  }

  @container page (max-width: 560px) {
    .fleet__metrics--wide {
      display: none;
    }

    .fleet__metrics--narrow {
      display: block;
    }

    .fleet__tabs {
      justify-content: stretch;
      gap: 0;
    }

    .fleet__tab {
      flex: 1;
      text-align: center;
    }
  }


  .fleet__skeleton {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-height: 116px;
    padding: 13px 15px;
    border: 0;
    border-radius: var(--radius);
    background: var(--raised);
    box-shadow: var(--shadow-card);
  }

  .fleet__skeleton-line {
    display: block;
    height: 10px;
    border-radius: 1px;
    background: var(--sunken);
    animation: fleet-pulse 1.4s ease-in-out infinite;
  }

  .fleet__skeleton-line--title {
    width: 54%;
    height: 13px;
  }

  .fleet__skeleton-line--id {
    width: 38%;
    margin-top: auto;
  }

  @keyframes fleet-pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.55;
    }
  }
</style>
