<script>
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Select from '$lib/components/Select.svelte';
  import Textarea from '$lib/components/Textarea.svelte';
  import { simDisplayName } from '$lib/format.js';

  let {
    sims = [],
    simsState = 'loading',
    simId = $bindable(''),
    value = $bindable(''),
    sending = false,
    onsend,
    onretry
  } = $props();

  const lineOptions = $derived(
    sims.map((sim) => ({ value: sim.id, label: simDisplayName(sim, sim.id) }))
  );
  const lineMissing = $derived(simsState === 'ready' && sims.length === 0);
  const canSend = $derived(Boolean(simId) && !sending && value.trim().length > 0);
  const linePlaceholder = $derived(
    simsState === 'loading' ? 'Loading…' : simsState === 'error' ? 'Lines unavailable' : 'No line'
  );

  /** @param {KeyboardEvent} event */
  function handleKeydown(event) {
    if (event.key !== 'Enter' || event.shiftKey || event.isComposing || event.keyCode === 229) {
      return;
    }
    if (!canSend) return;
    event.preventDefault();
    onsend?.();
  }
</script>

<div class="composer">
  <div class="composer__row">
    <div class="composer__line">
      <Select
        bind:value={simId}
        options={lineOptions}
        placeholder={linePlaceholder}
        disabled={simsState === 'loading' || sims.length === 0}
        label="Send from line"
      />
    </div>

    <label class="sr-only" for="composer-message">Message</label>
    <div class="composer__field">
      <Textarea
        id="composer-message"
        bind:value
        rows={2}
        placeholder="Write a message…"
        onkeydown={handleKeydown}
      />
    </div>

    <div class="composer__send">
      <Button
        variant="primary"
        onclick={() => onsend?.()}
        loading={sending}
        disabled={!canSend}
        ariaLabel="Send message"
      >
        <Icon name="send" size={14} />
        <span class="composer__send-label">Send</span>
      </Button>
    </div>
  </div>

  {#if simsState === 'error' || lineMissing}
    <p class="composer__note">
      {#if simsState === 'error'}
        Lines unavailable.
        <button type="button" class="composer__retry" onclick={() => onretry?.()}>Retry</button>
      {:else}
        No line registered.
      {/if}
    </p>
  {/if}
</div>

<style>
  .composer {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 14px 16px;
    min-width: 0;
  }

  .composer__row {
    display: grid;
    grid-template-columns: 152px minmax(0, 1fr) auto;
    align-items: end;
    gap: 10px;
    min-width: 0;
  }

  .composer__line,
  .composer__field {
    min-width: 0;
  }

  .composer__note {
    display: flex;
    align-items: center;
    gap: 5px;
    margin: 0;
    color: var(--ink-3);
    font-size: var(--text-2xs);
    line-height: 1.4;
  }

  .composer__retry {
    padding: 0;
    border: 0;
    background: none;
    color: var(--ink-2);
    font-size: inherit;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .composer__retry:hover {
    color: var(--ink);
  }

  .composer__send {
    display: flex;
    align-items: center;
    flex: none;
  }

  .composer__send :global(.btn) {
    height: 36px;
    min-width: 84px;
  }

  @container page (max-width: 620px) {
    .composer {
      padding: 10px 16px;
    }

    .composer__row {
      grid-template-columns: minmax(0, 1fr) auto;
      grid-template-areas:
        'field field'
        'line send';
      align-items: center;
      gap: 8px;
    }

    .composer__field {
      grid-area: field;
    }

    .composer__line {
      grid-area: line;
    }

    .composer__send {
      grid-area: send;
    }
  }
</style>
