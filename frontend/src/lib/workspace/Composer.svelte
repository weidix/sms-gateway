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

  const MAX_EDITOR_HEIGHT = 140;

  const lineOptions = $derived(
    sims.map((sim) => ({ value: sim.id, label: simDisplayName(sim, sim.id) }))
  );
  const lineMissing = $derived(simsState === 'ready' && sims.length === 0);
  const canSend = $derived(Boolean(simId) && !sending && value.trim().length > 0);
  const linePlaceholder = $derived(
    simsState === 'loading' ? 'Loading…' : simsState === 'error' ? 'Lines unavailable' : 'No line'
  );

  let fieldWrap = $state(null);

  /** @param {HTMLTextAreaElement} textarea */
  function fitEditor(textarea) {
    if (!textarea) return;
    textarea.style.height = 'auto';
    textarea.style.height = `${Math.min(textarea.scrollHeight, MAX_EDITOR_HEIGHT)}px`;
  }

  /** @param {Event & { currentTarget: HTMLTextAreaElement }} event */
  function handleInput(event) {
    fitEditor(event.currentTarget);
  }

  // Shrink back after sending or clearing the draft.
  $effect(() => {
    value;
    fitEditor(fieldWrap?.querySelector('textarea') ?? null);
  });

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
  <div class="composer__field" bind:this={fieldWrap}>
    <label class="sr-only" for="composer-message">Message</label>
    <Textarea
      id="composer-message"
      bind:value
      rows={1}
      placeholder="Write a message…"
      onkeydown={handleKeydown}
      oninput={handleInput}
    />
  </div>

  <div class="composer__tools">
    <div class="composer__line">
      <Select
        bind:value={simId}
        options={lineOptions}
        placeholder={linePlaceholder}
        disabled={simsState === 'loading' || sims.length === 0}
        label="Send from line"
      />
    </div>

    <span class="composer__hint">Enter to send · Shift + Enter for a new line</span>

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
    padding: 12px 16px 14px;
    min-width: 0;
  }

  .composer__field {
    min-width: 0;
  }

  /*
   * The editor owns a full row and grows upward as text wraps. There is no
   * manual resize handle: dragging it on a bottom-docked composer is what made
   * the input grow "upwards" unexpectedly. Height is capped and scrolls after
   * the cap.
   */
  .composer__field :global(textarea) {
    display: block;
    width: 100%;
    min-height: 44px !important;
    height: 44px;
    max-height: 140px;
    padding: 10px 14px !important;
    line-height: 1.55 !important;
    resize: none !important;
    overflow-y: auto;
  }

  .composer__tools {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .composer__line {
    flex: 0 0 200px;
    min-width: 0;
  }

  .composer__hint {
    flex: 1 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--muted);
    font-size: var(--text-2xs);
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

  @container page (max-width: 620px) {
    .composer {
      padding: 10px 16px 12px;
    }

    .composer__field :global(textarea) {
      min-height: 56px !important;
    }

    .composer__hint {
      display: none;
    }

    .composer__line {
      flex: 1 1 auto;
    }
  }
</style>
