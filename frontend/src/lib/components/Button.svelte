<script>
  import Spinner from './Spinner.svelte';

  let {
    variant = 'secondary',
    size = 'md',
    type = 'button',
    disabled = false,
    loading = false,
    block = false,
    href = '',
    title = '',
    ariaLabel = '',
    onclick,
    children
  } = $props();

  const inert = $derived(disabled || loading);
</script>

{#if href && !inert}
  <a class="btn btn--{variant} btn--{size}" {href} {title} aria-label={ariaLabel || undefined}>
    {#if loading}<Spinner size={12} />{/if}
    {@render children?.()}
  </a>
{:else}
  <button
    class="btn btn--{variant} btn--{size}"
    class:btn--block={block}
    class:btn--loading={loading}
    {type}
    disabled={inert}
    {title}
    aria-label={ariaLabel || undefined}
    aria-busy={loading || undefined}
    {onclick}
  >
    {#if loading}<Spinner size={12} />{/if}
    {@render children?.()}
  </button>
{/if}

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 36px;
    padding: 0 16px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: var(--raised);
    color: var(--ink);
    font-size: var(--text-sm);
    font-weight: 500;
    line-height: 1;
    white-space: nowrap;
    transition:
      background var(--dur) ease,
      border-color var(--dur) ease,
      color var(--dur) ease,
      transform var(--dur-fast) var(--ease-out);
  }

  .btn:hover:not(:disabled) {
    background: var(--sunken);
    border-color: var(--line-strong);
  }

  .btn:active:not(:disabled) {
    transform: translateY(0.5px) scale(0.99);
  }

  .btn:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }

  /* primary: solid ink */
  .btn--primary {
    background: var(--ink);
    border-color: var(--ink);
    color: var(--paper);
  }

  .btn--primary:hover:not(:disabled) {
    background: #000;
    border-color: #000;
  }

  /* ghost: quiet inline action */
  .btn--ghost {
    background: transparent;
    border-color: transparent;
    color: var(--ink-2);
  }

  .btn--ghost:hover:not(:disabled) {
    background: var(--sunken);
    border-color: transparent;
    color: var(--ink);
  }

  /* danger: destructive, outlined until hovered */
  .btn--danger {
    background: transparent;
    border-color: color-mix(in srgb, var(--danger) 45%, var(--line));
    color: var(--danger);
  }

  .btn--danger:hover:not(:disabled) {
    background: color-mix(in srgb, var(--danger) 8%, var(--raised));
    border-color: var(--danger);
    color: var(--danger);
  }

  .btn--sm {
    height: 30px;
    padding: 0 11px;
    font-size: var(--text-xs);
    gap: 5px;
  }

  .btn--lg {
    height: 40px;
    padding: 0 18px;
    font-size: var(--text-base);
  }

  .btn--block {
    display: flex;
    width: 100%;
  }

  .btn--loading {
    cursor: progress;
  }
</style>
