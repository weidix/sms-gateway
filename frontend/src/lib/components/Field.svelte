<script>
  let { label = '', hint = '', error = '', forId = '', children } = $props();
</script>

<div class="field" class:field--invalid={error}>
  {#if label}
    <label class="field__label" for={forId || undefined}>{label}</label>
  {/if}
  {@render children?.()}
  {#if error}
    <p class="field__message field__message--error">{error}</p>
  {:else if hint}
    <p class="field__message">{hint}</p>
  {/if}
</div>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .field__label {
    font-size: var(--text-sm);
    font-weight: 500;
    letter-spacing: 0.01em;
    color: var(--ink-2);
  }

  .field__message {
    margin: 0;
    font-size: var(--text-xs);
    line-height: 1.45;
    color: var(--ink-3);
  }

  .field__message--error {
    color: var(--danger);
  }
</style>
