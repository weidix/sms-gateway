<script>
  import Icon from './Icon.svelte';

  let {
    name,
    label = '',
    size = 30,
    variant = 'ghost',
    disabled = false,
    onclick,
    children
  } = $props();
</script>

<button
  class="icon-btn icon-btn--{variant}"
  style="--box: {size}px"
  {disabled}
  title={label || undefined}
  aria-label={label || name}
  {onclick}
>
  {#if children}
    {@render children()}
  {:else}
    <Icon {name} size={size <= 30 ? 15 : 16} />
  {/if}
</button>

<style>
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--box);
    height: var(--box);
    padding: 0;
    border: 1px solid transparent;
    border-radius: 10px;
    background: transparent;
    color: var(--ink-3);
    transition:
      background var(--dur) ease,
      color var(--dur) ease,
      border-color var(--dur) ease;
  }

  .icon-btn:hover:not(:disabled) {
    background: var(--sunken);
    color: var(--ink);
  }

  .icon-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .icon-btn--outline {
    border-color: var(--line);
    background: var(--raised);
    color: var(--ink-2);
  }

  .icon-btn--outline:hover:not(:disabled) {
    border-color: var(--ink-3);
  }
</style>
