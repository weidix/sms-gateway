<script>
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import Button from '$lib/components/Button.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import SignalBars from '$lib/components/SignalBars.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import HealthPanel from '$lib/fleet/HealthPanel.svelte';
  import StoragePanel from '$lib/fleet/StoragePanel.svelte';
  import DevicePanel from '$lib/fleet/DevicePanel.svelte';
  import AtConsole from '$lib/fleet/AtConsole.svelte';
  import EditIdentityDialog from '$lib/fleet/EditIdentityDialog.svelte';
  import { api } from '$lib/api.js';
  import { simDisplayName } from '$lib/format.js';
  import { healthState } from '$lib/health.js';
  import { toastError, toastOk } from '$lib/stores/toast.svelte.js';

  const REFRESH_MS = 20000;

  let loadedId = $state('');
  let sim = $state(null);
  let cards = $state([]);
  let storage = $state(null);
  let loading = $state(true);
  let error = $state(null);
  let notFound = $state(false);
  let storageLoading = $state(false);
  let storageError = $state(null);
  let refreshingHealth = $state(false);
  let refreshingInbox = $state(false);
  let editOpen = $state(false);

  const card = $derived(cards.find((item) => item.id === loadedId) ?? null);
  const displayName = $derived(simDisplayName(card, loadedId));
  const iccid = $derived(card?.id ?? sim?.sim_id ?? '');
  const health = $derived(healthState(sim?.health_status));
  const rssi = $derived(sim?.signal_quality?.rssi ?? null);

  async function loadLine(id) {
    loading = true;
    error = null;
    notFound = false;
    storageLoading = true;
    storageError = null;

    const [infoResult, cardsResult, storageResult] = await Promise.allSettled([
      api.simInfo(id),
      api.simCards(),
      api.storageStatus(id)
    ]);

    if (id !== loadedId) return;

    if (infoResult.status === 'rejected') {
      if (infoResult.reason?.status === 404) notFound = true;
      else error = infoResult.reason;
      loading = false;
      storageLoading = false;
      return;
    }

    sim = infoResult.value;
    if (cardsResult.status === 'fulfilled' && Array.isArray(cardsResult.value)) {
      cards = cardsResult.value;
    }
    if (storageResult.status === 'fulfilled') {
      storage = storageResult.value;
    } else {
      storageError = storageResult.reason;
    }
    storageLoading = false;
    loading = false;
  }

  async function refreshHealth({ quiet = false } = {}) {
    const id = loadedId;
    if (!id || refreshingHealth) return;
    refreshingHealth = true;
    try {
      const next = await api.simInfo(id);
      if (id === loadedId) sim = next;
    } catch (cause) {
      if (cause?.status === 404) {
        notFound = true;
        sim = null;
        cards = [];
      } else if (!quiet) {
        toastError(cause?.message ?? 'Could not refresh health.');
      }
    } finally {
      refreshingHealth = false;
    }
  }

  async function refreshStorage() {
    const id = loadedId;
    if (!id) return;
    storageLoading = true;
    try {
      const next = await api.storageStatus(id);
      if (id === loadedId) {
        storage = next;
        storageError = null;
      }
    } catch (cause) {
      if (id === loadedId) storageError = cause;
    } finally {
      storageLoading = false;
    }
  }

  async function reloadStoragePanel() {
    await Promise.allSettled([refreshStorage(), refreshHealth({ quiet: true })]);
  }

  async function reloadCards() {
    try {
      const next = await api.simCards();
      if (Array.isArray(next)) cards = next;
    } catch {
      // The previous card record stays on screen.
    }
  }

  async function refreshInbox() {
    const id = loadedId;
    if (!id || refreshingInbox) return;
    refreshingInbox = true;
    try {
      await api.refreshSim(id);
      toastOk('Inbox refreshed.');
      await Promise.allSettled([refreshHealth({ quiet: true }), refreshStorage(), reloadCards()]);
    } catch (cause) {
      toastError(cause?.message ?? 'Could not refresh the inbox.');
    } finally {
      refreshingInbox = false;
    }
  }

  $effect(() => {
    const id = page.params.simId ?? '';
    if (!id || id === loadedId) return;
    loadedId = id;
    sim = null;
    cards = [];
    storage = null;
    error = null;
    notFound = false;
    loadLine(id);
  });

  onMount(() => {
    const timer = setInterval(() => {
      if (document.visibilityState === 'visible') refreshHealth({ quiet: true });
    }, REFRESH_MS);

    const onVisibilityChange = () => {
      if (document.visibilityState === 'visible') refreshHealth({ quiet: true });
    };
    document.addEventListener('visibilitychange', onVisibilityChange);

    return () => {
      clearInterval(timer);
      document.removeEventListener('visibilitychange', onVisibilityChange);
    };
  });
</script>

<svelte:head>
  <title>{displayName || loadedId || 'Line'} — SMS Gateway</title>
</svelte:head>

{#snippet headerMeta()}
  <span class="mono">{iccid}</span>
  <Badge tone={health.tone} dot>{health.label}</Badge>
  <span class="detail__signal">
    <SignalBars {rssi} size="md" />
    <span class="mono">{rssi == null ? '—' : `${rssi}/31`}</span>
  </span>
{/snippet}

{#snippet headerActions()}
  <Button loading={refreshingInbox} disabled={refreshingInbox} onclick={refreshInbox}>
    Refresh inbox
  </Button>
  <Button variant="primary" onclick={() => (editOpen = true)}>Edit identity</Button>
{/snippet}

<div class="page page--scroll detail-page">
  <div class="detail__top">
    <a class="back" href="/fleet">
      <Icon name="arrow-left" size={13} />
      <span>Fleet</span>
    </a>
    {#if sim}
      <PageHeader title={displayName} meta={headerMeta} actions={headerActions} />
    {:else if loading}
      <div class="detail__head-skeleton" aria-busy="true" aria-label="Loading line">
        <span class="sk sk--title"></span>
        <span class="sk sk--meta"></span>
      </div>
    {/if}
  </div>

  {#if notFound}
    <div class="detail__state">
      <EmptyState icon="radio" title="Line not found" />
    </div>
  {:else if error}
    <div class="detail__state">
      <EmptyState icon="alert" title="Line unavailable" description={error.message}>
        <Button onclick={() => loadLine(loadedId)}>Retry</Button>
      </EmptyState>
    </div>
  {:else if loading && !sim}
    <div class="detail__body" aria-busy="true" aria-label="Loading line">
      <div class="detail__side">
        {#each [1, 2, 3] as block (block)}
          <div class="sk sk--panel"></div>
        {/each}
      </div>
      <div class="sk sk--console"></div>
    </div>
  {:else if sim}
    <div class="detail__body">
      <div class="detail__side">
        <HealthPanel {sim} refreshing={refreshingHealth} onRefresh={refreshHealth} />
        <StoragePanel
          simId={loadedId}
          status={storage?.status ?? ''}
          loading={storageLoading}
          error={storageError}
          onReload={reloadStoragePanel}
        />
        <DevicePanel {sim} {card} />
      </div>
      <div class="detail__console">
        {#key loadedId}
          <AtConsole simId={loadedId} />
        {/key}
      </div>
    </div>
  {/if}
</div>

<EditIdentityDialog
  bind:open={editOpen}
  simId={loadedId}
  alias={card?.alias ?? ''}
  phone={card?.phone_number ?? ''}
  onSaved={reloadCards}
/>

<style>
  .detail-page {
    width: 100%;
    max-width: 1500px;
    margin-inline: auto;
  }

  /* The page is a flex column; keep regions at natural height and let the
     scroll container scroll instead of shrinking the header on tall content. */
  .detail-page > :global(*) {
    flex: none;
  }

  .detail__top {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
  }

  .back {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    align-self: flex-start;
    margin: -6px -8px;
    padding: 6px 8px;
    color: var(--ink-3);
    font-size: var(--text-sm);
    transition: color var(--dur) ease;
  }

  .back:hover {
    color: var(--ink);
  }

  .detail__state {
    display: flex;
    flex: 1;
    align-items: center;
    justify-content: center;
    min-height: 0;
  }

  .detail__signal {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .detail__body {
    display: grid;
    grid-template-columns: minmax(0, 5fr) minmax(0, 7fr);
    gap: var(--space-4);
    align-items: stretch;
    min-width: 0;
  }

  .detail__side {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-width: 0;
  }

  .detail__console {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .detail__head-skeleton {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--line);
  }

  .sk {
    display: block;
    border-radius: 1px;
    background: var(--sunken);
    animation: detail-pulse 1.4s ease-in-out infinite;
  }

  .sk--title {
    width: 220px;
    max-width: 60%;
    height: 22px;
  }

  .sk--meta {
    width: 340px;
    max-width: 80%;
    height: 11px;
  }

  .sk--panel {
    height: 180px;
  }

  .sk--panel:nth-child(2) {
    height: 240px;
  }

  .sk--panel:nth-child(3) {
    height: 210px;
  }

  .sk--console {
    height: 700px;
  }

  @keyframes detail-pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.55;
    }
  }

  @container page (max-width: 980px) {
    .detail__body {
      grid-template-columns: minmax(0, 1fr);
    }

    .sk--console {
      height: 460px;
    }
  }

  @container page (max-width: 760px) {
    .detail-page :global(.page-header) {
      flex-direction: column;
      align-items: stretch;
      gap: var(--space-3);
    }

    .detail-page :global(.page-header__actions) {
      display: flex;
      width: 100%;
      justify-content: stretch;
      gap: 8px;
    }

    .detail-page :global(.page-header__actions) :global(.btn) {
      flex: 1 1 0;
    }

    .detail-page :global(.page-header__actions) {
      flex-wrap: wrap;
    }
  }
</style>
