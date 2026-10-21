<script>
  import Panel from '$lib/components/Panel.svelte';
  import Select from '$lib/components/Select.svelte';
  import IconButton from '$lib/components/IconButton.svelte';
  import Button from '$lib/components/Button.svelte';
  import UsageBar from './UsageBar.svelte';
  import { api } from '$lib/api.js';
  import { parseStorageStatus } from '$lib/format.js';
  import { toastError, toastOk } from '$lib/stores/toast.svelte.js';

  let { simId, status = '', loading = false, error = null, onReload } = $props();

  const STORAGE_OPTIONS = [
    { value: 'SIM', label: 'SIM card' },
    { value: 'ME', label: 'Modem memory' },
    { value: 'MT', label: 'Modem and SIM' }
  ];

  const STORAGE_CODES = { SM: 'SIM', SIM: 'SIM', ME: 'ME', MT: 'MT' };

  const parsed = $derived(parseStorageStatus(status));
  const currentStorage = $derived(
    parsed?.read?.storage
      ? (STORAGE_CODES[String(parsed.read.storage).toUpperCase()] ?? '')
      : ''
  );

  let selected = $state('SIM');
  let applying = $state(false);

  $effect(() => {
    if (currentStorage) selected = currentStorage;
  });

  async function applyStorage() {
    if (applying) return;
    applying = true;
    try {
      await api.setStorage(simId, selected);
      toastOk(`Storage set to ${selected}.`);
      await onReload?.();
    } catch (cause) {
      toastError(cause?.message ?? 'Could not change the storage location.');
    } finally {
      applying = false;
    }
  }
</script>

{#snippet actions()}
  <IconButton
    name="refresh"
    label="Re-read storage usage"
    size={30}
    disabled={loading || applying}
    onclick={() => onReload?.()}
  />
{/snippet}

<Panel title="Storage" {actions}>
  <div class="storage">
    {#if loading && !parsed && !status}
      <div class="storage__skeleton" aria-busy="true" aria-label="Loading storage usage">
        {#each [1, 2, 3] as row (row)}
          <span class="storage__skeleton-row"></span>
        {/each}
      </div>
    {:else if parsed}
      <div class="storage__usage">
        <UsageBar label="Read" used={parsed.read.used} total={parsed.read.total} storage={parsed.read.storage} />
        <UsageBar label="Write" used={parsed.write.used} total={parsed.write.total} storage={parsed.write.storage} />
        <UsageBar
          label="Receive"
          used={parsed.receive.used}
          total={parsed.receive.total}
          storage={parsed.receive.storage}
        />
      </div>
    {:else}
      <p class="storage__note" title={error?.message ?? undefined}>
        {error ? 'Usage unavailable.' : 'No usage reported.'}
      </p>
    {/if}

    <div class="storage__control">
      <span class="storage__label">Location</span>
      <div class="storage__select">
        <Select bind:value={selected} options={STORAGE_OPTIONS} label="SMS storage location" disabled={applying} />
      </div>
      <Button loading={applying} disabled={applying} onclick={applyStorage}>Apply</Button>
    </div>
  </div>
</Panel>

<style>
  .storage {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-width: 0;
  }

  .storage__usage {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .storage__note {
    font-size: var(--text-sm);
    color: var(--ink-3);
  }

  .storage__skeleton {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .storage__skeleton-row {
    display: block;
    height: 12px;
    border-radius: 1px;
    background: var(--sunken);
    animation: storage-pulse 1.4s ease-in-out infinite;
  }

  .storage__skeleton-row:nth-child(2) {
    width: 82%;
  }

  .storage__skeleton-row:nth-child(3) {
    width: 64%;
  }

  .storage__control {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .storage__label {
    font-size: var(--text-xs);
    color: var(--ink-3);
    white-space: nowrap;
  }

  .storage__select {
    min-width: 0;
  }

  @keyframes storage-pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.55;
    }
  }
</style>
