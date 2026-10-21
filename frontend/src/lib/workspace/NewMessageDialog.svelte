<script>
  import Dialog from '$lib/components/Dialog.svelte';
  import Button from '$lib/components/Button.svelte';
  import Field from '$lib/components/Field.svelte';
  import Input from '$lib/components/Input.svelte';
  import Select from '$lib/components/Select.svelte';
  import Textarea from '$lib/components/Textarea.svelte';
  import { simDisplayName } from '$lib/format.js';

  let {
    open = $bindable(false),
    sims = [],
    simsState = 'loading',
    defaultSimId = '',
    busy = false,
    error = '',
    onsubmit
  } = $props();

  let contact = $state('');
  let simId = $state('');
  let message = $state('');
  let formError = $state('');
  let wasOpen = false;

  const lineOptions = $derived(
    sims.map((sim) => ({ value: sim.id, label: simDisplayName(sim, sim.id) }))
  );
  const canSend = $derived(
    contact.trim().length > 0 && message.trim().length > 0 && Boolean(simId) && !busy
  );

  $effect(() => {
    if (open && !wasOpen) {
      contact = '';
      message = '';
      formError = '';
      simId = defaultSimId || sims[0]?.id || '';
    }
    wasOpen = open;
  });

  $effect(() => {
    const fallback = defaultSimId || sims[0]?.id || '';
    if (open && !simId && fallback) simId = fallback;
  });

  /** @param {HTMLElement} node */
  function focusContact(node) {
    node.querySelector('input')?.focus();
    return {};
  }

  /** @param {SubmitEvent} [event] */
  async function submit(event) {
    event?.preventDefault();
    if (busy) return;
    formError = '';
    if (!contact.trim()) {
      formError = 'Enter a contact.';
      return;
    }
    if (!simId) {
      formError = 'Choose a line.';
      return;
    }
    if (!message.trim()) {
      formError = 'Enter a message.';
      return;
    }
    await onsubmit?.({
      contact: contact.trim(),
      simId,
      message: message.trim()
    });
  }
</script>

<Dialog bind:open title="New message" size="md">
  <form class="new" use:focusContact onsubmit={submit}>
    <Field label="Contact" forId="new-contact" hint="Phone number or saved contact.">
      <Input
        id="new-contact"
        bind:value={contact}
        placeholder="+8613800138000"
        disabled={busy}
        autocomplete="off"
      />
    </Field>

    <Field label="Line" forId="new-line">
      <Select
        id="new-line"
        bind:value={simId}
        options={lineOptions}
        placeholder={simsState === 'loading' ? 'Loading…' : 'Select a line'}
        disabled={simsState === 'loading' || sims.length === 0}
        label="Send from line"
      />
    </Field>

    <Field label="Message" forId="new-message">
      <Textarea id="new-message" bind:value={message} rows={4} disabled={busy} />
    </Field>

    {#if formError || error}
      <p class="new__error" role="alert">{formError || error}</p>
    {/if}

    <button type="submit" class="new__submit-proxy" tabindex="-1" aria-hidden="true">Send</button>
  </form>

  {#snippet footer()}
    <Button variant="ghost" onclick={() => (open = false)} disabled={busy}>Cancel</Button>
    <Button variant="primary" onclick={submit} loading={busy} disabled={!canSend}>Send</Button>
  {/snippet}
</Dialog>

<style>
  .new {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-width: 0;
  }

  .new__error {
    margin: 0;
    padding: 8px 10px;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--danger);
    font-size: var(--text-sm);
    line-height: 1.45;
  }

  .new__submit-proxy {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    border: 0;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
  }
</style>
