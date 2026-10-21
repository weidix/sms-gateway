<script>
  /** Compact label/value rows. Mono values, tabular figures, no row rules. */
  let { items = [] } = $props();
</script>

<dl class="facts">
  {#each items as item (item.label)}
    <div class="facts__row">
      <dt class="facts__label">{item.label}</dt>
      <dd class="facts__value mono truncate" title={item.title || item.value || undefined}>
        {item.value || '—'}
      </dd>
    </div>
  {/each}
</dl>

<style>
  .facts {
    display: flex;
    flex-direction: column;
    gap: 7px;
    margin: 0;
  }

  .facts__row {
    display: grid;
    grid-template-columns: 82px minmax(0, 1fr);
    align-items: baseline;
    gap: 12px;
  }

  .facts__label {
    font-size: var(--text-xs);
    color: var(--ink-3);
    white-space: nowrap;
  }

  .facts__value {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--ink-2);
  }
</style>
