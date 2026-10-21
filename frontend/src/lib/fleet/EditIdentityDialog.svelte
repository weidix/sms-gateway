<script>
  import Dialog from '$lib/components/Dialog.svelte';
  import Button from '$lib/components/Button.svelte';
  import Field from '$lib/components/Field.svelte';
  import Input from '$lib/components/Input.svelte';
  import { api } from '$lib/api.js';
  import { toastOk } from '$lib/stores/toast.svelte.js';

  let { open = $bindable(false), simId, alias = '', phone = '', onSaved } = $props();

  let aliasDraft = $state('');
  let phoneDraft = $state('');
  let saving = $state(false);
  let error = $state('');

  $effect(() => {
    if (!open) return;
    aliasDraft = alias ?? '';
    phoneDraft = phone ?? '';
    error = '';
  });

  async function save() {
    if (saving) return;
    saving = true;
    error = '';
    try {
      await api.updateAlias(simId, aliasDraft.trim() || null);
      await api.updatePhone(simId, phoneDraft.trim() || null);
      open = false;
      toastOk('Line identity updated.');
      await onSaved?.();
    } catch (cause) {
      error = cause?.message ?? 'Could not update the line identity.';
    } finally {
      saving = false;
    }
  }
</script>

{#snippet footer()}
  <Button variant="ghost" disabled={saving} onclick={() => (open = false)}>Cancel</Button>
  <Button variant="primary" loading={saving} disabled={saving} onclick={save}>Save</Button>
{/snippet}

<Dialog bind:open title="Edit identity" size="sm" footer={footer}>
  <div class="form-grid form-grid--single">
    <Field label="Name" forId="identity-alias">
      <Input id="identity-alias" bind:value={aliasDraft} placeholder="Desk line" disabled={saving} />
    </Field>
    <Field label="Phone" forId="identity-phone">
      <Input
        id="identity-phone"
        bind:value={phoneDraft}
        placeholder="+00 000 000 000"
        disabled={saving}
      />
    </Field>
    {#if error}
      <p class="dialog-error" role="alert">{error}</p>
    {/if}
  </div>
</Dialog>

<style>
  .dialog-error {
    font-size: var(--text-sm);
    color: var(--danger);
  }
</style>
