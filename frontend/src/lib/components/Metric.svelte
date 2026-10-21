<script>
  let { label = '', value = '', unit = '', hint = '', children } = $props();
</script>

<div class="metric">
  <span class="metric__label">{label}</span>
  {#if children}
    <div class="metric__value metric__value--custom">{@render children()}</div>
  {:else}
    <div class="metric__value num">
      {value}
      {#if unit}<span class="metric__unit">{unit}</span>{/if}
    </div>
  {/if}
  {#if hint}<span class="metric__hint">{hint}</span>{/if}
</div>

<style>
  .metric {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .metric__label {
    font-family: var(--font-mono);
    font-size: var(--text-2xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .metric__value {
    font-family: var(--font-sans);
    font-size: var(--text-xl);
    font-weight: 600;
    line-height: 1.05;
    letter-spacing: -0.03em;
    color: var(--ink);
    font-variant-numeric: tabular-nums;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .metric__value--custom {
    font-family: inherit;
    font-size: var(--text-base);
    letter-spacing: 0;
  }

  .metric__unit {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--ink-3);
    margin-left: 2px;
    letter-spacing: 0;
  }

  .metric__hint {
    font-size: var(--text-xs);
    color: var(--ink-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
