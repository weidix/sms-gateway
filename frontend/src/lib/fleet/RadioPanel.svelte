<script>
  import Panel from '$lib/components/Panel.svelte';
  import Button from '$lib/components/Button.svelte';
  import Facts from './Facts.svelte';
  import { api } from '$lib/api.js';
  import { toastError, toastOk } from '$lib/stores/toast.svelte.js';

  let { simId, airplaneMode = false, onReload } = $props();

  let applying = $state(false);

  const facts = $derived([
    { label: 'Radio', value: airplaneMode ? 'Off' : 'On' }
  ]);

  async function toggle() {
    if (applying) return;
    applying = true;
    try {
      await api.setAirplaneMode(simId, !airplaneMode);
      toastOk(airplaneMode ? 'Line enabled.' : 'Line disabled.');
      await onReload?.();
    } catch (cause) {
      toastError(cause?.message ?? 'Could not change airplane mode.');
    } finally {
      applying = false;
    }
  }
</script>

<Panel title="Airplane mode" description="Stops this SIM from registering with the network.">
  <div class="airplane">
    <Facts items={facts} />
    {#if airplaneMode}
      <p class="airplane__note">
        SMS polling and health checks stay paused until the line is enabled.
      </p>
    {/if}
    <div class="airplane__control">
      <Button
        variant={airplaneMode ? 'primary' : 'danger'}
        loading={applying}
        disabled={applying}
        onclick={toggle}
      >
        {airplaneMode ? 'Enable line' : 'Disable line'}
      </Button>
    </div>
  </div>
</Panel>

<style>
  .airplane {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
  }

  .airplane__note {
    font-size: var(--text-xs);
    line-height: 1.45;
    color: var(--ink-3);
  }

  .airplane__control {
    display: flex;
    justify-content: flex-end;
  }
</style>
